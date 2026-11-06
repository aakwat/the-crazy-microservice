// Package bff is the browser-facing half of the gateway: login, callback, logout and cookie-to-token mapping.
package bff

import (
	"errors"
	"log/slog"
	"net/http"

	"github.com/aakwat/the-crazy-microservice/gateway/internal/app"
	"github.com/aakwat/the-crazy-microservice/gateway/internal/transport/problem"
)

// afterLoginPath is where a fresh login lands until a frontend exists.
const afterLoginPath = "/bff/user"

// Handler serves the /bff routes and the Authorize middleware.
type Handler struct {
	svc *app.Service
	log *slog.Logger
}

// New returns a Handler backed by svc.
func New(svc *app.Service, log *slog.Logger) *Handler {
	return &Handler{svc: svc, log: log}
}

// Register adds the /bff routes; logout answers GET for a plain link and POST for a form.
func (h *Handler) Register(mux *http.ServeMux) {
	mux.HandleFunc("GET /bff/login", h.login)
	mux.HandleFunc("GET /bff/callback", h.callback)
	mux.HandleFunc("GET /bff/logout", h.logout)
	mux.HandleFunc("POST /bff/logout", h.logout)
	mux.HandleFunc("GET /bff/logged-out", h.loggedOut)
}

func (h *Handler) login(w http.ResponseWriter, r *http.Request) {
	start, err := h.svc.StartLogin(r.Context())
	if err != nil {
		h.fail(w, r, err)
		return
	}
	setCookie(w, loginCookie, start.LoginID, start.TTL)
	http.Redirect(w, r, start.RedirectURL, http.StatusFound)
}

func (h *Handler) callback(w http.ResponseWriter, r *http.Request) {
	q := r.URL.Query()
	if q.Has("error") {
		problem.Write(w, http.StatusBadRequest, "login was cancelled or refused")
		return
	}
	c, err := r.Cookie(loginCookie)
	if err != nil {
		problem.Write(w, http.StatusBadRequest, "login expired or was started in another browser; start again")
		return
	}
	clearCookie(w, loginCookie)

	id, ttl, err := h.svc.CompleteLogin(r.Context(), c.Value, q.Get("state"), q.Get("code"))
	if errors.Is(err, app.ErrLoginInvalid) {
		h.log.InfoContext(r.Context(), "login rejected", "err", err, "traceparent", r.Header.Get("Traceparent"))
		problem.Write(w, http.StatusBadRequest, "login expired or is invalid; start again")
		return
	}
	if err != nil {
		h.fail(w, r, err)
		return
	}
	setCookie(w, sessionCookie, id, ttl)
	http.Redirect(w, r, afterLoginPath, http.StatusSeeOther)
}

func (h *Handler) logout(w http.ResponseWriter, r *http.Request) {
	target := "/bff/logged-out"
	if c, err := r.Cookie(sessionCookie); err == nil {
		endURL, err := h.svc.Logout(r.Context(), c.Value)
		if err != nil {
			h.log.WarnContext(r.Context(), "logout incomplete", "err", err, "traceparent", r.Header.Get("Traceparent"))
		}
		if endURL != "" {
			target = endURL
		}
	}
	clearCookie(w, sessionCookie)
	http.Redirect(w, r, target, http.StatusSeeOther)
}

func (h *Handler) loggedOut(w http.ResponseWriter, _ *http.Request) {
	w.Header().Set("Content-Type", "text/plain; charset=utf-8")
	_, _ = w.Write([]byte("You are logged out.\n"))
}

// fail logs the real error and shows the client only a generic problem.
func (h *Handler) fail(w http.ResponseWriter, r *http.Request, err error) {
	h.log.ErrorContext(r.Context(), "request failed", "err", err, "traceparent", r.Header.Get("Traceparent"))
	problem.Write(w, http.StatusInternalServerError, "unexpected error")
}
