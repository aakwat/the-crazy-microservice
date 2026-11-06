package edge

import (
	"errors"
	"io"
	"net/http"
	"net/http/httptest"
	"regexp"
	"strings"
	"testing"
)

var ok = http.HandlerFunc(func(w http.ResponseWriter, _ *http.Request) { w.WriteHeader(http.StatusNoContent) })

func TestSecurityHeadersFillGapsButKeepUpstreamValues(t *testing.T) {
	upstream := http.HandlerFunc(func(w http.ResponseWriter, _ *http.Request) {
		w.Header().Set("X-Frame-Options", "SAMEORIGIN")
		w.WriteHeader(http.StatusOK)
	})
	rec := httptest.NewRecorder()

	SecurityHeaders(upstream).ServeHTTP(rec, httptest.NewRequest(http.MethodGet, "/", nil))

	h := rec.Result().Header
	if h.Get("X-Content-Type-Options") != "nosniff" || h.Get("Referrer-Policy") != "no-referrer" {
		t.Fatalf("defaults missing: %v", h)
	}
	if got := h.Values("X-Frame-Options"); len(got) != 1 || got[0] != "SAMEORIGIN" {
		t.Fatalf("upstream value must win without duplicates, got %v", got)
	}
}

func TestMaxBodyRejectsDeclaredAndStreamedOversizeBodies(t *testing.T) {
	rec := httptest.NewRecorder()
	MaxBody(8)(ok).ServeHTTP(rec, httptest.NewRequest(http.MethodPost, "/", strings.NewReader("123456789")))
	if rec.Code != http.StatusRequestEntityTooLarge {
		t.Fatalf("declared oversize body: want 413, got %d", rec.Code)
	}

	var readErr error
	reader := http.HandlerFunc(func(_ http.ResponseWriter, r *http.Request) { _, readErr = io.ReadAll(r.Body) })
	req := httptest.NewRequest(http.MethodPost, "/", strings.NewReader("123456789"))
	req.ContentLength = -1
	MaxBody(8)(reader).ServeHTTP(httptest.NewRecorder(), req)
	var tooLarge *http.MaxBytesError
	if !errors.As(readErr, &tooLarge) {
		t.Fatalf("streamed oversize body: want MaxBytesError, got %v", readErr)
	}
}

func TestPerIPLimitsEachClientSeparately(t *testing.T) {
	handler := NewPerIP(1, 2).Wrap(ok)
	call := func(addr string) *httptest.ResponseRecorder {
		req := httptest.NewRequest(http.MethodPost, "/auth/register", nil)
		req.RemoteAddr = addr
		rec := httptest.NewRecorder()
		handler.ServeHTTP(rec, req)
		return rec
	}

	call("203.0.113.1:1000")
	call("203.0.113.1:1001")
	limited := call("203.0.113.1:1002")
	if limited.Code != http.StatusTooManyRequests || limited.Header().Get("Retry-After") != "1" {
		t.Fatalf("third request in the burst: want 429 with Retry-After, got %d", limited.Code)
	}
	if call("203.0.113.2:1000").Code != http.StatusNoContent {
		t.Fatal("another client must have its own bucket")
	}
}

func TestPerIPRefusesNewClientsWhenFullOfActiveOnes(t *testing.T) {
	limiter := NewPerIP(1, 1)
	limiter.maxClients = 1
	handler := limiter.Wrap(ok)
	call := func(addr string) int {
		req := httptest.NewRequest(http.MethodPost, "/", nil)
		req.RemoteAddr = addr
		rec := httptest.NewRecorder()
		handler.ServeHTTP(rec, req)
		return rec.Code
	}

	if call("203.0.113.1:1000") != http.StatusNoContent {
		t.Fatal("the first client fits")
	}
	if call("203.0.113.2:1000") != http.StatusTooManyRequests {
		t.Fatal("a new client must be refused while the table is full of active buckets")
	}
}

func TestPerIPGroupsIPv6ByPrefix(t *testing.T) {
	handler := NewPerIP(1, 1).Wrap(ok)
	call := func(addr string) int {
		req := httptest.NewRequest(http.MethodPost, "/", nil)
		req.RemoteAddr = addr
		rec := httptest.NewRecorder()
		handler.ServeHTTP(rec, req)
		return rec.Code
	}

	call("[2001:db8:1:2::1]:1000")
	if call("[2001:db8:1:2::ffff]:1000") != http.StatusTooManyRequests {
		t.Fatal("addresses in the same /64 must share a bucket")
	}
}

func TestTraceparentReplacesTheClientValue(t *testing.T) {
	var got string
	capture := http.HandlerFunc(func(_ http.ResponseWriter, r *http.Request) { got = r.Header.Get("Traceparent") })
	req := httptest.NewRequest(http.MethodGet, "/", nil)
	forged := "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01"
	req.Header.Set("Traceparent", forged)

	Traceparent(capture).ServeHTTP(httptest.NewRecorder(), req)

	if got == forged || !regexp.MustCompile(`^00-[0-9a-f]{32}-[0-9a-f]{16}-01$`).MatchString(got) {
		t.Fatalf("want a fresh valid traceparent, got %q", got)
	}
}
