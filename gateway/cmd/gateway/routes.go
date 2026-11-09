package main

import (
	"context"
	"crypto/rand"
	"encoding/hex"
	"errors"
	"log/slog"
	"net/http"
	"net/http/httputil"
	"net/url"

	"github.com/aakwat/the-crazy-microservice/gateway/internal/bff"
)

// authPages are auth's own browser and discovery endpoints, served on the gateway's origin.
var authPages = []string{
	"GET /login",
	"GET /oauth2/authorize", "POST /oauth2/authorize",
	"GET /connect/logout", "POST /connect/logout",
	"GET /.well-known/", "GET /oauth2/jwks",
}

// securityHeaders go on every response; the gateway's values replace any that an upstream sends.
var securityHeaders = map[string]string{
	"X-Content-Type-Options": "nosniff",
	"Referrer-Policy":        "no-referrer",
	"X-Frame-Options":        "DENY",
}

// routes returns the gateway's whole public surface; any path not listed, /internal/* included, is a 404.
func routes(b *bff.BFF, auth http.Handler) http.Handler {
	mux := http.NewServeMux()
	// Each login start writes a Redis key, so anonymous callers get a budget.
	mux.Handle("GET /bff/login", perIP(loginPerMin, http.HandlerFunc(b.Login)))
	mux.HandleFunc("GET /bff/callback", b.Callback)
	mux.HandleFunc("GET /bff/logout", b.LogoutPage)
	mux.HandleFunc("POST /bff/logout", b.Logout)
	mux.HandleFunc("GET /bff/logged-out", b.LoggedOut)
	mux.Handle("GET /bff/user", b.Authorize(rewritePath("/userinfo", auth)))

	pages := bff.StripCredentials(auth)
	for _, pattern := range authPages {
		mux.Handle(pattern, pages)
	}
	// Each password attempt costs auth an Argon2 hash, and per-account lockout cannot stop a spray.
	mux.Handle("POST /login", perIP(loginPerMin, pages))
	mux.Handle("POST /auth/register", perIP(registerPerMin, pages))

	mux.HandleFunc("GET /healthz", func(w http.ResponseWriter, _ *http.Request) {
		w.WriteHeader(http.StatusNoContent)
	})
	mux.HandleFunc("/", func(w http.ResponseWriter, _ *http.Request) {
		bff.Problem(w, http.StatusNotFound, "")
	})

	cop := http.NewCrossOriginProtection()
	cop.SetDenyHandler(http.HandlerFunc(func(w http.ResponseWriter, _ *http.Request) {
		bff.Problem(w, http.StatusForbidden, "cross-origin request refused")
	}))
	return edge(cop.Handler(mux))
}

// edge runs before every route: a fresh trace, baseline security headers and a body cap.
func edge(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		// A client's traceparent is untrusted, so every request starts a new trace.
		r.Header.Set("Traceparent", newTraceparent())
		r.Header.Del("Tracestate")
		for name, value := range securityHeaders {
			w.Header().Set(name, value)
		}
		if r.ContentLength > maxBodyBytes {
			bff.Problem(w, http.StatusRequestEntityTooLarge, "request body is too large")
			return
		}
		r.Body = http.MaxBytesReader(w, r.Body, maxBodyBytes)
		next.ServeHTTP(w, r)
	})
}

// newTraceparent builds a W3C traceparent: version 00, a 16-byte trace id, an 8-byte span id, sampled.
func newTraceparent() string {
	var b [24]byte
	_, _ = rand.Read(b[:])
	return "00-" + hex.EncodeToString(b[:16]) + "-" + hex.EncodeToString(b[16:]) + "-01"
}

// newProxy forwards to target without retries; forwarded host and scheme come from config, never the client.
func newProxy(target, public *url.URL, log *slog.Logger) *httputil.ReverseProxy {
	transport := http.DefaultTransport.(*http.Transport).Clone()
	transport.Proxy = nil
	transport.ResponseHeaderTimeout = upstreamTimeout
	return &httputil.ReverseProxy{
		Rewrite: func(pr *httputil.ProxyRequest) {
			pr.SetURL(target)
			pr.SetXForwarded()
			pr.Out.Header.Set("X-Forwarded-Host", public.Host)
			pr.Out.Header.Set("X-Forwarded-Proto", public.Scheme)
		},
		ModifyResponse: func(res *http.Response) error {
			for name := range securityHeaders {
				res.Header.Del(name)
			}
			return nil
		},
		Transport: transport,
		ErrorHandler: func(w http.ResponseWriter, r *http.Request, err error) {
			var tooLarge *http.MaxBytesError
			switch {
			case errors.As(err, &tooLarge):
				bff.Problem(w, http.StatusRequestEntityTooLarge, "request body is too large")
			case errors.Is(err, context.Canceled):
				// The client hung up, so nobody is left to read an answer.
			default:
				log.WarnContext(r.Context(), "upstream failed", "err", err, "traceparent", r.Header.Get("Traceparent"))
				bff.Problem(w, http.StatusBadGateway, "upstream service is unavailable")
			}
		},
	}
}

// rewritePath serves the request from path upstream, so the gateway's URL space stays its own.
func rewritePath(path string, next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		u := *r.URL
		u.Path, u.RawPath = path, ""
		r2 := r.WithContext(r.Context())
		r2.URL = &u
		next.ServeHTTP(w, r2)
	})
}
