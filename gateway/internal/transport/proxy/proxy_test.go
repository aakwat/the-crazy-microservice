package proxy

import (
	"io"
	"log/slog"
	"net/http"
	"net/http/httptest"
	"net/url"
	"strings"
	"testing"
	"time"
)

var (
	discard = slog.New(slog.DiscardHandler)
	public  = &url.URL{Scheme: "https", Host: "tickets.example"}
)

func target(t *testing.T, h http.HandlerFunc) *url.URL {
	t.Helper()
	srv := httptest.NewServer(h)
	t.Cleanup(srv.Close)
	u, err := url.Parse(srv.URL)
	if err != nil {
		t.Fatal(err)
	}
	return u
}

func TestForwardedHeadersIgnoreTheClient(t *testing.T) {
	seen := make(chan *http.Request, 1)
	up := target(t, func(w http.ResponseWriter, r *http.Request) { seen <- r })
	req := httptest.NewRequest(http.MethodGet, "http://evil.test/login?error", nil)
	req.Header.Set("X-Forwarded-For", "198.51.100.66")
	req.Header.Set("X-Forwarded-Host", "evil.test")

	New(up, public, time.Second, discard).ServeHTTP(httptest.NewRecorder(), req)

	got := <-seen
	if got.URL.Path != "/login" || got.URL.RawQuery != "error" {
		t.Fatalf("path or query lost: %s", got.URL)
	}
	h := got.Header
	if h.Get("X-Forwarded-For") != "192.0.2.1" || h.Get("X-Forwarded-Host") != "tickets.example" || h.Get("X-Forwarded-Proto") != "https" {
		t.Fatalf("forwarded headers must come from the connection and config, never the client, got %v", h)
	}
}

func TestUnreachableUpstreamIsA502Problem(t *testing.T) {
	srv := httptest.NewServer(http.NotFoundHandler())
	gone, _ := url.Parse(srv.URL)
	srv.Close()
	rec := httptest.NewRecorder()

	New(gone, public, time.Second, discard).ServeHTTP(rec, httptest.NewRequest(http.MethodGet, "/", nil))

	if rec.Code != http.StatusBadGateway || rec.Header().Get("Content-Type") != "application/problem+json" {
		t.Fatalf("want a 502 problem, got %d %q", rec.Code, rec.Header().Get("Content-Type"))
	}
}

func TestSlowUpstreamIsA504Problem(t *testing.T) {
	up := target(t, func(_ http.ResponseWriter, r *http.Request) {
		select {
		case <-r.Context().Done():
		case <-time.After(5 * time.Second):
		}
	})
	rec := httptest.NewRecorder()

	New(up, public, 50*time.Millisecond, discard).ServeHTTP(rec, httptest.NewRequest(http.MethodGet, "/", nil))

	if rec.Code != http.StatusGatewayTimeout {
		t.Fatalf("want 504, got %d", rec.Code)
	}
}

func TestStreamedOversizeBodyIsA413Problem(t *testing.T) {
	up := target(t, func(_ http.ResponseWriter, r *http.Request) { _, _ = io.Copy(io.Discard, r.Body) })
	rec := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodPost, "/auth/register", nil)
	req.Body = http.MaxBytesReader(rec, io.NopCloser(strings.NewReader(strings.Repeat("x", 64))), 8)
	req.ContentLength = -1

	New(up, public, time.Second, discard).ServeHTTP(rec, req)

	if rec.Code != http.StatusRequestEntityTooLarge {
		t.Fatalf("want 413, got %d", rec.Code)
	}
}
