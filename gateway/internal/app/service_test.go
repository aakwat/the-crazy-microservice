package app

import (
	"context"
	"errors"
	"net/http"
	"net/url"
	"strings"
	"sync"
	"testing"
	"time"

	"github.com/aakwat/the-crazy-microservice/gateway/internal/adapters/oidc"
	"github.com/aakwat/the-crazy-microservice/gateway/internal/domain"
	"github.com/aakwat/the-crazy-microservice/gateway/internal/testidp"
)

func TestLoginCreatesASessionForTheSubject(t *testing.T) {
	svc, idp, store, _ := newService(t)
	ctx := context.Background()

	id := login(t, svc, idp)

	sess, ok, _ := store.Get(ctx, id)
	if !ok || sess.Subject != "acc-1" {
		t.Fatalf("session not stored for the subject: %+v", sess)
	}
	if token, err := svc.AccessToken(ctx, id); err != nil || token != sess.AccessToken {
		t.Fatalf("a fresh token must be returned without refreshing, got %v", err)
	}
}

func TestCallbackWorksOnceAndOnlyWithItsState(t *testing.T) {
	svc, idp, _, _ := newService(t)
	ctx := context.Background()

	start, q := startLogin(t, svc)
	code := idp.IssueCode(q.Get("code_challenge"), q.Get("nonce"), "acc-1")
	if _, _, err := svc.CompleteLogin(ctx, start.LoginID, q.Get("state"), code); err != nil {
		t.Fatalf("first callback must succeed: %v", err)
	}
	if _, _, err := svc.CompleteLogin(ctx, start.LoginID, q.Get("state"), code); !errors.Is(err, ErrLoginInvalid) {
		t.Fatalf("a replayed callback must be rejected, got %v", err)
	}

	start, q = startLogin(t, svc)
	code = idp.IssueCode(q.Get("code_challenge"), q.Get("nonce"), "acc-1")
	if _, _, err := svc.CompleteLogin(ctx, start.LoginID, "guessed", code); !errors.Is(err, ErrLoginInvalid) {
		t.Fatalf("a wrong state must be rejected, got %v", err)
	}
	if _, _, err := svc.CompleteLogin(ctx, start.LoginID, q.Get("state"), code); !errors.Is(err, ErrLoginInvalid) {
		t.Fatalf("a wrong state must also burn the login, got %v", err)
	}
}

func TestConcurrentRequestsShareOneRefresh(t *testing.T) {
	svc, idp, store, clock := newService(t)
	ctx := context.Background()
	id := login(t, svc, idp)
	before, _, _ := store.Get(ctx, id)

	idp.SetAccessTTL(time.Hour)
	clock.advance(14*time.Minute + 45*time.Second)

	tokens := make([]string, 5)
	errs := make([]error, 5)
	var wg sync.WaitGroup
	for i := range tokens {
		wg.Go(func() { tokens[i], errs[i] = svc.AccessToken(ctx, id) })
	}
	wg.Wait()

	// With rotation, a second refresh would fail and end the session, so every caller succeeding proves one refresh.
	for i := range tokens {
		if errs[i] != nil || tokens[i] == before.AccessToken || tokens[i] != tokens[0] {
			t.Fatalf("caller %d: token changed=%v same-as-others=%v err=%v",
				i, tokens[i] != before.AccessToken, tokens[i] == tokens[0], errs[i])
		}
	}
}

func TestFailedRefreshEndsTheSession(t *testing.T) {
	svc, idp, store, clock := newService(t)
	ctx := context.Background()
	id := login(t, svc, idp)
	sess, _, _ := store.Get(ctx, id)
	sess.RefreshToken = "revoked-elsewhere"
	_ = store.Save(ctx, id, sess, time.Hour)

	clock.advance(15 * time.Minute)

	if _, err := svc.AccessToken(ctx, id); !errors.Is(err, ErrNoSession) {
		t.Fatalf("want ErrNoSession, got %v", err)
	}
	if _, ok, _ := store.Get(ctx, id); ok {
		t.Fatal("a session whose refresh failed must be deleted")
	}
}

func TestLogoutDuringRefreshDoesNotReviveTheSession(t *testing.T) {
	svc, idp, store, clock := newService(t)
	ctx := context.Background()
	id := login(t, svc, idp)
	idp.SetAccessTTL(time.Hour)
	clock.advance(14*time.Minute + 45*time.Second)
	// A logout on another replica lands while this one waits for auth to rotate the tokens.
	store.beforeReplace = func() { _ = store.Delete(ctx, id) }

	if _, err := svc.AccessToken(ctx, id); !errors.Is(err, ErrNoSession) {
		t.Fatalf("want ErrNoSession, got %v", err)
	}
	if _, ok, _ := store.Get(ctx, id); ok {
		t.Fatal("the refresh revived a logged-out session")
	}
	if revoked := idp.RevokedTokens(); len(revoked) != 1 {
		t.Fatalf("the orphaned refresh token must be revoked, got %v", revoked)
	}
}

func TestRefreshUsesTheSessionAnotherReplicaAlreadyRotated(t *testing.T) {
	svc, idp, store, clock := newService(t)
	ctx := context.Background()
	id := login(t, svc, idp)
	clock.advance(14*time.Minute + 45*time.Second)
	// Another replica finishes its refresh between this replica's read and its lock.
	store.onLock = func() {
		sess, _, _ := store.Get(ctx, id)
		sess.AccessToken, sess.AccessExpiry = "rotated-elsewhere", clock.now().Add(time.Hour)
		_ = store.Save(ctx, id, sess, time.Hour)
	}

	token, err := svc.AccessToken(ctx, id)
	if err != nil || token != "rotated-elsewhere" {
		t.Fatalf("want the other replica's token without a second refresh, got %q %v", token, err)
	}
}

func TestAuthOutageDuringRefreshKeepsTheSession(t *testing.T) {
	svc, idp, store, clock := newService(t)
	ctx := context.Background()
	id := login(t, svc, idp)
	clock.advance(15 * time.Minute)
	idp.Server.Close()

	if _, err := svc.AccessToken(ctx, id); err == nil || errors.Is(err, ErrNoSession) {
		t.Fatalf("an outage must be an error but not a logout, got %v", err)
	}
	if _, ok, _ := store.Get(ctx, id); !ok {
		t.Fatal("an auth outage must not delete the session")
	}
}

func TestSessionEndsAtTheAbsoluteCap(t *testing.T) {
	svc, idp, store, clock := newService(t)
	ctx := context.Background()
	id := login(t, svc, idp)

	clock.advance(12 * time.Hour)

	if _, err := svc.AccessToken(ctx, id); !errors.Is(err, ErrNoSession) {
		t.Fatalf("want ErrNoSession at the absolute cap, got %v", err)
	}
	if _, ok, _ := store.Get(ctx, id); ok {
		t.Fatal("an expired session must be deleted")
	}
}

func TestLogoutRevokesDeletesAndEndsTheAuthSession(t *testing.T) {
	svc, idp, store, _ := newService(t)
	ctx := context.Background()
	id := login(t, svc, idp)
	sess, _, _ := store.Get(ctx, id)

	endURL, err := svc.Logout(ctx, id)
	if err != nil || !strings.Contains(endURL, "id_token_hint=") {
		t.Fatalf("logout failed: %q %v", endURL, err)
	}
	if revoked := idp.RevokedTokens(); len(revoked) != 1 || revoked[0] != sess.RefreshToken {
		t.Fatalf("refresh token not revoked: %v", revoked)
	}
	if _, ok, _ := store.Get(ctx, id); ok {
		t.Fatal("the session must be deleted")
	}
	if endURL, err := svc.Logout(ctx, id); err != nil || endURL != "" {
		t.Fatalf("logout without a session must be a no-op, got %q %v", endURL, err)
	}
}

func newService(t *testing.T) (*Service, *testidp.IdP, *memStore, *testClock) {
	t.Helper()
	idp := testidp.New(t)
	provider := oidc.New(context.Background(), oidc.Config{
		Issuer: idp.Issuer(), InternalURL: idp.Issuer(),
		ClientID: idp.ClientID, ClientSecret: idp.ClientSecret,
		RedirectURL: "http://gateway.test/bff/callback", PostLogoutURL: "http://gateway.test/",
		Audience: testidp.Audience, HTTPClient: http.DefaultClient,
	})
	store := newMemStore()
	clock := &testClock{t: time.Now()}
	lifetime := domain.Lifetime{Idle: 8 * time.Hour, Absolute: 12 * time.Hour}
	return New(store, provider, lifetime, clock.now), idp, store, clock
}

func startLogin(t *testing.T, svc *Service) (LoginStart, url.Values) {
	t.Helper()
	start, err := svc.StartLogin(context.Background())
	if err != nil {
		t.Fatal(err)
	}
	u, err := url.Parse(start.RedirectURL)
	if err != nil {
		t.Fatal(err)
	}
	return start, u.Query()
}

// login plays the browser: auth's login page is replaced by issuing a code for the URL's challenge.
func login(t *testing.T, svc *Service, idp *testidp.IdP) string {
	t.Helper()
	start, q := startLogin(t, svc)
	code := idp.IssueCode(q.Get("code_challenge"), q.Get("nonce"), "acc-1")
	id, _, err := svc.CompleteLogin(context.Background(), start.LoginID, q.Get("state"), code)
	if err != nil {
		t.Fatal(err)
	}
	return id
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

// memStore is an in-memory domain.SessionStore; it ignores TTLs because the clock is faked.
type memStore struct {
	mu       sync.Mutex
	sessions map[string]domain.Session
	pending  map[string]domain.PendingLogin
	locks    map[string]bool
	// onLock and beforeReplace stage what another replica does at that moment.
	onLock        func()
	beforeReplace func()
}

func newMemStore() *memStore {
	return &memStore{sessions: map[string]domain.Session{}, pending: map[string]domain.PendingLogin{},
		locks: map[string]bool{}}
}

func (m *memStore) Save(_ context.Context, id string, s domain.Session, _ time.Duration) error {
	m.mu.Lock()
	defer m.mu.Unlock()
	m.sessions[id] = s
	return nil
}

func (m *memStore) Get(_ context.Context, id string) (domain.Session, bool, error) {
	m.mu.Lock()
	defer m.mu.Unlock()
	s, ok := m.sessions[id]
	return s, ok, nil
}

func (m *memStore) Replace(_ context.Context, id string, s domain.Session, _ time.Duration) (bool, error) {
	if m.beforeReplace != nil {
		m.beforeReplace()
	}
	m.mu.Lock()
	defer m.mu.Unlock()
	if _, ok := m.sessions[id]; !ok {
		return false, nil
	}
	m.sessions[id] = s
	return true, nil
}

func (m *memStore) Delete(_ context.Context, id string) error {
	m.mu.Lock()
	defer m.mu.Unlock()
	delete(m.sessions, id)
	return nil
}

func (m *memStore) SavePending(_ context.Context, id string, p domain.PendingLogin, _ time.Duration) error {
	m.mu.Lock()
	defer m.mu.Unlock()
	m.pending[id] = p
	return nil
}

func (m *memStore) TakePending(_ context.Context, id string) (domain.PendingLogin, bool, error) {
	m.mu.Lock()
	defer m.mu.Unlock()
	p, ok := m.pending[id]
	delete(m.pending, id)
	return p, ok, nil
}

func (m *memStore) TryLock(_ context.Context, id string, _ time.Duration) (bool, error) {
	m.mu.Lock()
	if m.locks[id] {
		m.mu.Unlock()
		return false, nil
	}
	m.locks[id] = true
	m.mu.Unlock()
	if m.onLock != nil {
		m.onLock()
	}
	return true, nil
}

func (m *memStore) Unlock(_ context.Context, id string) error {
	m.mu.Lock()
	defer m.mu.Unlock()
	delete(m.locks, id)
	return nil
}
