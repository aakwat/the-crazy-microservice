package main

import (
	"context"
	"errors"
	"log/slog"
	"net/http"
	"net/http/httputil"
	"net/url"

	"github.com/aakwat/the-crazy-microservice/gateway/internal/bff"
)

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
