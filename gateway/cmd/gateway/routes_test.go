package main

import (
	"context"
	"log/slog"
	"net/http"
	"net/http/httptest"
	"net/url"
	"sync"
	"testing"
	"time"

	"github.com/aakwat/the-crazy-microservice/gateway/internal/adapters/oidc"
	"github.com/aakwat/the-crazy-microservice/gateway/internal/adapters/redisstore"
	"github.com/aakwat/the-crazy-microservice/gateway/internal/app"
	"github.com/aakwat/the-crazy-microservice/gateway/internal/testidp"
	"github.com/aakwat/the-crazy-microservice/gateway/internal/transport/bff"
	"github.com/aakwat/the-crazy-microservice/gateway/internal/transport/edge"
	"github.com/aakwat/the-crazy-microservice/gateway/internal/transport/proxy"
	"github.com/alicebob/miniredis/v2"
	"github.com/redis/go-redis/v9"
)

func TestInternalAndUnknownPathsAre404(t *testing.T) {
	h, auth, _ := newRoutes(t)

	for _, req := range []*http.Request{
		httptest.NewRequest(http.MethodGet, "/internal/tickets/1", nil),
		httptest.NewRequest(http.MethodGet, "/actuator/env", nil),
		httptest.NewRequest(http.MethodDelete, "/login", nil),
	} {
		rec := serve(h, req)
		if rec.Code != http.StatusNotFound || rec.Header().Get("Content-Type") != "application/problem+json" {
			t.Errorf("%s %s: want a 404 problem, got %d", req.Method, req.URL.Path, rec.Code)
		}
	}
	if calls := auth.count(); calls != 0 {
		t.Fatalf("no 404 path may reach auth, got %d calls", calls)
	}
}

func TestAuthPagesAreProxiedWithoutGatewayCookies(t *testing.T) {
	h, auth, _ := newRoutes(t)
	req := httptest.NewRequest(http.MethodGet, "/login", nil)
	req.Header.Set("Cookie", "JSESSIONID=abc; __Host-gw-session=secret")

	if rec := serve(h, req); rec.Code != http.StatusOK {
		t.Fatalf("want 200, got %d", rec.Code)
	}
	got := auth.last()
	if got.path != "/login" || got.cookie != "JSESSIONID=abc" || got.traceparent == "" {
		t.Fatalf("want /login with only auth's cookie and a traceparent, got %+v", got)
	}
}

func TestRegisterIsRateLimitedPerClient(t *testing.T) {
	h, _, _ := newRoutes(t)
	register := func() int {
		return serve(h, httptest.NewRequest(http.MethodPost, "/auth/register", nil)).Code
	}

	register()
	register()
	if code := register(); code != http.StatusTooManyRequests {
		t.Fatalf("third registration inside the burst: want 429, got %d", code)
	}
}

func TestCrossSitePostIsRefused(t *testing.T) {
	h, auth, _ := newRoutes(t)
	post := func(site string) int {
		req := httptest.NewRequest(http.MethodPost, "/login", nil)
		req.Header.Set("Sec-Fetch-Site", site)
		return serve(h, req).Code
	}

	if code := post("cross-site"); code != http.StatusForbidden {
		t.Fatalf("cross-site POST: want 403, got %d", code)
	}
	if code := post("same-origin"); code != http.StatusOK || auth.count() != 1 {
		t.Fatalf("same-origin POST: want 200 reaching auth once, got %d", code)
	}
}

func TestUserEndpointSendsTheTokenToUserinfo(t *testing.T) {
	h, auth, idp := newRoutes(t)
	token := idp.AccessToken(t, "acc-1", "at+jwt", audience)
	req := httptest.NewRequest(http.MethodGet, "/bff/user", nil)
	req.Header.Set("Authorization", "Bearer "+token)

	if rec := serve(h, req); rec.Code != http.StatusOK {
		t.Fatalf("want 200, got %d", rec.Code)
	}
	if got := auth.last(); got.path != "/userinfo" || got.authorization != "Bearer "+token {
		t.Fatalf("want /userinfo with the bearer token, got %+v", got)
	}
}

func TestHealthz(t *testing.T) {
	h, _, _ := newRoutes(t)

	if rec := serve(h, httptest.NewRequest(http.MethodGet, "/healthz", nil)); rec.Code != http.StatusNoContent {
		t.Fatalf("want 204, got %d", rec.Code)
	}
}

func newRoutes(t *testing.T) (http.Handler, *authStub, *testidp.IdP) {
	t.Helper()
	discard := slog.New(slog.DiscardHandler)
	idp := testidp.New(t)
	stub := &authStub{}
	srv := httptest.NewServer(stub)
	t.Cleanup(srv.Close)
	authURL, err := url.Parse(srv.URL)
	if err != nil {
		t.Fatal(err)
	}

	store := redisstore.New(redis.NewClient(&redis.Options{Addr: miniredis.RunT(t).Addr()}))
	provider := oidc.New(context.Background(), oidc.Config{
		Issuer: idp.Issuer(), InternalURL: idp.Issuer(),
		ClientID: idp.ClientID, ClientSecret: idp.ClientSecret,
		RedirectURL: "http://gateway.test/bff/callback", PostLogoutURL: "http://gateway.test/bff/logged-out",
		Audience: audience, HTTPClient: http.DefaultClient,
	})
	svc := app.New(store, provider, lifetime, time.Now)
	h := routes(bff.New(svc, discard), proxy.New(authURL, &url.URL{Scheme: "http", Host: "gateway.test"}, time.Second, discard), edge.NewPerIP(1, 2))
	return h, stub, idp
}

func serve(h http.Handler, req *http.Request) *httptest.ResponseRecorder {
	rec := httptest.NewRecorder()
	h.ServeHTTP(rec, req)
	return rec
}

type seen struct {
	path, cookie, authorization, traceparent string
}

// authStub stands in for auth behind the proxy and records what reached it.
type authStub struct {
	mu    sync.Mutex
	calls []seen
}

func (a *authStub) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	a.mu.Lock()
	a.calls = append(a.calls, seen{r.URL.Path, r.Header.Get("Cookie"), r.Header.Get("Authorization"), r.Header.Get("Traceparent")})
	a.mu.Unlock()
	w.WriteHeader(http.StatusOK)
}

func (a *authStub) count() int {
	a.mu.Lock()
	defer a.mu.Unlock()
	return len(a.calls)
}

func (a *authStub) last() seen {
	a.mu.Lock()
	defer a.mu.Unlock()
	if len(a.calls) == 0 {
		return seen{}
	}
	return a.calls[len(a.calls)-1]
}
