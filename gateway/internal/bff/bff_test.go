package bff

import (
	"context"
	"log/slog"
	"net/http"
	"net/http/cookiejar"
	"net/http/httptest"
	"net/url"
	"strings"
	"sync"
	"testing"
	"time"

	"github.com/alicebob/miniredis/v2"
	"github.com/redis/go-redis/v9"
)

func TestBrowserLoginUserLogout(t *testing.T) {
	g := newTestGateway(t)
	br := g.newBrowser(t)

	if res := g.login(t, br); res.StatusCode != http.StatusSeeOther || res.Header.Get("Location") != "/bff/user" {
		t.Fatalf("callback = %d %q, want 303 to /bff/user", res.StatusCode, res.Header.Get("Location"))
	}
	if got := br.do(t, http.MethodGet, "/bff/user", nil).StatusCode; got != http.StatusOK {
		t.Fatalf("GET /bff/user = %d, want %d", got, http.StatusOK)
	}
	if got := g.up.last(); !strings.HasPrefix(got.auth, "Bearer ey") || got.cookie != "" {
		t.Errorf("upstream got Authorization %q and Cookie %q, want a bearer token and no cookies", got.auth, got.cookie)
	}

	if got := br.do(t, http.MethodGet, "/bff/logout", nil).StatusCode; got != http.StatusOK {
		t.Errorf("GET /bff/logout = %d, want %d (the confirmation form)", got, http.StatusOK)
	}
	if got := br.do(t, http.MethodGet, "/bff/user", nil).StatusCode; got != http.StatusOK {
		t.Fatalf("GET /bff/user after GET /bff/logout = %d, want %d: a GET must not log out", got, http.StatusOK)
	}

	res := br.do(t, http.MethodPost, "/bff/logout", nil)
	wantPrefix := g.idp.issuer() + "/connect/logout?id_token_hint="
	if res.StatusCode != http.StatusSeeOther || !strings.HasPrefix(res.Header.Get("Location"), wantPrefix) {
		t.Errorf("POST /bff/logout = %d %q, want 303 to %s...", res.StatusCode, res.Header.Get("Location"), wantPrefix)
	}
	if got := len(g.idp.revokedTokens()); got != 1 {
		t.Errorf("revoked %d refresh tokens, want 1", got)
	}
	if got := br.do(t, http.MethodGet, "/bff/user", nil).StatusCode; got != http.StatusUnauthorized {
		t.Errorf("GET /bff/user after logout = %d, want %d", got, http.StatusUnauthorized)
	}
}

func TestLoginCookieIsHostOnlySecureAndHidden(t *testing.T) {
	g := newTestGateway(t)

	cookies := g.newBrowser(t).do(t, http.MethodGet, "/bff/login", nil).Cookies()

	if len(cookies) != 1 {
		t.Fatalf("GET /bff/login set %d cookies, want 1", len(cookies))
	}
	c := cookies[0]
	if c.Name != loginCookie || !c.Secure || !c.HttpOnly || c.SameSite != http.SameSiteLaxMode || c.Path != "/" || c.Domain != "" {
		t.Errorf("login cookie = %+v, want __Host- with Secure, HttpOnly, SameSite=Lax, Path=/ and no Domain", c)
	}
}

func TestCallbackWorksOnceAndOnlyWithItsState(t *testing.T) {
	g := newTestGateway(t)
	br := g.newBrowser(t)
	withLogin := func(id string) http.Header { return http.Header{"Cookie": {loginCookie + "=" + id}} }

	p := g.startLogin(t, br)
	if got := br.do(t, http.MethodGet, callbackPath(p.state, p.code), nil).StatusCode; got != http.StatusSeeOther {
		t.Fatalf("first callback = %d, want %d", got, http.StatusSeeOther)
	}
	if got := g.newBrowser(t).do(t, http.MethodGet, callbackPath(p.state, p.code), withLogin(p.id)).StatusCode; got != http.StatusBadRequest {
		t.Errorf("replayed callback = %d, want %d", got, http.StatusBadRequest)
	}

	p = g.startLogin(t, br)
	if got := g.newBrowser(t).do(t, http.MethodGet, "/bff/callback?error=access_denied", withLogin(p.id)).StatusCode; got != http.StatusBadRequest {
		t.Errorf("callback with ?error= = %d, want %d", got, http.StatusBadRequest)
	}
	if got := br.do(t, http.MethodGet, callbackPath("guessed", p.code), nil).StatusCode; got != http.StatusBadRequest {
		t.Errorf("callback with a wrong state = %d, want %d", got, http.StatusBadRequest)
	}
	if got := g.newBrowser(t).do(t, http.MethodGet, callbackPath(p.state, p.code), withLogin(p.id)).StatusCode; got != http.StatusBadRequest {
		t.Errorf("callback after a wrong state = %d, want %d: one guess must burn the login", got, http.StatusBadRequest)
	}
}

func TestConcurrentRequestsShareOneRefresh(t *testing.T) {
	g := newTestGateway(t)
	br := g.newBrowser(t)
	g.login(t, br)
	br.do(t, http.MethodGet, "/bff/user", nil)
	before := g.up.last().auth

	g.idp.setAccessTTL(time.Hour)
	g.clock.advance(14*time.Minute + 45*time.Second)

	statuses := make([]int, 5)
	var wg sync.WaitGroup
	for i := range statuses {
		wg.Go(func() { statuses[i] = br.status("/bff/user") })
	}
	wg.Wait()

	// With rotation a second refresh would fail and end the session, so five successes prove one refresh.
	calls := g.up.all()
	after := calls[len(calls)-1].auth
	for i, status := range statuses {
		if status != http.StatusOK {
			t.Errorf("request %d = %d, want %d", i, status, http.StatusOK)
		}
	}
	for _, c := range calls[len(calls)-len(statuses):] {
		if c.auth == before || c.auth != after {
			t.Fatalf("upstream tokens differ or did not change: every request must carry the one refreshed token")
		}
	}
}

func TestRejectedRefreshEndsTheSession(t *testing.T) {
	g := newTestGateway(t)
	br := g.newBrowser(t)
	g.login(t, br)
	ctx, id := context.Background(), br.cookie(sessionCookie)
	s, _, _ := g.b.loadSession(ctx, id)
	s.RefreshToken = "revoked-elsewhere"
	if err := g.b.createSession(ctx, id, s); err != nil {
		t.Fatal(err)
	}
	g.clock.advance(15 * time.Minute)

	if got := br.do(t, http.MethodGet, "/bff/user", nil).StatusCode; got != http.StatusUnauthorized {
		t.Errorf("GET /bff/user = %d, want %d", got, http.StatusUnauthorized)
	}
	if _, ok, _ := g.b.loadSession(ctx, id); ok {
		t.Error("session still stored after auth rejected its refresh token")
	}
}

func TestAuthOutageKeepsTheSession(t *testing.T) {
	g := newTestGateway(t)
	br := g.newBrowser(t)
	g.login(t, br)
	g.clock.advance(15 * time.Minute)
	g.idp.server.Close()

	if got := br.do(t, http.MethodGet, "/bff/user", nil).StatusCode; got != http.StatusServiceUnavailable {
		t.Errorf("GET /bff/user while auth is down = %d, want %d", got, http.StatusServiceUnavailable)
	}
	if _, ok, _ := g.b.loadSession(context.Background(), br.cookie(sessionCookie)); !ok {
		t.Error("session deleted during an auth outage; it must survive for the next request")
	}
}

func TestWrongClientSecretIsAnOutageNotALogout(t *testing.T) {
	g := newTestGateway(t)
	br := g.newBrowser(t)
	g.login(t, br)
	g.idp.rotateClientSecret()
	g.clock.advance(15 * time.Minute)

	if got := br.do(t, http.MethodGet, "/bff/user", nil).StatusCode; got != http.StatusServiceUnavailable {
		t.Errorf("GET /bff/user with a rejected client secret = %d, want %d", got, http.StatusServiceUnavailable)
	}
	if _, ok, _ := g.b.loadSession(context.Background(), br.cookie(sessionCookie)); !ok {
		t.Error("session deleted over invalid_client; only invalid_grant may end a session")
	}
}

func TestSessionEndsAtTheAbsoluteCap(t *testing.T) {
	g := newTestGateway(t)
	br := g.newBrowser(t)
	g.login(t, br)

	g.clock.advance(absoluteTTL)

	if got := br.do(t, http.MethodGet, "/bff/user", nil).StatusCode; got != http.StatusUnauthorized {
		t.Errorf("GET /bff/user after %v = %d, want %d", absoluteTTL, got, http.StatusUnauthorized)
	}
}

func TestBearerMustBeAnAccessTokenForUs(t *testing.T) {
	g := newTestGateway(t)
	for _, tc := range []struct {
		name, typ, audience string
		want                int
	}{
		{"access token", "at+jwt", fakeAudience, http.StatusOK},
		{"ID token shape", "JWT", fakeAudience, http.StatusUnauthorized},
		{"other audience", "at+jwt", "someone-else", http.StatusUnauthorized},
	} {
		t.Run(tc.name, func(t *testing.T) {
			token := g.idp.accessToken(t, tc.typ, tc.audience)
			res := g.newBrowser(t).do(t, http.MethodGet, "/bff/user", http.Header{"Authorization": {"Bearer " + token}})
			if res.StatusCode != tc.want {
				t.Errorf("GET /bff/user with typ %q, aud %q = %d, want %d", tc.typ, tc.audience, res.StatusCode, tc.want)
			}
		})
	}
}

func TestReplaceSessionNeverRevivesADeletedSession(t *testing.T) {
	g := newTestGateway(t)
	ctx := context.Background()
	s := session{AccessToken: "old", LoginAt: g.clock.now()}
	if err := g.b.createSession(ctx, "s1", s); err != nil {
		t.Fatal(err)
	}
	if err := g.b.rdb.Del(ctx, sessionKey+"s1").Err(); err != nil {
		t.Fatal(err)
	}

	if live, err := g.b.replaceSession(ctx, "s1", s); live || err != nil {
		t.Errorf("replaceSession() on a deleted session = %v, %v; want false, nil", live, err)
	}
	if _, ok, _ := g.b.loadSession(ctx, "s1"); ok {
		t.Error("replaceSession() recreated a deleted session")
	}
}

func TestStripCredentialsKeepsOnlyTheUpstreamsCookies(t *testing.T) {
	up := &upstream{}
	req := httptest.NewRequest(http.MethodGet, "/login", nil)
	req.Header.Set("Cookie", "JSESSIONID=abc; "+sessionCookie+"=secret; "+loginCookie+"=pending")
	req.Header.Set("Authorization", "Bearer for-the-gateway")

	StripCredentials(up).ServeHTTP(httptest.NewRecorder(), req)

	if got := up.last(); got.cookie != "JSESSIONID=abc" || got.auth != "" {
		t.Errorf("upstream got Cookie %q and Authorization %q, want only JSESSIONID=abc and no Authorization", got.cookie, got.auth)
	}
}

// testGateway serves the BFF routes over TLS, because browsers only send Secure cookies over https.
type testGateway struct {
	b     *BFF
	idp   *fakeIDP
	srv   *httptest.Server
	up    *upstream
	clock *testClock
}

func newTestGateway(t *testing.T) *testGateway {
	t.Helper()
	idp := newFakeIDP(t)
	rdb := redis.NewClient(&redis.Options{Addr: miniredis.RunT(t).Addr()})
	b := New(context.Background(), Config{
		PublicURL:       "https://gateway.test",
		Issuer:          idp.issuer(),
		AuthURL:         idp.issuer(),
		ClientID:        fakeClientID,
		ClientSecret:    fakeClientSecret,
		Audience:        fakeAudience,
		HTTPClient:      http.DefaultClient,
		AfterLoginPath:  "/bff/user",
		AfterLogoutPath: "/bff/logged-out",
	}, rdb, slog.New(slog.DiscardHandler))
	clock := &testClock{t: time.Now()}
	b.now = clock.now

	up := &upstream{}
	mux := http.NewServeMux()
	mux.HandleFunc("GET /bff/login", b.Login)
	mux.HandleFunc("GET /bff/callback", b.Callback)
	mux.HandleFunc("GET /bff/logout", b.LogoutPage)
	mux.HandleFunc("POST /bff/logout", b.Logout)
	mux.Handle("GET /bff/user", b.Authorize(up))
	srv := httptest.NewTLSServer(mux)
	t.Cleanup(srv.Close)
	return &testGateway{b: b, idp: idp, srv: srv, up: up, clock: clock}
}

type pendingLogin struct {
	id, state, code string
}

// startLogin plays the browser up to auth's login page, which the fake IdP replaces by issuing a code.
func (g *testGateway) startLogin(t *testing.T, br *browser) pendingLogin {
	t.Helper()
	res := br.do(t, http.MethodGet, "/bff/login", nil)
	authorize, err := url.Parse(res.Header.Get("Location"))
	if err != nil {
		t.Fatal(err)
	}
	q := authorize.Query()
	p := pendingLogin{state: q.Get("state"), code: g.idp.issueCode(q.Get("code_challenge"), q.Get("nonce"), "acc-1")}
	for _, c := range res.Cookies() {
		if c.Name == loginCookie {
			p.id = c.Value
		}
	}
	return p
}

func (g *testGateway) login(t *testing.T, br *browser) *http.Response {
	t.Helper()
	p := g.startLogin(t, br)
	return br.do(t, http.MethodGet, callbackPath(p.state, p.code), nil)
}

func callbackPath(state, code string) string {
	return "/bff/callback?" + url.Values{"state": {state}, "code": {code}}.Encode()
}

type browser struct {
	base   *url.URL
	client *http.Client
}

func (g *testGateway) newBrowser(t *testing.T) *browser {
	t.Helper()
	jar, err := cookiejar.New(nil)
	if err != nil {
		t.Fatal(err)
	}
	// srv.Client returns one shared client, so each browser copies it to keep its own cookie jar.
	client := *g.srv.Client()
	client.Jar = jar
	client.CheckRedirect = func(*http.Request, []*http.Request) error { return http.ErrUseLastResponse }
	base, err := url.Parse(g.srv.URL)
	if err != nil {
		t.Fatal(err)
	}
	return &browser{base: base, client: &client}
}

func (br *browser) do(t *testing.T, method, path string, header http.Header) *http.Response {
	t.Helper()
	req, err := http.NewRequest(method, br.base.String()+path, nil)
	if err != nil {
		t.Fatal(err)
	}
	for name, values := range header {
		req.Header[name] = values
	}
	res, err := br.client.Do(req)
	if err != nil {
		t.Fatal(err)
	}
	_ = res.Body.Close()
	return res
}

// status is do for other goroutines, which must not call t.Fatal; it returns 0 on a transport error.
func (br *browser) status(path string) int {
	res, err := br.client.Get(br.base.String() + path)
	if err != nil {
		return 0
	}
	_ = res.Body.Close()
	return res.StatusCode
}

func (br *browser) cookie(name string) string {
	for _, c := range br.client.Jar.Cookies(br.base) {
		if c.Name == name {
			return c.Value
		}
	}
	return ""
}

type testClock struct {
	mu sync.Mutex
	t  time.Time
}

func (c *testClock) now() time.Time {
	c.mu.Lock()
	defer c.mu.Unlock()
	return c.t
}

func (c *testClock) advance(d time.Duration) {
	c.mu.Lock()
	defer c.mu.Unlock()
	c.t = c.t.Add(d)
}

type call struct {
	auth, cookie string
}

// upstream stands in for a proxied service and records what the gateway forwarded.
type upstream struct {
	mu    sync.Mutex
	calls []call
}

func (u *upstream) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	u.mu.Lock()
	u.calls = append(u.calls, call{auth: r.Header.Get("Authorization"), cookie: r.Header.Get("Cookie")})
	u.mu.Unlock()
	w.WriteHeader(http.StatusOK)
}

func (u *upstream) all() []call {
	u.mu.Lock()
	defer u.mu.Unlock()
	return append([]call(nil), u.calls...)
}

func (u *upstream) last() call {
	calls := u.all()
	if len(calls) == 0 {
		return call{}
	}
	return calls[len(calls)-1]
}
