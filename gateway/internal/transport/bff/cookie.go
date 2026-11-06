package bff

import (
	"net/http"
	"time"
)

// The __Host- prefix makes browsers refuse these cookies unless Secure, Path=/ and host-only.
const (
	loginCookie   = "__Host-gw-login"
	sessionCookie = "__Host-gw-session"
)

// setCookie uses SameSite=Lax because the callback arrives as a top-level redirect from auth.
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
