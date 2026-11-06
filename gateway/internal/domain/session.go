// Package domain holds the gateway's session rules and ports; it imports only the standard library.
package domain

import "time"

// Session holds a user's tokens server-side; the browser only ever sees its random id.
type Session struct {
	Subject      string    `json:"sub"`
	AccessToken  string    `json:"access_token"`
	RefreshToken string    `json:"refresh_token"`
	IDToken      string    `json:"id_token"`
	AccessExpiry time.Time `json:"access_expiry"`
	LoginAt      time.Time `json:"login_at"`
}

// PendingLogin is what /bff/login must remember until auth redirects back to /bff/callback.
type PendingLogin struct {
	State    string `json:"state"`
	Verifier string `json:"verifier"`
	Nonce    string `json:"nonce"`
}

// Tokens is what the identity provider returns after a code exchange or a refresh.
type Tokens struct {
	Subject      string
	AccessToken  string
	RefreshToken string
	IDToken      string
	AccessExpiry time.Time
}

// Lifetime caps a session twice: Idle resets on every refresh, Absolute never moves.
type Lifetime struct {
	Idle     time.Duration
	Absolute time.Duration
}

// NeedsRefresh reports whether the access token expires within skew of now.
func (s Session) NeedsRefresh(now time.Time, skew time.Duration) bool {
	return !now.Add(skew).Before(s.AccessExpiry)
}

// Expired reports whether the absolute lifetime has passed; auth's rotation would otherwise never end it.
func (l Lifetime) Expired(s Session, now time.Time) bool {
	return !now.Before(s.LoginAt.Add(l.Absolute))
}

// TTL is how long storage keeps the session: the idle window, never past the absolute end.
func (l Lifetime) TTL(s Session, now time.Time) time.Duration {
	return min(l.Idle, s.LoginAt.Add(l.Absolute).Sub(now))
}
