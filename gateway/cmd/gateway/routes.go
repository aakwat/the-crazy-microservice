package main

import (
	"net/http"

	"github.com/aakwat/the-crazy-microservice/gateway/internal/bff"
)

// authPages are auth's own browser and discovery endpoints, served on the gateway's origin.
var authPages = []string{
	"GET /login",
	"GET /oauth2/authorize", "POST /oauth2/authorize",
	"GET /connect/logout", "POST /connect/logout",
	"GET /.well-known/", "GET /oauth2/jwks",
}

// ticketRoutes reach tickets only with an access token; tickets itself decides who may do what.
var ticketRoutes = []string{
	"POST /api/tickets", "GET /api/tickets",
	"GET /api/tickets/{id}", "PATCH /api/tickets/{id}",
	"GET /api/tickets/{id}/messages", "POST /api/tickets/{id}/messages",
	"POST /api/tickets/{id}/confirm", "POST /api/tickets/{id}/withdraw",
}

// webRoutes are the browser UI's pages and built assets; each API call it makes is checked on its own route.
// POST is the UI's server actions, which post to the page they run on; cross-origin protection covers them.
var webRoutes = []string{
	"GET /{$}", "GET /register", "GET /tickets/new", "GET /tickets/{id}", "GET /_next/", "GET /favicon.ico",
	"POST /{$}", "POST /register", "POST /tickets/new", "POST /tickets/{id}",
}

// accountRoutes are auth's admin API; auth itself admits only the admin role.
var accountRoutes = []string{"GET /auth/accounts", "PUT /auth/accounts/{id}/role"}

// routes returns the gateway's whole public surface; any path not listed, /internal/* included, is a 404.
func routes(b *bff.BFF, auth, tickets, web http.Handler) http.Handler {
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

	api := b.Authorize(tickets)
	for _, pattern := range ticketRoutes {
		mux.Handle(pattern, api)
	}
	admin := b.Authorize(auth)
	for _, pattern := range accountRoutes {
		mux.Handle(pattern, admin)
	}

	for _, pattern := range webRoutes {
		mux.Handle(pattern, web)
	}

	mux.HandleFunc("/", func(w http.ResponseWriter, _ *http.Request) {
		bff.Problem(w, http.StatusNotFound, "")
	})

	cop := http.NewCrossOriginProtection()
	cop.SetDenyHandler(http.HandlerFunc(func(w http.ResponseWriter, _ *http.Request) {
		bff.Problem(w, http.StatusForbidden, "cross-origin request refused")
	}))

	root := http.NewServeMux()
	// Outside the edge: the container probe runs every few seconds and would flood the traces.
	root.HandleFunc("GET /healthz", func(w http.ResponseWriter, _ *http.Request) {
		w.WriteHeader(http.StatusNoContent)
	})
	root.Handle("/", edge(cop.Handler(mux)))
	return root
}
