package main

import (
	"math"
	"net"
	"net/http"
	"net/netip"
	"strconv"
	"sync"
	"time"

	"github.com/aakwat/the-crazy-microservice/gateway/internal/bff"
	"golang.org/x/time/rate"
)

// maxClients bounds memory; when full, refilled buckets are forgotten and newcomers refused.
const maxClients = 10_000

// perIP allows each client perMinute requests a minute, all at once after a quiet minute.
func perIP(perMinute int, next http.Handler) http.Handler {
	return newLimiter(perMinute, maxClients).wrap(next)
}

// limiter is a token bucket per client; each replica counts alone, so a cluster allows replicas × the rate.
type limiter struct {
	rate       rate.Limit
	burst      int
	maxClients int
	retryAfter string

	mu      sync.Mutex
	clients map[string]*rate.Limiter
}

func newLimiter(perMinute, maxClients int) *limiter {
	return &limiter{
		rate:       rate.Every(time.Minute / time.Duration(perMinute)),
		burst:      perMinute,
		maxClients: maxClients,
		retryAfter: strconv.Itoa(int(math.Ceil(60 / float64(perMinute)))),
		clients:    map[string]*rate.Limiter{},
	}
}

func (l *limiter) wrap(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if !l.allow(clientKey(r), time.Now()) {
			w.Header().Set("Retry-After", l.retryAfter)
			bff.Problem(w, http.StatusTooManyRequests, "too many requests, try again later")
			return
		}
		next.ServeHTTP(w, r)
	})
}

func (l *limiter) allow(key string, now time.Time) bool {
	l.mu.Lock()
	defer l.mu.Unlock()
	bucket, ok := l.clients[key]
	if !ok {
		if len(l.clients) >= l.maxClients {
			l.forgetRefilled(now)
		}
		if len(l.clients) >= l.maxClients {
			return false
		}
		bucket = rate.NewLimiter(l.rate, l.burst)
		l.clients[key] = bucket
	}
	return bucket.AllowN(now, 1)
}

// forgetRefilled drops full buckets; a new bucket for that client starts full, so no limit is lost.
func (l *limiter) forgetRefilled(now time.Time) {
	for key, bucket := range l.clients {
		if bucket.TokensAt(now) >= float64(l.burst) {
			delete(l.clients, key)
		}
	}
}

// clientKey uses RemoteAddr while the gateway is the edge; behind Traefik it must read a trusted X-Forwarded-For.
func clientKey(r *http.Request) string {
	host, _, err := net.SplitHostPort(r.RemoteAddr)
	if err != nil {
		host = r.RemoteAddr
	}
	addr, err := netip.ParseAddr(host)
	if err != nil {
		return host
	}
	addr = addr.Unmap()
	if addr.Is6() {
		// One IPv6 host usually holds a whole /64, so limiting single addresses would be trivial to dodge.
		prefix, _ := addr.Prefix(64)
		return prefix.String()
	}
	return addr.String()
}
