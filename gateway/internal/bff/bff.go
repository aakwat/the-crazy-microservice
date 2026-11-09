// Package bff is the gateway's browser half: OIDC login, sessions in Redis, and an access token per request.
package bff

import (
	"context"
	"encoding/json"
	"errors"
	"log/slog"
	"net/http"
	"time"

	"github.com/coreos/go-oidc/v3/oidc"
	"github.com/redis/go-redis/v9"
	"golang.org/x/oauth2"
)

// Config is what differs between deployments.
type Config struct {
	PublicURL    string // origin browsers use, e.g. http://127.0.0.1:8080
	Issuer       string // iss in every token, and where browsers are sent to log in
	AuthURL      string // where the gateway reaches auth over the private network
	ClientID     string
	ClientSecret string
	Audience     string // aud every access token must carry
	HTTPClient   *http.Client

	AfterLoginPath  string // where a fresh login lands on the gateway's origin, e.g. a frontend's "/"
	AfterLogoutPath string // where the browser ends up after logout; auth must have it registered
}

const (
	idleTTL     = 8 * time.Hour
	absoluteTTL = 12 * time.Hour
	pendingTTL  = 10 * time.Minute
	refreshSkew = 30 * time.Second
	// lockTTL outlasts main's timeout on calls to auth, so a slow refresh still holds the lock.
	lockTTL      = 15 * time.Second
	waitInterval = 100 * time.Millisecond
)

// BFF runs the browser login and turns a session cookie or bearer token into an access token.
type BFF struct {
	cfg      Config
	rdb      *redis.Client
	oauth    *oauth2.Config
	idTokens *oidc.IDTokenVerifier
	tokens   *oidc.IDTokenVerifier
	log      *slog.Logger
	now      func() time.Time
}

// New returns a BFF; auth's signing keys are fetched on first use and again on an unknown kid.
func New(ctx context.Context, cfg Config, rdb *redis.Client, log *slog.Logger) *BFF {
	keys := oidc.NewRemoteKeySet(oidc.ClientContext(ctx, cfg.HTTPClient), cfg.AuthURL+"/oauth2/jwks")
	algs := []string{oidc.ES256}
	return &BFF{
		cfg: cfg,
		rdb: rdb,
		oauth: &oauth2.Config{
			ClientID:     cfg.ClientID,
			ClientSecret: cfg.ClientSecret,
			RedirectURL:  cfg.PublicURL + "/bff/callback",
			Scopes:       []string{oidc.ScopeOpenID, "email"},
			Endpoint: oauth2.Endpoint{
				// The browser goes to the public issuer; the code exchange stays on the private network.
				AuthURL:   cfg.Issuer + "/oauth2/authorize",
				TokenURL:  cfg.AuthURL + "/oauth2/token",
				AuthStyle: oauth2.AuthStyleInHeader,
			},
		},
		idTokens: oidc.NewVerifier(cfg.Issuer, keys, &oidc.Config{ClientID: cfg.ClientID, SupportedSigningAlgs: algs}),
		tokens:   oidc.NewVerifier(cfg.Issuer, keys, &oidc.Config{SkipClientIDCheck: true, SupportedSigningAlgs: algs}),
		log:      log,
		now:      time.Now,
	}
}

// Problem writes an RFC 9457 body; detail must be safe to show a client, never an internal error.
func Problem(w http.ResponseWriter, status int, detail string) {
	body := map[string]any{"type": "about:blank", "title": http.StatusText(status), "status": status}
	if detail != "" {
		body["detail"] = detail
	}
	w.Header().Set("Content-Type", "application/problem+json")
	w.WriteHeader(status)
	_ = json.NewEncoder(w).Encode(body)
}

// fail logs the real error and shows the client only msg.
func (b *BFF) fail(w http.ResponseWriter, r *http.Request, status int, msg string, err error) {
	b.log.ErrorContext(r.Context(), msg, "err", err, "traceparent", r.Header.Get("Traceparent"))
	Problem(w, status, msg)
}

// client makes x/oauth2 call auth through the gateway's own HTTP client.
func (b *BFF) client(ctx context.Context) context.Context {
	return context.WithValue(ctx, oauth2.HTTPClient, b.cfg.HTTPClient)
}

// rejected reports auth refusing the code or refresh token itself; a bad client secret is an outage, not a logout.
func rejected(err error) bool {
	var re *oauth2.RetrieveError
	return errors.As(err, &re) && re.ErrorCode == "invalid_grant"
}
