package domain

import (
	"context"
	"errors"
	"time"
)

// ErrTokenRejected means auth refused a code or token itself; any other error may be a passing outage.
var ErrTokenRejected = errors.New("token rejected by the identity provider")

// SessionStore is the port for session and pending-login storage, shared by every replica.
type SessionStore interface {
	Save(ctx context.Context, id string, s Session, ttl time.Duration) error
	Get(ctx context.Context, id string) (Session, bool, error)
	// Replace overwrites a session only if it still exists, so a refresh cannot revive a logged-out session.
	Replace(ctx context.Context, id string, s Session, ttl time.Duration) (bool, error)
	Delete(ctx context.Context, id string) error
	SavePending(ctx context.Context, id string, p PendingLogin, ttl time.Duration) error
	// TakePending reads and deletes in one step, so a login attempt completes at most once.
	TakePending(ctx context.Context, id string) (PendingLogin, bool, error)
	// TryLock lets one replica refresh a session; with rotation a second refresh would end it.
	TryLock(ctx context.Context, id string, ttl time.Duration) (bool, error)
	Unlock(ctx context.Context, id string) error
}

// IdentityProvider is the port to the authorization server (auth).
type IdentityProvider interface {
	AuthCodeURL(state, verifier, nonce string) string
	// Exchange redeems a code and verifies the ID token and nonce; a refusal wraps ErrTokenRejected.
	Exchange(ctx context.Context, code, verifier, nonce string) (Tokens, error)
	// Refresh rotates the tokens; a refusal wraps ErrTokenRejected.
	Refresh(ctx context.Context, refreshToken string) (Tokens, error)
	Revoke(ctx context.Context, refreshToken string) error
	// VerifyAccessToken checks a bearer token: signature, issuer, audience and typ at+jwt.
	VerifyAccessToken(ctx context.Context, raw string) error
	EndSessionURL(idTokenHint string) string
}
