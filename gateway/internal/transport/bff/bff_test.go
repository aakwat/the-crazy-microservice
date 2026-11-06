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

	"github.com/aakwat/the-crazy-microservice/gateway/internal/adapters/oidc"
	"github.com/aakwat/the-crazy-microservice/gateway/internal/adapters/redisstore"
	"github.com/aakwat/the-crazy-microservice/gateway/internal/app"
	"github.com/aakwat/the-crazy-microservice/gateway/internal/domain"
	"github.com/aakwat/the-crazy-microservice/gateway/internal/testidp"
	"github.com/alicebob/miniredis/v2"
	"github.com/redis/go-redis/v9"
)

func TestBrowserLoginThenMeThenLogout(t *testing.T) {
	gw := newGateway(t)
	browser := gw.newBrowser(t)

	res := browser.send(t, http.MethodGet, "/bff/login", nil)
	if res.StatusCode != http.StatusFound {
		t.Fatalf("login: want 302, got %d", res.StatusCode)
	}
	authorize, _ := url.Parse(res.Header.Get("Location"))
	q := authorize.Query()
	code := gw.idp.IssueCode(q.Get("code_challenge"), q.Get("nonce"), "acc-1")

	res = browser.send(t, http.MethodGet, "/bff/callback?"+url.Values{"state": {q.Get("state")}, "code": {code}}.Encode(), nil)
	if res.StatusCode != http.StatusSeeOther || res.Header.Get("Location") != afterLoginPath {
		t.Fatalf("callback: want 303 to %s, got %d %q", afterLoginPath, res.StatusCode, res.Header.Get("Location"))
	}

	if res = browser.send(t, http.MethodGet, "/bff/user", nil); res.StatusCode != http.StatusOK {
		t.Fatalf("/bff/user with a session: want 200, got %d", res.StatusCode)
	}
	auth, cookie := gw.upstream.last()
	if !strings.HasPrefix(auth, "Bearer ey") || cookie != "" {
		t.Fatalf("upstream must get a bearer token and no cookies, got auth=%q cookie=%q", auth, cookie)
	}

	res = browser.send(t, http.MethodPost, "/bff/logout", nil)
	if res.StatusCode != http.StatusSeeOther || !strings.Contains(res.Header.Get("Location"), "/connect/logout?id_token_hint=") {
		t.Fatalf("logout: want 303 to auth's end-session URL, got %d %q", res.StatusCode, res.Header.Get("Location"))
	}
	if res = browser.send(t, http.MethodGet, "/bff/user", nil); res.StatusCode != http.StatusUnauthorized {
		t.Fatalf("/bff/user after logout: want 401, got %d", res.StatusCode)
	}
}

func TestLoginCookieIsHostOnlySecureAndHidden(t *testing.T) {
	gw := newGateway(t)

	res := gw.newBrowser(t).send(t, http.MethodGet, "/bff/login", nil)

	cookies := res.Cookies()
	if len(cookies) != 1 {
		t.Fatalf("want one cookie, got %v", cookies)
	}
	c := cookies[0]
	if c.Name != loginCookie || !c.Secure || !c.HttpOnly || c.SameSite != http.SameSiteLaxMode || c.Path != "/" || c.Domain != "" {
		t.Fatalf("unsafe login cookie: %+v", c)
	}
}

func TestCallbackRejectsMissingLoginAndRefusals(t *testing.T) {
	gw := newGateway(t)
	browser := gw.newBrowser(t)

	for _, path := range []string{"/bff/callback?state=s&code=c", "/bff/callback?error=access_denied"} {
		res := browser.send(t, http.MethodGet, path, nil)
		if res.StatusCode != http.StatusBadRequest || res.Header.Get("Content-Type") != "application/problem+json" {
			t.Fatalf("%s: want a 400 problem, got %d %q", path, res.StatusCode, res.Header.Get("Content-Type"))
		}
	}
}

func TestBearerTokensMustBeAccessTokens(t *testing.T) {
	gw := newGateway(t)
	client := gw.newBrowser(t)
	bearer := func(token string) http.Header { return http.Header{"Authorization": {"Bearer " + token}} }

	valid := gw.idp.AccessToken(t, "acc-1", "at+jwt", testidp.Audience)
	if res := client.send(t, http.MethodGet, "/bff/user", bearer(valid)); res.StatusCode != http.StatusOK {
		t.Fatalf("a valid access token: want 200, got %d", res.StatusCode)
	}
	if auth, _ := gw.upstream.last(); auth != "Bearer "+valid {
		t.Fatalf("upstream must receive the caller's token, got %q", auth)
	}

	idTokenShaped := gw.idp.AccessToken(t, "acc-1", "JWT", testidp.Audience)
	res := client.send(t, http.MethodGet, "/bff/user", bearer(idTokenShaped))
	if res.StatusCode != http.StatusUnauthorized || res.Header.Get("WWW-Authenticate") != "Bearer" {
		t.Fatalf("typ JWT: want 401 with WWW-Authenticate, got %d", res.StatusCode)
	}
}

type gateway struct {
	srv      *httptest.Server
	idp      *testidp.IdP
	upstream *upstream
}

func newGateway(t *testing.T) *gateway {
	t.Helper()
	idp := testidp.New(t)
	store := redisstore.New(redis.NewClient(&redis.Options{Addr: miniredis.RunT(t).Addr()}))
	provider := oidc.New(context.Background(), oidc.Config{
		Issuer: idp.Issuer(), InternalURL: idp.Issuer(),
		ClientID: idp.ClientID, ClientSecret: idp.ClientSecret,
		RedirectURL: "https://gateway.test/bff/callback", PostLogoutURL: "https://gateway.test/bff/logged-out",
		Audience: testidp.Audience, HTTPClient: http.DefaultClient,
	})
	svc := app.New(store, provider, domain.Lifetime{Idle: 8 * time.Hour, Absolute: 12 * time.Hour}, time.Now)
	h := New(svc, slog.New(slog.DiscardHandler))

	up := &upstream{}
	mux := http.NewServeMux()
	h.Register(mux)
	mux.Handle("GET /bff/user", h.Authorize(up))
	// TLS, because browsers and cookiejar only send Secure cookies over https.
	srv := httptest.NewTLSServer(mux)
	t.Cleanup(srv.Close)
	return &gateway{srv: srv, idp: idp, upstream: up}
}

type browser struct {
	base   string
	client *http.Client
}

func (g *gateway) newBrowser(t *testing.T) *browser {
	t.Helper()
	jar, err := cookiejar.New(nil)
	if err != nil {
		t.Fatal(err)
	}
	client := g.srv.Client()
	client.Jar = jar
	client.CheckRedirect = func(*http.Request, []*http.Request) error { return http.ErrUseLastResponse }
	return &browser{base: g.srv.URL, client: client}
}

func (b *browser) send(t *testing.T, method, path string, header http.Header) *http.Response {
	t.Helper()
	req, err := http.NewRequest(method, b.base+path, nil)
	if err != nil {
		t.Fatal(err)
	}
	for name, values := range header {
		req.Header[name] = values
	}
	res, err := b.client.Do(req)
	if err != nil {
		t.Fatal(err)
	}
	_ = res.Body.Close()
	return res
}

// upstream stands in for a proxied service and records what the gateway forwarded.
type upstream struct {
	mu     sync.Mutex
	auth   string
	cookie string
}

func (u *upstream) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	u.mu.Lock()
	u.auth, u.cookie = r.Header.Get("Authorization"), r.Header.Get("Cookie")
	u.mu.Unlock()
	w.WriteHeader(http.StatusOK)
}

func (u *upstream) last() (auth, cookie string) {
	u.mu.Lock()
	defer u.mu.Unlock()
	return u.auth, u.cookie
}
