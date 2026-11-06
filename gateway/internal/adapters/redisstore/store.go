// Package redisstore implements domain.SessionStore on Redis, under gw:, the only prefix the ACL allows.
package redisstore

import (
	"context"
	"encoding/json"
	"errors"
	"time"

	"github.com/aakwat/the-crazy-microservice/gateway/internal/domain"
	"github.com/redis/go-redis/v9"
)

const (
	sessionPrefix = "gw:session:"
	loginPrefix   = "gw:login:"
	lockPrefix    = "gw:refresh-lock:"
)

// Store keeps sessions, pending logins and refresh locks in Redis.
type Store struct {
	rdb *redis.Client
}

var _ domain.SessionStore = (*Store)(nil)

// New returns a Store that uses rdb for every session operation.
func New(rdb *redis.Client) *Store {
	return &Store{rdb: rdb}
}

func (s *Store) Save(ctx context.Context, id string, sess domain.Session, ttl time.Duration) error {
	return setJSON(ctx, s.rdb, sessionPrefix+id, sess, ttl)
}

func (s *Store) Get(ctx context.Context, id string) (domain.Session, bool, error) {
	var sess domain.Session
	ok, err := getJSON(s.rdb.Get(ctx, sessionPrefix+id), &sess)
	return sess, ok, err
}

// Replace writes only over an existing session (SET XX), so a late refresh cannot revive a deleted one.
func (s *Store) Replace(ctx context.Context, id string, sess domain.Session, ttl time.Duration) (bool, error) {
	b, err := json.Marshal(sess)
	if err != nil {
		return false, err
	}
	return s.rdb.SetXX(ctx, sessionPrefix+id, b, ttl).Result()
}

func (s *Store) Delete(ctx context.Context, id string) error {
	return s.rdb.Del(ctx, sessionPrefix+id).Err()
}

func (s *Store) SavePending(ctx context.Context, id string, p domain.PendingLogin, ttl time.Duration) error {
	return setJSON(ctx, s.rdb, loginPrefix+id, p, ttl)
}

func (s *Store) TakePending(ctx context.Context, id string) (domain.PendingLogin, bool, error) {
	var p domain.PendingLogin
	ok, err := getJSON(s.rdb.GetDel(ctx, loginPrefix+id), &p)
	return p, ok, err
}

func (s *Store) TryLock(ctx context.Context, id string, ttl time.Duration) (bool, error) {
	return s.rdb.SetNX(ctx, lockPrefix+id, "1", ttl).Result()
}

// Unlock uses a plain DEL: a refresh slower than the lock TTL could free a later holder's lock (a retry).
func (s *Store) Unlock(ctx context.Context, id string) error {
	return s.rdb.Del(ctx, lockPrefix+id).Err()
}

func setJSON(ctx context.Context, rdb *redis.Client, key string, v any, ttl time.Duration) error {
	b, err := json.Marshal(v)
	if err != nil {
		return err
	}
	return rdb.Set(ctx, key, b, ttl).Err()
}

func getJSON(cmd *redis.StringCmd, v any) (bool, error) {
	b, err := cmd.Bytes()
	if errors.Is(err, redis.Nil) {
		return false, nil
	}
	if err != nil {
		return false, err
	}
	return true, json.Unmarshal(b, v)
}
