package oidc

import (
	"context"
	"crypto/sha256"
	"encoding/base64"
	"net/http"
	"net/url"
	"testing"

	"github.com/aakwat/the-crazy-microservice/gateway/internal/testidp"
	"golang.org/x/oauth2"
)

func newProvider(t *testing.T) (*Provider, *testidp.IdP) {
	t.Helper()
	idp := testidp.New(t)
	p := New(context.Background(), Config{
		Issuer: idp.Issuer(), InternalURL: idp.Issuer(),
		ClientID: idp.ClientID, ClientSecret: idp.ClientSecret,
		RedirectURL: "http://gateway.test/bff/callback", PostLogoutURL: "http://gateway.test/",
		Audience: testidp.Audience, HTTPClient: http.DefaultClient,
	})
	return p, idp
}

func challengeOf(verifier string) string {
	sum := sha256.Sum256([]byte(verifier))
	return base64.RawURLEncoding.EncodeToString(sum[:])
}

func TestAuthCodeURLRequestsPKCEAndNonce(t *testing.T) {
	p, _ := newProvider(t)

	u, err := url.Parse(p.AuthCodeURL("st", oauth2.GenerateVerifier(), "n1"))
	if err != nil {
		t.Fatal(err)
	}
	q := u.Query()
	if q.Get("code_challenge_method") != "S256" || q.Get("nonce") != "n1" || q.Get("state") != "st" {
		t.Fatalf("missing PKCE, nonce or state: %s", u)
	}
}

func TestExchangeVerifiesPKCEAndNonce(t *testing.T) {
	p, idp := newProvider(t)
	ctx := context.Background()
	verifier := oauth2.GenerateVerifier()

	tokens, err := p.Exchange(ctx, idp.IssueCode(challengeOf(verifier), "n1", "acc-1"), verifier, "n1")
	if err != nil || tokens.Subject != "acc-1" || tokens.RefreshToken == "" {
		t.Fatalf("good exchange failed: %+v %v", tokens, err)
	}
	if _, err := p.Exchange(ctx, idp.IssueCode(challengeOf(verifier), "n1", "acc-1"), oauth2.GenerateVerifier(), "n1"); err == nil {
		t.Fatal("a wrong PKCE verifier must be rejected")
	}
	if _, err := p.Exchange(ctx, idp.IssueCode(challengeOf(verifier), "n1", "acc-1"), verifier, "other"); err == nil {
		t.Fatal("a nonce mismatch must be rejected")
	}
}

func TestRefreshRotatesAndTheOldTokenDies(t *testing.T) {
	p, idp := newProvider(t)
	ctx := context.Background()
	verifier := oauth2.GenerateVerifier()
	first, _ := p.Exchange(ctx, idp.IssueCode(challengeOf(verifier), "n", "acc-1"), verifier, "n")

	second, err := p.Refresh(ctx, first.RefreshToken)
	if err != nil || second.RefreshToken == first.RefreshToken {
		t.Fatalf("refresh must rotate: %+v %v", second, err)
	}
	if _, err := p.Refresh(ctx, first.RefreshToken); err == nil {
		t.Fatal("the rotated-away refresh token must not work again")
	}
}

func TestVerifyAccessTokenRequiresTypAndAudience(t *testing.T) {
	p, idp := newProvider(t)
	ctx := context.Background()

	if err := p.VerifyAccessToken(ctx, idp.AccessToken(t, "acc-1", "at+jwt", testidp.Audience)); err != nil {
		t.Fatalf("a valid access token was rejected: %v", err)
	}
	if err := p.VerifyAccessToken(ctx, idp.AccessToken(t, "acc-1", "JWT", testidp.Audience)); err == nil {
		t.Fatal("typ JWT (what an ID token carries) must be rejected")
	}
	if err := p.VerifyAccessToken(ctx, idp.AccessToken(t, "acc-1", "at+jwt", "someone-else")); err == nil {
		t.Fatal("a token for another audience must be rejected")
	}
}

func TestRevokeReachesTheProvider(t *testing.T) {
	p, idp := newProvider(t)

	if err := p.Revoke(context.Background(), "rt-1"); err != nil {
		t.Fatal(err)
	}
	if revoked := idp.RevokedTokens(); len(revoked) != 1 || revoked[0] != "rt-1" {
		t.Fatalf("revocation not received: %v", revoked)
	}
}
