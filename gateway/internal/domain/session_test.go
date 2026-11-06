package domain

import (
	"testing"
	"time"
)

var t0 = time.Date(2026, 11, 6, 9, 0, 0, 0, time.UTC)

func TestNeedsRefreshWithinSkew(t *testing.T) {
	s := Session{AccessExpiry: t0.Add(time.Minute)}

	if s.NeedsRefresh(t0, 30*time.Second) {
		t.Fatal("a token valid for another minute must not refresh with a 30s skew")
	}
	if !s.NeedsRefresh(t0.Add(31*time.Second), 30*time.Second) {
		t.Fatal("a token expiring inside the skew must refresh")
	}
}

func TestLifetimeEndsAtTheAbsoluteCapEvenWhileActive(t *testing.T) {
	l := Lifetime{Idle: 8 * time.Hour, Absolute: 12 * time.Hour}
	s := Session{LoginAt: t0}

	if l.Expired(s, t0.Add(11*time.Hour+59*time.Minute)) {
		t.Fatal("still inside the absolute lifetime")
	}
	if !l.Expired(s, t0.Add(12*time.Hour)) {
		t.Fatal("the absolute lifetime has passed")
	}
}

func TestTTLIsTheIdleWindowCappedByTheAbsoluteEnd(t *testing.T) {
	l := Lifetime{Idle: 8 * time.Hour, Absolute: 12 * time.Hour}
	s := Session{LoginAt: t0}

	if got := l.TTL(s, t0.Add(time.Hour)); got != 8*time.Hour {
		t.Fatalf("early in the session the idle window applies, got %v", got)
	}
	if got := l.TTL(s, t0.Add(10*time.Hour)); got != 2*time.Hour {
		t.Fatalf("late in the session the absolute end caps it, got %v", got)
	}
}
