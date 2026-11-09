package bff

import (
	"context"
	"encoding/base64"
	"encoding/json"
	"errors"
	"fmt"
	"net/http"
	"slices"
	"strings"
	"time"

	"golang.org/x/oauth2"
)

var errNoSession = errors.New("no valid session or access token")

// Authorize sets the upstream Authorization from the session cookie or a verified bearer; cookies stay here.
func (b *BFF) Authorize(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		token, err := b.accessToken(r)
		if errors.Is(err, errNoSession) {
			if _, cookieErr := r.Cookie(sessionCookie); cookieErr == nil {
				clearCookie(w, sessionCookie)
			}
			w.Header().Set("WWW-Authenticate", "Bearer")
			Problem(w, http.StatusUnauthorized, "log in or send a valid access token")
			return
		}
		if err != nil {
			b.fail(w, r, http.StatusServiceUnavailable, "sign-in is unavailable; try again", err)
			return
		}
		r.Header.Set("Authorization", "Bearer "+token)
		r.Header.Del("Cookie")
		next.ServeHTTP(w, r)
	})
}

// accessToken prefers the session cookie; a bearer token must pass the checks every service applies.
func (b *BFF) accessToken(r *http.Request) (string, error) {
	if c, err := r.Cookie(sessionCookie); err == nil {
		return b.sessionToken(r.Context(), c.Value)
	}
	scheme, raw, ok := strings.Cut(r.Header.Get("Authorization"), " ")
	if !ok || !strings.EqualFold(scheme, "Bearer") || raw == "" {
		return "", errNoSession
	}
	if err := b.verifyBearer(r.Context(), raw); err != nil {
		return "", fmt.Errorf("%w: %w", errNoSession, err)
	}
	return raw, nil
}

// verifyBearer accepts only RFC 9068 access tokens (typ at+jwt) for our audience; go-oidc does not check typ.
func (b *BFF) verifyBearer(ctx context.Context, raw string) error {
	if typ, err := jwtType(raw); err != nil || !strings.EqualFold(typ, "at+jwt") {
		return errors.New("typ must be at+jwt")
	}
	token, err := b.tokens.Verify(ctx, raw)
	if err != nil {
		return err
	}
	if !slices.Contains(token.Audience, b.cfg.Audience) {
		return errors.New("wrong audience")
	}
	return nil
}

// jwtType reads the unverified typ header; verifyBearer checks the signature right after.
func jwtType(raw string) (string, error) {
	encoded, _, _ := strings.Cut(raw, ".")
	decoded, err := base64.RawURLEncoding.DecodeString(encoded)
	if err != nil {
		return "", err
	}
	var header struct {
		Typ string `json:"typ"`
	}
	err = json.Unmarshal(decoded, &header)
	return header.Typ, err
}

// sessionToken returns the session's access token, refreshing it when it is about to expire.
func (b *BFF) sessionToken(ctx context.Context, id string) (string, error) {
	s, ok, err := b.loadSession(ctx, id)
	if err != nil {
		return "", err
	}
	if !ok || s.ttl(b.now()) <= 0 {
		_ = b.rdb.Del(ctx, sessionKey+id).Err()
		return "", errNoSession
	}
	if !s.needsRefresh(b.now()) {
		return s.AccessToken, nil
	}
	return b.refresh(ctx, id)
}

// refresh lets one replica rotate the tokens; with rotation, refreshing the same token twice ends the session.
func (b *BFF) refresh(ctx context.Context, id string) (string, error) {
	locked, err := b.rdb.SetNX(ctx, lockKey+id, "1", lockTTL).Result()
	if err != nil {
		return "", err
	}
	if !locked {
		return b.awaitRefresh(ctx, id)
	}
	defer func() { _ = b.rdb.Del(context.WithoutCancel(ctx), lockKey+id).Err() }()
	// Finish even if the browser leaves: once auth rotates, losing the new token would end the session.
	ctx = context.WithoutCancel(ctx)

	// Reread under the lock: another replica may have rotated already, and the old token is now dead.
	s, ok, err := b.loadSession(ctx, id)
	if err != nil {
		return "", err
	}
	if !ok {
		return "", errNoSession
	}
	if !s.needsRefresh(b.now()) {
		return s.AccessToken, nil
	}

	tok, err := b.oauth.TokenSource(b.client(ctx), &oauth2.Token{RefreshToken: s.RefreshToken}).Token()
	if rejected(err) {
		_ = b.rdb.Del(ctx, sessionKey+id).Err()
		return "", fmt.Errorf("%w: %w", errNoSession, err)
	}
	if err != nil {
		// Keep the session: auth may only be restarting, and the next request retries.
		return "", fmt.Errorf("refresh: %w", err)
	}
	s.AccessToken, s.Expiry = tok.AccessToken, tok.Expiry
	if tok.RefreshToken != "" {
		s.RefreshToken = tok.RefreshToken
	}
	live, err := b.replaceSession(ctx, id, s)
	if err != nil {
		return "", err
	}
	if !live {
		// Logged out mid-refresh: the new refresh token belongs to no session, so revoke it.
		_ = b.revoke(ctx, s.RefreshToken)
		return "", errNoSession
	}
	return s.AccessToken, nil
}

// awaitRefresh polls until the lock holder has saved fresh tokens; polling is cheap at this volume.
func (b *BFF) awaitRefresh(ctx context.Context, id string) (string, error) {
	for range int(lockTTL / waitInterval) {
		select {
		case <-ctx.Done():
			return "", ctx.Err()
		case <-time.After(waitInterval):
		}
		s, ok, err := b.loadSession(ctx, id)
		if err != nil {
			return "", err
		}
		if !ok {
			return "", errNoSession
		}
		if !s.needsRefresh(b.now()) {
			return s.AccessToken, nil
		}
	}
	return "", errors.New("timed out waiting for a concurrent refresh")
}
