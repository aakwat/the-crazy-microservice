package main

import (
	"context"
	"log/slog"
	"net/http"
	"net/http/httptest"
	"net/url"
	"strings"
	"sync"
	"testing"
	"time"

	"github.com/aakwat/the-crazy-microservice/gateway/internal/bff"
	"github.com/alicebob/miniredis/v2"
	"github.com/redis/go-redis/v9"
)

func TestUnlistedPathsAre404AndNeverReachAuth(t *testing.T) {
	h, auth := newTestRoutes(t)
	for _, tc := range []struct{ method, path string }{
		{http.MethodGet, "/internal/tickets/1"},
		{http.MethodGet, "/actuator/env"},
		{http.MethodDelete, "/login"},
		{http.MethodPost, "/oauth2/token"},
		{http.MethodPost, "/oauth2/revoke"},
	} {
		rec := serve(h, httptest.NewRequest(tc.method, tc.path, nil))
		if rec.Code != http.StatusNotFound || rec.Header().Get("Content-Type") != "application/problem+json" {
			t.Errorf("%s %s = %d %q, want a 404 problem", tc.method, tc.path, rec.Code, rec.Header().Get("Content-Type"))
		}
	}
	if got := len(auth.all()); got != 0 {
		t.Errorf("auth received %d requests, want 0", got)
	}
}

func TestAuthPagesGetNoGatewayCredentialsAndTrustedForwarding(t *testing.T) {
	h, auth := newTestRoutes(t)
	forged := "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01"
	req := httptest.NewRequest(http.MethodGet, "http://evil.test/login", nil)
	req.Header.Set("Cookie", "JSESSIONID=abc; __Host-gw-session=secret")
	req.Header.Set("Authorization", "Bearer for-the-gateway")
	req.Header.Set("X-Forwarded-For", "198.51.100.66")
	req.Header.Set("X-Forwarded-Host", "evil.test")
	req.Header.Set("Traceparent", forged)

	rec := serve(h, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("GET /login = %d, want %d", rec.Code, http.StatusOK)
	}
	got := auth.last()
	want := seen{path: "/login", cookie: "JSESSIONID=abc", forwardedFor: "192.0.2.1",
		forwardedHost: "gateway.test", forwardedProto: "http"}
	if got.traceparent == "" || got.traceparent == forged {
		t.Errorf("auth got traceparent %q, want a fresh one", got.traceparent)
	}
	got.traceparent = ""
	if got != want {
		t.Errorf("auth got %+v, want %+v", got, want)
	}
	if frame := rec.Result().Header.Values("X-Frame-Options"); len(frame) != 1 || frame[0] != "DENY" {
		t.Errorf("X-Frame-Options = %q, want exactly [DENY]", frame)
	}
}

func TestCrossSitePostIsRefused(t *testing.T) {
	h, _ := newTestRoutes(t)
	for _, tc := range []struct {
		site string
		want int
	}{
		{"cross-site", http.StatusForbidden},
		{"same-origin", http.StatusOK},
	} {
		req := httptest.NewRequest(http.MethodPost, "/login", nil)
		req.Header.Set("Sec-Fetch-Site", tc.site)
		if got := serve(h, req).Code; got != tc.want {
			t.Errorf("POST /login with Sec-Fetch-Site %q = %d, want %d", tc.site, got, tc.want)
		}
	}
}

func TestAnonymousEndpointsAreRateLimited(t *testing.T) {
	h, _ := newTestRoutes(t)
	for _, tc := range []struct {
		method, path string
		perMin       int
		retryAfter   string
	}{
		{http.MethodPost, "/auth/register", registerPerMin, "12"},
		{http.MethodGet, "/bff/login", loginPerMin, "3"},
		{http.MethodPost, "/login", loginPerMin, "3"},
	} {
		for range tc.perMin {
			serve(h, httptest.NewRequest(tc.method, tc.path, nil))
		}
		rec := serve(h, httptest.NewRequest(tc.method, tc.path, nil))
		if rec.Code != http.StatusTooManyRequests || rec.Header().Get("Retry-After") != tc.retryAfter {
			t.Errorf("%s %s past %d a minute = %d, Retry-After %q; want 429, %q",
				tc.method, tc.path, tc.perMin, rec.Code, rec.Header().Get("Retry-After"), tc.retryAfter)
		}
	}
}

func TestOversizeBodyIsA413Problem(t *testing.T) {
	h, auth := newTestRoutes(t)
	req := httptest.NewRequest(http.MethodPost, "/auth/register", strings.NewReader(strings.Repeat("x", maxBodyBytes+1)))

	if got := serve(h, req).Code; got != http.StatusRequestEntityTooLarge {
		t.Errorf("POST /auth/register with %d bytes = %d, want %d", maxBodyBytes+1, got, http.StatusRequestEntityTooLarge)
	}
	if got := len(auth.all()); got != 0 {
		t.Errorf("auth received %d requests, want 0", got)
	}
}

func TestAuthDownIsA502Problem(t *testing.T) {
	gone := httptest.NewServer(http.NotFoundHandler())
	target, _ := url.Parse(gone.URL)
	gone.Close()
	proxy := newProxy(target, &url.URL{Scheme: "http", Host: "gateway.test"}, slog.New(slog.DiscardHandler))

	rec := serve(proxy, httptest.NewRequest(http.MethodGet, "/login", nil))

	if rec.Code != http.StatusBadGateway || rec.Header().Get("Content-Type") != "application/problem+json" {
		t.Errorf("GET /login with auth down = %d %q, want a 502 problem", rec.Code, rec.Header().Get("Content-Type"))
	}
}

func TestLimiterGroupsIPv6ByPrefix(t *testing.T) {
	l := newLimiter(1, maxClients)
	now := time.Now()

	l.allow(clientKey(&http.Request{RemoteAddr: "[2001:db8:1:2::1]:1000"}), now)

	if l.allow(clientKey(&http.Request{RemoteAddr: "[2001:db8:1:2::ffff]:1000"}), now) {
		t.Error("a second address in the same /64 got its own bucket, want a shared one")
	}
}

func TestLimiterRefusesNewcomersWhileFullOfActiveClients(t *testing.T) {
	l := newLimiter(1, 1)
	now := time.Now()

	if !l.allow("203.0.113.1", now) {
		t.Fatal("allow(first client) = false, want true")
	}
	if l.allow("203.0.113.2", now) {
		t.Error("allow(newcomer) with a full table of active buckets = true, want false")
	}
	if !l.allow("203.0.113.2", now.Add(time.Minute)) {
		t.Error("allow(newcomer) after the active bucket refilled = false, want true")
	}
}

func newTestRoutes(t *testing.T) (http.Handler, *authStub) {
	t.Helper()
	auth := &authStub{}
	srv := httptest.NewServer(auth)
	t.Cleanup(srv.Close)
	target, err := url.Parse(srv.URL)
	if err != nil {
		t.Fatal(err)
	}
	discard := slog.New(slog.DiscardHandler)
	rdb := redis.NewClient(&redis.Options{Addr: miniredis.RunT(t).Addr()})
	b := bff.New(context.Background(), bff.Config{
		PublicURL:       "http://gateway.test",
		Issuer:          "http://gateway.test",
		AuthURL:         srv.URL,
		ClientID:        "gateway",
		ClientSecret:    "secret",
		Audience:        audience,
		HTTPClient:      http.DefaultClient,
		AfterLoginPath:  "/bff/user",
		AfterLogoutPath: "/bff/logged-out",
	}, rdb, discard)
	return routes(b, newProxy(target, &url.URL{Scheme: "http", Host: "gateway.test"}, discard)), auth
}

func serve(h http.Handler, req *http.Request) *httptest.ResponseRecorder {
	rec := httptest.NewRecorder()
	h.ServeHTTP(rec, req)
	return rec
}

type seen struct {
	path, cookie, authorization, traceparent    string
	forwardedFor, forwardedHost, forwardedProto string
}

// authStub stands in for auth behind the proxy, records what reached it and sets its own frame policy.
type authStub struct {
	mu    sync.Mutex
	calls []seen
}

func (a *authStub) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	a.mu.Lock()
	a.calls = append(a.calls, seen{
		path:           r.URL.Path,
		cookie:         r.Header.Get("Cookie"),
		authorization:  r.Header.Get("Authorization"),
		traceparent:    r.Header.Get("Traceparent"),
		forwardedFor:   r.Header.Get("X-Forwarded-For"),
		forwardedHost:  r.Header.Get("X-Forwarded-Host"),
		forwardedProto: r.Header.Get("X-Forwarded-Proto"),
	})
	a.mu.Unlock()
	w.Header().Set("X-Frame-Options", "SAMEORIGIN")
	w.WriteHeader(http.StatusOK)
}

func (a *authStub) all() []seen {
	a.mu.Lock()
	defer a.mu.Unlock()
	return append([]seen(nil), a.calls...)
}

func (a *authStub) last() seen {
	calls := a.all()
	if len(calls) == 0 {
		return seen{}
	}
	return calls[len(calls)-1]
}
