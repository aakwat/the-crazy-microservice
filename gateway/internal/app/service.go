// Package app holds the gateway's use cases; it reaches storage and auth only through domain ports.
package app

import (
	"context"
	"crypto/rand"
	"crypto/subtle"
	"errors"
	"fmt"
	"time"

	"github.com/aakwat/the-crazy-microservice/gateway/internal/domain"
)

// ErrNoSession means the caller must log in again: no session, expired, or its refresh failed.
var ErrNoSession = errors.New("no valid session")

// ErrLoginInvalid means a callback that is expired, replayed, or does not match its login.
var ErrLoginInvalid = errors.New("login is invalid or expired")

const (
	pendingTTL   = 10 * time.Minute
	lockTTL      = 5 * time.Second
	refreshSkew  = 30 * time.Second
	waitInterval = 100 * time.Millisecond
	waitAttempts = 30
)

// Service runs the browser login, logout and per-request token use cases.
type Service struct {
	store    domain.SessionStore
	idp      domain.IdentityProvider
	lifetime domain.Lifetime
	now      func() time.Time
}

// New returns a Service; now is injected so lifetimes can be tested without waiting.
func New(store domain.SessionStore, idp domain.IdentityProvider, lifetime domain.Lifetime, now func() time.Time) *Service {
	return &Service{store: store, idp: idp, lifetime: lifetime, now: now}
}

// LoginStart is what the browser needs: a cookie value and where to go next.
type LoginStart struct {
	LoginID     string
	RedirectURL string
	TTL         time.Duration
}

// StartLogin remembers a fresh state, nonce and PKCE verifier and returns auth's login URL.
func (s *Service) StartLogin(ctx context.Context) (LoginStart, error) {
	loginID := rand.Text()
	pending := domain.PendingLogin{
		State: rand.Text(),
		Nonce: rand.Text(),
		// Two 26-character values give a 52-character verifier, inside RFC 7636's 43 to 128.
		Verifier: rand.Text() + rand.Text(),
	}
	if err := s.store.SavePending(ctx, loginID, pending, pendingTTL); err != nil {
		return LoginStart{}, fmt.Errorf("save pending login: %w", err)
	}
	return LoginStart{
		LoginID:     loginID,
		RedirectURL: s.idp.AuthCodeURL(pending.State, pending.Verifier, pending.Nonce),
		TTL:         pendingTTL,
	}, nil
}

// CompleteLogin redeems the callback once and returns a new session id with its storage TTL.
func (s *Service) CompleteLogin(ctx context.Context, loginID, state, code string) (string, time.Duration, error) {
	pending, ok, err := s.store.TakePending(ctx, loginID)
	if err != nil {
		return "", 0, fmt.Errorf("take pending login: %w", err)
	}
	if !ok || subtle.ConstantTimeCompare([]byte(state), []byte(pending.State)) != 1 {
		return "", 0, ErrLoginInvalid
	}
	tokens, err := s.idp.Exchange(ctx, code, pending.Verifier, pending.Nonce)
	if err != nil {
		return "", 0, fmt.Errorf("%w: %w", ErrLoginInvalid, err)
	}

	now := s.now()
	sess := domain.Session{
		Subject:      tokens.Subject,
		AccessToken:  tokens.AccessToken,
		RefreshToken: tokens.RefreshToken,
		IDToken:      tokens.IDToken,
		AccessExpiry: tokens.AccessExpiry,
		LoginAt:      now,
	}
	// A brand-new id for every login, so a session id planted before login is useless.
	id := rand.Text()
	ttl := s.lifetime.TTL(sess, now)
	if err := s.store.Save(ctx, id, sess, ttl); err != nil {
		return "", 0, fmt.Errorf("save session: %w", err)
	}
	return id, ttl, nil
}

// AccessToken returns a usable access token for the session, refreshing it when close to expiry.
func (s *Service) AccessToken(ctx context.Context, sessionID string) (string, error) {
	sess, ok, err := s.store.Get(ctx, sessionID)
	if err != nil {
		return "", fmt.Errorf("get session: %w", err)
	}
	if !ok {
		return "", ErrNoSession
	}

	now := s.now()
	if s.lifetime.Expired(sess, now) {
		_ = s.store.Delete(ctx, sessionID)
		return "", ErrNoSession
	}
	if !sess.NeedsRefresh(now, refreshSkew) {
		return sess.AccessToken, nil
	}
	return s.refresh(ctx, sessionID, sess)
}

// refresh lets one caller rotate the tokens; with rotation, a second refresh would end the session.
func (s *Service) refresh(ctx context.Context, sessionID string, sess domain.Session) (string, error) {
	locked, err := s.store.TryLock(ctx, sessionID, lockTTL)
	if err != nil {
		return "", fmt.Errorf("lock session: %w", err)
	}
	if !locked {
		return s.awaitRefresh(ctx, sessionID)
	}
	defer func() { _ = s.store.Unlock(context.WithoutCancel(ctx), sessionID) }()

	tokens, err := s.idp.Refresh(ctx, sess.RefreshToken)
	if err != nil {
		_ = s.store.Delete(ctx, sessionID)
		return "", fmt.Errorf("%w: %w", ErrNoSession, err)
	}
	sess.AccessToken = tokens.AccessToken
	sess.AccessExpiry = tokens.AccessExpiry
	if tokens.RefreshToken != "" {
		sess.RefreshToken = tokens.RefreshToken
	}
	if err := s.store.Save(ctx, sessionID, sess, s.lifetime.TTL(sess, s.now())); err != nil {
		return "", fmt.Errorf("save session: %w", err)
	}
	return sess.AccessToken, nil
}

// awaitRefresh polls until the lock holder has saved fresh tokens; polling is cheap at this volume.
func (s *Service) awaitRefresh(ctx context.Context, sessionID string) (string, error) {
	for range waitAttempts {
		select {
		case <-ctx.Done():
			return "", ctx.Err()
		case <-time.After(waitInterval):
		}
		sess, ok, err := s.store.Get(ctx, sessionID)
		if err != nil {
			return "", fmt.Errorf("get session: %w", err)
		}
		if !ok {
			return "", ErrNoSession
		}
		if !sess.NeedsRefresh(s.now(), refreshSkew) {
			return sess.AccessToken, nil
		}
	}
	return "", errors.New("timed out waiting for a concurrent token refresh")
}

// Logout revokes and deletes the session and returns auth's end-session URL (empty without a session).
func (s *Service) Logout(ctx context.Context, sessionID string) (string, error) {
	sess, ok, err := s.store.Get(ctx, sessionID)
	if err != nil {
		return "", fmt.Errorf("get session: %w", err)
	}
	if !ok {
		return "", nil
	}
	revokeErr := s.idp.Revoke(ctx, sess.RefreshToken)
	if err := s.store.Delete(ctx, sessionID); err != nil {
		return "", fmt.Errorf("delete session: %w", err)
	}
	return s.idp.EndSessionURL(sess.IDToken), revokeErr
}

// VerifyBearer checks a token that a non-browser client sent itself.
func (s *Service) VerifyBearer(ctx context.Context, raw string) error {
	return s.idp.VerifyAccessToken(ctx, raw)
}
