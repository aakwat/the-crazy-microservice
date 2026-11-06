package redisstore

import (
	"context"
	"testing"
	"time"

	"github.com/aakwat/the-crazy-microservice/gateway/internal/domain"
	"github.com/alicebob/miniredis/v2"
	"github.com/redis/go-redis/v9"
)

func newStore(t *testing.T) (*Store, *miniredis.Miniredis) {
	t.Helper()
	mr := miniredis.RunT(t)
	return New(redis.NewClient(&redis.Options{Addr: mr.Addr()})), mr
}

func TestSessionRoundTripsAndExpires(t *testing.T) {
	store, mr := newStore(t)
	ctx := context.Background()
	want := domain.Session{Subject: "acc-1", AccessToken: "at", RefreshToken: "rt",
		AccessExpiry: time.Date(2026, 11, 6, 9, 15, 0, 0, time.UTC)}

	if err := store.Save(ctx, "s1", want, time.Hour); err != nil {
		t.Fatal(err)
	}
	got, ok, err := store.Get(ctx, "s1")
	if err != nil || !ok || got != want {
		t.Fatalf("got %+v ok=%v err=%v", got, ok, err)
	}

	mr.FastForward(time.Hour + time.Second)
	if _, ok, _ := store.Get(ctx, "s1"); ok {
		t.Fatal("session must be gone after its TTL")
	}
}

func TestReplaceNeverRecreatesADeletedSession(t *testing.T) {
	store, _ := newStore(t)
	ctx := context.Background()
	_ = store.Save(ctx, "s1", domain.Session{AccessToken: "old"}, time.Hour)

	if ok, err := store.Replace(ctx, "s1", domain.Session{AccessToken: "new"}, time.Hour); !ok || err != nil {
		t.Fatalf("replacing a live session must succeed: %v %v", ok, err)
	}
	_ = store.Delete(ctx, "s1")
	if ok, _ := store.Replace(ctx, "s1", domain.Session{AccessToken: "newer"}, time.Hour); ok {
		t.Fatal("replace must not write a deleted session")
	}
	if _, found, _ := store.Get(ctx, "s1"); found {
		t.Fatal("the deleted session came back")
	}
}

func TestPendingLoginCanBeTakenOnlyOnce(t *testing.T) {
	store, _ := newStore(t)
	ctx := context.Background()
	if err := store.SavePending(ctx, "l1", domain.PendingLogin{State: "st"}, time.Minute); err != nil {
		t.Fatal(err)
	}

	first, ok, _ := store.TakePending(ctx, "l1")
	if !ok || first.State != "st" {
		t.Fatalf("first take should succeed, got %+v ok=%v", first, ok)
	}
	if _, ok, _ := store.TakePending(ctx, "l1"); ok {
		t.Fatal("a replayed callback must not find the pending login again")
	}
}

func TestOnlyOneHolderOfTheRefreshLock(t *testing.T) {
	store, _ := newStore(t)
	ctx := context.Background()

	first, _ := store.TryLock(ctx, "s1", 5*time.Second)
	second, _ := store.TryLock(ctx, "s1", 5*time.Second)
	if !first || second {
		t.Fatalf("want first=true second=false, got %v %v", first, second)
	}

	_ = store.Unlock(ctx, "s1")
	if again, _ := store.TryLock(ctx, "s1", 5*time.Second); !again {
		t.Fatal("after unlock the lock must be free")
	}
}
