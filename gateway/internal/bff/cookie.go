package bff

import (
	"net/http"
	"slices"
	"strings"
	"time"
)

// cookiePrefix marks the gateway's cookies; __Host-Http- makes browsers demand Secure, Path=/, host-only and set by HTTP only.
const (
	cookiePrefix  = "__Host-Http-gw-"
	loginCookie   = cookiePrefix + "login"
	sessionCookie = cookiePrefix + "session"
)

// StripCredentials keeps only the upstream's own cookies, so auth's pages never see a gateway session or token.
func StripCredentials(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		r.Header.Del("Authorization")
		kept := slices.DeleteFunc(r.Cookies(), func(c *http.Cookie) bool {
			return strings.HasPrefix(c.Name, cookiePrefix)
		})
		r.Header.Del("Cookie")
		for _, c := range kept {
			r.AddCookie(c)
		}
		next.ServeHTTP(w, r)
	})
}

// setCookie uses SameSite=Lax, not Strict, so a link from an email or another site still arrives signed in.
func setCookie(w http.ResponseWriter, name, value string, maxAge time.Duration) {
	http.SetCookie(w, &http.Cookie{
		Name:     name,
		Value:    value,
		Path:     "/",
		MaxAge:   int(maxAge.Seconds()),
		HttpOnly: true,
		Secure:   true,
		SameSite: http.SameSiteLaxMode,
	})
}

func clearCookie(w http.ResponseWriter, name string) {
	setCookie(w, name, "", -time.Second)
}
