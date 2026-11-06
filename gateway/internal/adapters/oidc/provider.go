// Package oidc implements domain.IdentityProvider against auth, with x/oauth2 and go-oidc.
package oidc

import (
	"context"
	"crypto/subtle"
	"errors"
	"fmt"
	"net/http"
	"net/url"
	"slices"
	"strings"

	"github.com/aakwat/the-crazy-microservice/gateway/internal/domain"
	gooidc "github.com/coreos/go-oidc/v3/oidc"
	"golang.org/x/oauth2"
)

// accessTokenType is RFC 9068's header value; it is what tells an access token from an ID token.
const accessTokenType = "at+jwt"

// Config separates the public issuer (in tokens and browser URLs) from where the gateway reaches auth.
type Config struct {
	Issuer        string
	InternalURL   string
	ClientID      string
	ClientSecret  string
	RedirectURL   string
	PostLogoutURL string
	Audience      string
	HTTPClient    *http.Client
}

// Provider talks to auth: browser URLs use the public issuer, back-channel calls the private URL.
type Provider struct {
	cfg          Config
	oauth        *oauth2.Config
	idTokens     *gooidc.IDTokenVerifier
	accessTokens *gooidc.IDTokenVerifier
}

var _ domain.IdentityProvider = (*Provider)(nil)

// New builds a provider whose JWKS is fetched lazily and refetched on an unknown kid.
func New(ctx context.Context, cfg Config) *Provider {
	keys := gooidc.NewRemoteKeySet(gooidc.ClientContext(ctx, cfg.HTTPClient), cfg.InternalURL+"/oauth2/jwks")
	algs := []string{gooidc.ES256}
	return &Provider{
		cfg: cfg,
		oauth: &oauth2.Config{
			ClientID:     cfg.ClientID,
			ClientSecret: cfg.ClientSecret,
			RedirectURL:  cfg.RedirectURL,
			Scopes:       []string{gooidc.ScopeOpenID, "email"},
			Endpoint: oauth2.Endpoint{
				AuthURL:   cfg.Issuer + "/oauth2/authorize",
				TokenURL:  cfg.InternalURL + "/oauth2/token",
				AuthStyle: oauth2.AuthStyleInHeader,
			},
		},
		idTokens: gooidc.NewVerifier(cfg.Issuer, keys,
			&gooidc.Config{ClientID: cfg.ClientID, SupportedSigningAlgs: algs}),
		accessTokens: gooidc.NewVerifier(cfg.Issuer, keys,
			&gooidc.Config{SkipClientIDCheck: true, SupportedSigningAlgs: algs}),
	}
}

// AuthCodeURL is where the browser starts a login: code flow with PKCE S256 and a nonce.
func (p *Provider) AuthCodeURL(state, verifier, nonce string) string {
	return p.oauth.AuthCodeURL(state, oauth2.S256ChallengeOption(verifier), gooidc.Nonce(nonce))
}

// Exchange redeems the code over the private network and verifies the ID token and its nonce.
func (p *Provider) Exchange(ctx context.Context, code, verifier, nonce string) (domain.Tokens, error) {
	tok, err := p.oauth.Exchange(p.client(ctx), code, oauth2.VerifierOption(verifier))
	if err != nil {
		return domain.Tokens{}, fmt.Errorf("code exchange: %w", err)
	}
	rawID, _ := tok.Extra("id_token").(string)
	idToken, err := p.idTokens.Verify(ctx, rawID)
	if err != nil {
		return domain.Tokens{}, fmt.Errorf("id token: %w", err)
	}
	if subtle.ConstantTimeCompare([]byte(idToken.Nonce), []byte(nonce)) != 1 {
		return domain.Tokens{}, errors.New("id token: nonce mismatch")
	}
	return domain.Tokens{
		Subject:      idToken.Subject,
		AccessToken:  tok.AccessToken,
		RefreshToken: tok.RefreshToken,
		IDToken:      rawID,
		AccessExpiry: tok.Expiry,
	}, nil
}

// Refresh trades a refresh token for new tokens; auth rotates, so the old refresh token is now dead.
func (p *Provider) Refresh(ctx context.Context, refreshToken string) (domain.Tokens, error) {
	tok, err := p.oauth.TokenSource(p.client(ctx), &oauth2.Token{RefreshToken: refreshToken}).Token()
	if err != nil {
		return domain.Tokens{}, fmt.Errorf("refresh: %w", err)
	}
	return domain.Tokens{
		AccessToken:  tok.AccessToken,
		RefreshToken: tok.RefreshToken,
		AccessExpiry: tok.Expiry,
	}, nil
}

// Revoke tells auth to invalidate a refresh token (RFC 7009); any non-200 answer is an error.
func (p *Provider) Revoke(ctx context.Context, refreshToken string) error {
	form := url.Values{"token": {refreshToken}, "token_type_hint": {"refresh_token"}}
	req, err := http.NewRequestWithContext(ctx, http.MethodPost, p.cfg.InternalURL+"/oauth2/revoke",
		strings.NewReader(form.Encode()))
	if err != nil {
		return err
	}
	req.Header.Set("Content-Type", "application/x-www-form-urlencoded")
	req.SetBasicAuth(p.cfg.ClientID, p.cfg.ClientSecret)
	res, err := p.cfg.HTTPClient.Do(req)
	if err != nil {
		return fmt.Errorf("revoke: %w", err)
	}
	defer res.Body.Close()
	if res.StatusCode != http.StatusOK {
		return fmt.Errorf("revoke: auth answered %d", res.StatusCode)
	}
	return nil
}

// VerifyAccessToken accepts only RFC 9068 access tokens for our audience; go-oidc does not check typ.
func (p *Provider) VerifyAccessToken(ctx context.Context, raw string) error {
	if typ, err := headerType(raw); err != nil || !strings.EqualFold(typ, accessTokenType) {
		return errors.New("access token: typ must be at+jwt")
	}
	token, err := p.accessTokens.Verify(ctx, raw)
	if err != nil {
		return fmt.Errorf("access token: %w", err)
	}
	if !slices.Contains(token.Audience, p.cfg.Audience) {
		return errors.New("access token: wrong audience")
	}
	return nil
}

// EndSessionURL ends auth's own sign-on session too, so the next login asks for the password again.
func (p *Provider) EndSessionURL(idTokenHint string) string {
	return p.cfg.Issuer + "/connect/logout?" + url.Values{
		"id_token_hint":            {idTokenHint},
		"post_logout_redirect_uri": {p.cfg.PostLogoutURL},
	}.Encode()
}

func (p *Provider) client(ctx context.Context) context.Context {
	return context.WithValue(ctx, oauth2.HTTPClient, p.cfg.HTTPClient)
}
