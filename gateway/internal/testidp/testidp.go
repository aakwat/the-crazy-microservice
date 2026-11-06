// Package testidp is a fake authorization server for tests: real ES256 tokens and a real PKCE check.
package testidp

import (
	"crypto/ecdsa"
	"crypto/elliptic"
	"crypto/rand"
	"crypto/sha256"
	"encoding/base64"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"sync"
	"testing"
	"time"

	"github.com/go-jose/go-jose/v4"
	"github.com/go-jose/go-jose/v4/jwt"
)

// Audience is what access tokens are issued for, matching auth's configuration.
const Audience = "crazy-ms"

// IdP is the fake server; tests read Revoked and set AccessTTL.
type IdP struct {
	Server       *httptest.Server
	ClientID     string
	ClientSecret string
	AccessTTL    time.Duration

	key     *ecdsa.PrivateKey
	mu      sync.Mutex
	codes   map[string]grant
	refresh map[string]string
	Revoked []string
}

type grant struct {
	challenge string
	nonce     string
	subject   string
}

// New starts a fake server that is closed when the test ends.
func New(t *testing.T) *IdP {
	t.Helper()
	key, err := ecdsa.GenerateKey(elliptic.P256(), rand.Reader)
	if err != nil {
		t.Fatal(err)
	}
	p := &IdP{ClientID: "gateway", ClientSecret: "gateway-secret", AccessTTL: 15 * time.Minute,
		key: key, codes: map[string]grant{}, refresh: map[string]string{}}
	mux := http.NewServeMux()
	mux.HandleFunc("GET /oauth2/jwks", p.jwks)
	mux.HandleFunc("POST /oauth2/token", p.token)
	mux.HandleFunc("POST /oauth2/revoke", p.revoke)
	p.Server = httptest.NewServer(mux)
	t.Cleanup(p.Server.Close)
	return p
}

// Issuer is the iss every token carries; tests use it as both public and internal URL.
func (p *IdP) Issuer() string {
	return p.Server.URL
}

// IssueCode stands in for the browser login: it returns a code bound to a PKCE challenge and nonce.
func (p *IdP) IssueCode(challenge, nonce, subject string) string {
	p.mu.Lock()
	defer p.mu.Unlock()
	code := rand.Text()
	p.codes[code] = grant{challenge: challenge, nonce: nonce, subject: subject}
	return code
}

// AccessToken signs a token with the given typ and audience, for testing bearer verification.
func (p *IdP) AccessToken(t *testing.T, subject, typ, audience string) string {
	t.Helper()
	raw, err := p.sign(typ, map[string]any{"sub": subject, "aud": []string{audience}}, p.AccessTTL)
	if err != nil {
		t.Fatal(err)
	}
	return raw
}

func (p *IdP) jwks(w http.ResponseWriter, _ *http.Request) {
	set := jose.JSONWebKeySet{Keys: []jose.JSONWebKey{
		{Key: &p.key.PublicKey, KeyID: "test", Algorithm: string(jose.ES256), Use: "sig"}}}
	_ = json.NewEncoder(w).Encode(set)
}

func (p *IdP) token(w http.ResponseWriter, r *http.Request) {
	if id, secret, ok := r.BasicAuth(); !ok || id != p.ClientID || secret != p.ClientSecret {
		oauthError(w, http.StatusUnauthorized, "invalid_client")
		return
	}
	p.mu.Lock()
	defer p.mu.Unlock()

	switch r.FormValue("grant_type") {
	case "authorization_code":
		g, ok := p.codes[r.FormValue("code")]
		delete(p.codes, r.FormValue("code"))
		if !ok || challenge(r.FormValue("code_verifier")) != g.challenge {
			oauthError(w, http.StatusBadRequest, "invalid_grant")
			return
		}
		p.issue(w, g.subject, g.nonce)
	case "refresh_token":
		subject, ok := p.refresh[r.FormValue("refresh_token")]
		delete(p.refresh, r.FormValue("refresh_token"))
		if !ok {
			oauthError(w, http.StatusBadRequest, "invalid_grant")
			return
		}
		p.issue(w, subject, "")
	default:
		oauthError(w, http.StatusBadRequest, "unsupported_grant_type")
	}
}

func (p *IdP) revoke(w http.ResponseWriter, r *http.Request) {
	p.mu.Lock()
	defer p.mu.Unlock()
	p.Revoked = append(p.Revoked, r.FormValue("token"))
	delete(p.refresh, r.FormValue("token"))
}

// issue must be called with p.mu held; it rotates the refresh token on every call.
func (p *IdP) issue(w http.ResponseWriter, subject, nonce string) {
	access, err := p.sign("at+jwt", map[string]any{"sub": subject, "aud": []string{Audience}}, p.AccessTTL)
	if err != nil {
		oauthError(w, http.StatusInternalServerError, "server_error")
		return
	}
	body := map[string]any{"access_token": access, "token_type": "Bearer",
		"expires_in": int(p.AccessTTL.Seconds()), "refresh_token": rand.Text()}
	p.refresh[body["refresh_token"].(string)] = subject
	if nonce != "" {
		id, err := p.sign("JWT", map[string]any{"sub": subject, "aud": []string{p.ClientID}, "nonce": nonce}, time.Hour)
		if err != nil {
			oauthError(w, http.StatusInternalServerError, "server_error")
			return
		}
		body["id_token"] = id
	}
	w.Header().Set("Content-Type", "application/json")
	_ = json.NewEncoder(w).Encode(body)
}

func (p *IdP) sign(typ string, claims map[string]any, ttl time.Duration) (string, error) {
	opts := (&jose.SignerOptions{}).WithType(jose.ContentType(typ)).WithHeader("kid", "test")
	signer, err := jose.NewSigner(jose.SigningKey{Algorithm: jose.ES256, Key: p.key}, opts)
	if err != nil {
		return "", err
	}
	now := time.Now()
	claims["iss"] = p.Issuer()
	claims["iat"] = now.Unix()
	claims["exp"] = now.Add(ttl).Unix()
	return jwt.Signed(signer).Claims(claims).Serialize()
}

func challenge(verifier string) string {
	sum := sha256.Sum256([]byte(verifier))
	return base64.RawURLEncoding.EncodeToString(sum[:])
}

func oauthError(w http.ResponseWriter, status int, code string) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	_ = json.NewEncoder(w).Encode(map[string]string{"error": code})
}
