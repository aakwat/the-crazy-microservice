package bff

import (
	"context"
	"encoding/json"
	"errors"
	"time"

	"github.com/redis/go-redis/v9"
)

// Keys live under gw:, the only prefix the gateway's Redis user may touch.
const (
	sessionKey = "gw:session:"
	loginKey   = "gw:login:"
	lockKey    = "gw:refresh-lock:"
)

// session is what Redis holds per login; the browser only ever sees its random id.
type session struct {
	Subject      string    `json:"sub"`
	AccessToken  string    `json:"access_token"`
	RefreshToken string    `json:"refresh_token"`
	IDToken      string    `json:"id_token"`
	Expiry       time.Time `json:"expiry"`
	LoginAt      time.Time `json:"login_at"`
}

// pending is what a login must remember until auth redirects back.
type pending struct {
	State    string `json:"state"`
	Verifier string `json:"verifier"`
	Nonce    string `json:"nonce"`
}

// ttl is the idle window, never past the absolute end; zero or less means the session is over.
func (s session) ttl(now time.Time) time.Duration {
	return min(idleTTL, s.LoginAt.Add(absoluteTTL).Sub(now))
}

func (s session) needsRefresh(now time.Time) bool {
	return !now.Add(refreshSkew).Before(s.Expiry)
}

func (b *BFF) loadSession(ctx context.Context, id string) (session, bool, error) {
	var s session
	ok, err := getJSON(b.rdb.Get(ctx, sessionKey+id), &s)
	return s, ok, err
}

func (b *BFF) createSession(ctx context.Context, id string, s session) error {
	data, err := json.Marshal(s)
	if err != nil {
		return err
	}
	return b.rdb.Set(ctx, sessionKey+id, data, s.ttl(b.now())).Err()
}

// replaceSession uses SET XX, so a refresh cannot revive a session that was logged out meanwhile.
func (b *BFF) replaceSession(ctx context.Context, id string, s session) (bool, error) {
	ttl := s.ttl(b.now())
	if ttl <= 0 {
		// Redis treats a non-positive TTL as no expiry, which would make the session immortal.
		return false, nil
	}
	data, err := json.Marshal(s)
	if err != nil {
		return false, err
	}
	return b.rdb.SetXX(ctx, sessionKey+id, data, ttl).Result()
}

func (b *BFF) savePending(ctx context.Context, id string, p pending) error {
	data, err := json.Marshal(p)
	if err != nil {
		return err
	}
	return b.rdb.Set(ctx, loginKey+id, data, pendingTTL).Err()
}

// takePending reads and deletes in one step (GETDEL), so a login completes at most once.
func (b *BFF) takePending(ctx context.Context, id string) (pending, bool, error) {
	var p pending
	ok, err := getJSON(b.rdb.GetDel(ctx, loginKey+id), &p)
	return p, ok, err
}

func getJSON(cmd *redis.StringCmd, v any) (bool, error) {
	data, err := cmd.Bytes()
	if errors.Is(err, redis.Nil) {
		return false, nil
	}
	if err != nil {
		return false, err
	}
	return true, json.Unmarshal(data, v)
}
