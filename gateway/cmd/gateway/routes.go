package main

import (
	"net/http"

	"github.com/aakwat/the-crazy-microservice/gateway/internal/transport/bff"
	"github.com/aakwat/the-crazy-microservice/gateway/internal/transport/edge"
	"github.com/aakwat/the-crazy-microservice/gateway/internal/transport/problem"
)

// authPages are auth's own browser and discovery endpoints, served on the gateway's origin.
var authPages = []string{
	"GET /login", "POST /login",
	"GET /oauth2/authorize", "POST /oauth2/authorize",
	"GET /connect/logout", "POST /connect/logout",
	"GET /.well-known/", "GET /oauth2/jwks",
}

// routes is the gateway's whole public surface; any path not listed, /internal/* included, is a 404.
func routes(h *bff.Handler, auth http.Handler, register *edge.PerIP) http.Handler {
	mux := http.NewServeMux()
	h.Register(mux)
	mux.Handle("GET /bff/user", h.Authorize(rewritePath("/userinfo", auth)))

	pages := bff.HideCookies(auth)
	for _, pattern := range authPages {
		mux.Handle(pattern, pages)
	}
	mux.Handle("POST /auth/register", register.Wrap(pages))

	mux.HandleFunc("GET /healthz", func(w http.ResponseWriter, _ *http.Request) {
		w.WriteHeader(http.StatusNoContent)
	})
	mux.HandleFunc("/", func(w http.ResponseWriter, _ *http.Request) {
		problem.Write(w, http.StatusNotFound, "")
	})

	cop := http.NewCrossOriginProtection()
	cop.SetDenyHandler(http.HandlerFunc(func(w http.ResponseWriter, _ *http.Request) {
		problem.Write(w, http.StatusForbidden, "cross-origin request refused")
	}))
	return edge.Traceparent(edge.SecurityHeaders(edge.MaxBody(maxBodyBytes)(cop.Handler(mux))))
}

// rewritePath serves the request from path on the upstream, so the gateway's URL space stays its own.
func rewritePath(path string, next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		u := *r.URL
		u.Path, u.RawPath = path, ""
		r2 := r.WithContext(r.Context())
		r2.URL = &u
		next.ServeHTTP(w, r2)
	})
}
