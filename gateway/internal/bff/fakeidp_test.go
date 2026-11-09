package bff

import (
	"crypto/ecdsa"
	"crypto/elliptic"
	"crypto/rand"
	"crypto/sha256"
	"encoding/base64"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"slices"
	"sync"
	"testing"
	"time"

	"github.com/go-jose/go-jose/v4"
	"github.com/go-jose/go-jose/v4/jwt"
)

const (
	fakeClientID     = "gateway"
	fakeClientSecret = "gateway-secret"
	fakeAudience     = "crazy-ms"
)

// fakeIDP is an in-process authorization server with real ES256 tokens, a real PKCE check and rotation.
type fakeIDP struct {
	server *httptest.Server
	key    *ecdsa.PrivateKey

	mu           sync.Mutex
	clientSecret string
	accessTTL    time.Duration
	codes        map[string]grant
	refresh      map[string]string
	revoked      []string
}

type grant struct {
	challenge string
	nonce     string
	subject   string
}

func newFakeIDP(t *testing.T) *fakeIDP {
	t.Helper()
	key, err := ecdsa.GenerateKey(elliptic.P256(), rand.Reader)
	if err != nil {
		t.Fatal(err)
	}
	p := &fakeIDP{key: key, clientSecret: fakeClientSecret, accessTTL: 15 * time.Minute, codes: map[string]grant{}, refresh: map[string]string{}}
	mux := http.NewServeMux()
	mux.HandleFunc("GET /oauth2/jwks", p.jwks)
	mux.HandleFunc("POST /oauth2/token", p.token)
	mux.HandleFunc("POST /oauth2/revoke", p.revoke)
	p.server = httptest.NewServer(mux)
	t.Cleanup(p.server.Close)
	return p
}

func (p *fakeIDP) issuer() string {
	return p.server.URL
}

// issueCode stands in for the login page: it returns a code bound to a PKCE challenge and nonce.
func (p *fakeIDP) issueCode(challenge, nonce, subject string) string {
	p.mu.Lock()
	defer p.mu.Unlock()
	code := rand.Text()
	p.codes[code] = grant{challenge: challenge, nonce: nonce, subject: subject}
	return code
}

// accessToken signs a token with the given typ and audience, for testing bearer checks.
func (p *fakeIDP) accessToken(t *testing.T, typ, audience string) string {
	t.Helper()
	p.mu.Lock()
	ttl := p.accessTTL
	p.mu.Unlock()
	raw, err := p.sign(typ, map[string]any{"sub": "acc-1", "aud": []string{audience}}, ttl)
	if err != nil {
		t.Fatal(err)
	}
	return raw
}

func (p *fakeIDP) setAccessTTL(d time.Duration) {
	p.mu.Lock()
	defer p.mu.Unlock()
	p.accessTTL = d
}

// rotateClientSecret makes auth reject the gateway's client credentials from now on.
func (p *fakeIDP) rotateClientSecret() {
	p.mu.Lock()
	defer p.mu.Unlock()
	p.clientSecret = "rotated"
}

func (p *fakeIDP) revokedTokens() []string {
	p.mu.Lock()
	defer p.mu.Unlock()
	return slices.Clone(p.revoked)
}

func (p *fakeIDP) jwks(w http.ResponseWriter, _ *http.Request) {
	set := jose.JSONWebKeySet{Keys: []jose.JSONWebKey{
		{Key: &p.key.PublicKey, KeyID: "test", Algorithm: string(jose.ES256), Use: "sig"},
	}}
	_ = json.NewEncoder(w).Encode(set)
}

func (p *fakeIDP) token(w http.ResponseWriter, r *http.Request) {
	p.mu.Lock()
	defer p.mu.Unlock()
	if id, secret, ok := r.BasicAuth(); !ok || id != fakeClientID || secret != p.clientSecret {
		oauthError(w, http.StatusUnauthorized, "invalid_client")
		return
	}

	switch r.FormValue("grant_type") {
	case "authorization_code":
		g, ok := p.codes[r.FormValue("code")]
		delete(p.codes, r.FormValue("code"))
		if !ok || challengeOf(r.FormValue("code_verifier")) != g.challenge {
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

func (p *fakeIDP) revoke(_ http.ResponseWriter, r *http.Request) {
	p.mu.Lock()
	defer p.mu.Unlock()
	p.revoked = append(p.revoked, r.FormValue("token"))
	delete(p.refresh, r.FormValue("token"))
}

// issue must run with p.mu held; it rotates the refresh token on every call.
func (p *fakeIDP) issue(w http.ResponseWriter, subject, nonce string) {
	access, err := p.sign("at+jwt", map[string]any{"sub": subject, "aud": []string{fakeAudience}}, p.accessTTL)
	if err != nil {
		oauthError(w, http.StatusInternalServerError, "server_error")
		return
	}
	refresh := rand.Text()
	p.refresh[refresh] = subject
	body := map[string]any{"access_token": access, "token_type": "Bearer",
		"expires_in": int(p.accessTTL.Seconds()), "refresh_token": refresh}
	if nonce != "" {
		claims := map[string]any{"sub": subject, "aud": []string{fakeClientID}, "nonce": nonce}
		if body["id_token"], err = p.sign("JWT", claims, time.Hour); err != nil {
			oauthError(w, http.StatusInternalServerError, "server_error")
			return
		}
	}
	w.Header().Set("Content-Type", "application/json")
	_ = json.NewEncoder(w).Encode(body)
}

func (p *fakeIDP) sign(typ string, claims map[string]any, ttl time.Duration) (string, error) {
	opts := (&jose.SignerOptions{}).WithType(jose.ContentType(typ)).WithHeader("kid", "test")
	signer, err := jose.NewSigner(jose.SigningKey{Algorithm: jose.ES256, Key: p.key}, opts)
	if err != nil {
		return "", err
	}
	now := time.Now()
	claims["iss"], claims["iat"], claims["exp"] = p.issuer(), now.Unix(), now.Add(ttl).Unix()
	return jwt.Signed(signer).Claims(claims).Serialize()
}

func challengeOf(verifier string) string {
	sum := sha256.Sum256([]byte(verifier))
	return base64.RawURLEncoding.EncodeToString(sum[:])
}

func oauthError(w http.ResponseWriter, status int, code string) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	_ = json.NewEncoder(w).Encode(map[string]string{"error": code})
}
