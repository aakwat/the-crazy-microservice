package bff

import (
	"context"
	"crypto/rand"
	"crypto/subtle"
	"errors"
	"fmt"
	"net/http"
	"net/url"
	"strings"

	"github.com/coreos/go-oidc/v3/oidc"
	"golang.org/x/oauth2"
)

// Login starts the code flow with PKCE (S256), state and nonce, and sends the browser to auth.
func (b *BFF) Login(w http.ResponseWriter, r *http.Request) {
	id := rand.Text()
	p := pending{State: rand.Text(), Verifier: oauth2.GenerateVerifier(), Nonce: rand.Text()}
	if err := b.savePending(r.Context(), id, p); err != nil {
		b.fail(w, r, http.StatusServiceUnavailable, "session store is unavailable", err)
		return
	}
	setCookie(w, loginCookie, id, pendingTTL)
	target := b.oauth.AuthCodeURL(p.State, oauth2.S256ChallengeOption(p.Verifier), oidc.Nonce(p.Nonce))
	http.Redirect(w, r, target, http.StatusFound)
}

// Callback redeems the code once and starts a session under a fresh id, so an id planted earlier is useless.
func (b *BFF) Callback(w http.ResponseWriter, r *http.Request) {
	ctx, q := r.Context(), r.URL.Query()
	c, err := r.Cookie(loginCookie)
	if err != nil || q.Has("error") {
		Problem(w, http.StatusBadRequest, "login was cancelled or expired; start again")
		return
	}
	clearCookie(w, loginCookie)

	p, ok, err := b.takePending(ctx, c.Value)
	if err != nil {
		b.fail(w, r, http.StatusServiceUnavailable, "session store is unavailable", err)
		return
	}
	if !ok || subtle.ConstantTimeCompare([]byte(q.Get("state")), []byte(p.State)) != 1 {
		Problem(w, http.StatusBadRequest, "login expired or is invalid; start again")
		return
	}
	tok, err := b.oauth.Exchange(b.client(ctx), q.Get("code"), oauth2.VerifierOption(p.Verifier))
	if rejected(err) {
		Problem(w, http.StatusBadRequest, "login expired or is invalid; start again")
		return
	}
	if err != nil {
		b.fail(w, r, http.StatusBadGateway, "auth is unavailable", err)
		return
	}
	if tok.RefreshToken == "" {
		b.fail(w, r, http.StatusBadGateway, "auth issued no refresh token", errors.New("token response without refresh_token"))
		return
	}
	rawID, _ := tok.Extra("id_token").(string)
	idToken, err := b.idTokens.Verify(ctx, rawID)
	if err != nil || subtle.ConstantTimeCompare([]byte(idToken.Nonce), []byte(p.Nonce)) != 1 {
		b.log.WarnContext(ctx, "id token rejected", "err", err, "traceparent", r.Header.Get("Traceparent"))
		Problem(w, http.StatusBadRequest, "login expired or is invalid; start again")
		return
	}

	id := rand.Text()
	s := session{
		Subject:      idToken.Subject,
		AccessToken:  tok.AccessToken,
		RefreshToken: tok.RefreshToken,
		IDToken:      rawID,
		Expiry:       tok.Expiry,
		LoginAt:      b.now(),
	}
	if err := b.createSession(ctx, id, s); err != nil {
		b.fail(w, r, http.StatusServiceUnavailable, "session store is unavailable", err)
		return
	}
	setCookie(w, sessionCookie, id, absoluteTTL)
	http.Redirect(w, r, b.cfg.AfterLoginPath, http.StatusSeeOther)
}

// LogoutPage only shows a form: a cross-site link can trigger a GET, which must never end a session.
func (b *BFF) LogoutPage(w http.ResponseWriter, _ *http.Request) {
	w.Header().Set("Content-Type", "text/html; charset=utf-8")
	_, _ = w.Write([]byte(logoutForm))
}

const logoutForm = `<!doctype html>
<title>Log out</title>
<form method="post" action="/bff/logout"><button type="submit">Log out</button></form>
`

// Logout revokes the refresh token, forgets the session and ends auth's own sign-on session too.
func (b *BFF) Logout(w http.ResponseWriter, r *http.Request) {
	ctx := r.Context()
	target := b.cfg.AfterLogoutPath
	if c, err := r.Cookie(sessionCookie); err == nil {
		s, ok, err := b.loadSession(ctx, c.Value)
		if err != nil {
			b.log.WarnContext(ctx, "session not loaded for logout", "err", err)
		}
		if ok {
			if err := b.revoke(ctx, s.RefreshToken); err != nil {
				b.log.WarnContext(ctx, "refresh token not revoked", "err", err)
			}
			target = b.cfg.Issuer + "/connect/logout?" + url.Values{
				"id_token_hint":            {s.IDToken},
				"post_logout_redirect_uri": {b.cfg.PublicURL + b.cfg.AfterLogoutPath},
			}.Encode()
		}
		if err := b.rdb.Del(ctx, sessionKey+c.Value).Err(); err != nil {
			b.log.WarnContext(ctx, "session not deleted", "err", err)
		}
	}
	clearCookie(w, sessionCookie)
	http.Redirect(w, r, target, http.StatusSeeOther)
}

// LoggedOut is the default page auth sends the browser to after its own logout.
func (b *BFF) LoggedOut(w http.ResponseWriter, _ *http.Request) {
	w.Header().Set("Content-Type", "text/plain; charset=utf-8")
	_, _ = w.Write([]byte("You are logged out.\n"))
}

// revoke tells auth to invalidate a refresh token (RFC 7009).
func (b *BFF) revoke(ctx context.Context, token string) error {
	form := url.Values{"token": {token}, "token_type_hint": {"refresh_token"}}
	req, err := http.NewRequestWithContext(ctx, http.MethodPost, b.cfg.AuthURL+"/oauth2/revoke",
		strings.NewReader(form.Encode()))
	if err != nil {
		return err
	}
	req.Header.Set("Content-Type", "application/x-www-form-urlencoded")
	// RFC 6749 section 2.3.1 form-encodes client credentials before Basic auth, as x/oauth2 does.
	req.SetBasicAuth(url.QueryEscape(b.cfg.ClientID), url.QueryEscape(b.cfg.ClientSecret))
	res, err := b.cfg.HTTPClient.Do(req)
	if err != nil {
		return err
	}
	defer res.Body.Close()
	if res.StatusCode != http.StatusOK {
		return fmt.Errorf("auth answered %d", res.StatusCode)
	}
	return nil
}
