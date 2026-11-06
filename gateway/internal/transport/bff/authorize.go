package bff

import (
	"errors"
	"fmt"
	"net/http"
	"strings"

	"github.com/aakwat/the-crazy-microservice/gateway/internal/app"
	"github.com/aakwat/the-crazy-microservice/gateway/internal/transport/problem"
)

var errUnauthenticated = errors.New("no session cookie or valid bearer token")

// Authorize puts an access token on the upstream request, from the session cookie or a verified bearer.
func (h *Handler) Authorize(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		token, err := h.accessToken(r)
		if errors.Is(err, app.ErrNoSession) || errors.Is(err, errUnauthenticated) {
			if errors.Is(err, app.ErrNoSession) {
				clearCookie(w, sessionCookie)
			}
			w.Header().Set("WWW-Authenticate", "Bearer")
			problem.Write(w, http.StatusUnauthorized, "log in or send a valid access token")
			return
		}
		if err != nil {
			h.fail(w, r, err)
			return
		}
		r.Header.Set("Authorization", "Bearer "+token)
		// Services authenticate by token only; the gateway's cookies never leave it.
		r.Header.Del("Cookie")
		next.ServeHTTP(w, r)
	})
}

// accessToken prefers the session cookie; a bearer token must pass the same checks every service applies.
func (h *Handler) accessToken(r *http.Request) (string, error) {
	if c, err := r.Cookie(sessionCookie); err == nil {
		return h.svc.AccessToken(r.Context(), c.Value)
	}
	scheme, raw, ok := strings.Cut(r.Header.Get("Authorization"), " ")
	if !ok || !strings.EqualFold(scheme, "Bearer") || raw == "" {
		return "", errUnauthenticated
	}
	if err := h.svc.VerifyBearer(r.Context(), raw); err != nil {
		return "", fmt.Errorf("%w: %w", errUnauthenticated, err)
	}
	return raw, nil
}
