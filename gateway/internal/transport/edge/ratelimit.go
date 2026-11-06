package edge

import (
	"math"
	"net"
	"net/http"
	"net/netip"
	"strconv"
	"sync"
	"time"

	"github.com/aakwat/the-crazy-microservice/gateway/internal/transport/problem"
	"golang.org/x/time/rate"
)

// defaultMaxClients bounds memory; past it, refilled clients are forgotten and new ones refused.
const defaultMaxClients = 10_000

// PerIP is a token bucket per client; each replica counts alone, so the cluster allows replicas × rate.
type PerIP struct {
	rate       rate.Limit
	burst      int
	maxClients int
	mu         sync.Mutex
	clients    map[string]*rate.Limiter
}

// NewPerIP allows burst requests at once per client, refilled at r per second.
func NewPerIP(r rate.Limit, burst int) *PerIP {
	return &PerIP{rate: r, burst: burst, maxClients: defaultMaxClients, clients: map[string]*rate.Limiter{}}
}

// Wrap answers 429 with Retry-After once a client has used up its bucket.
func (p *PerIP) Wrap(next http.Handler) http.Handler {
	retryAfter := strconv.Itoa(int(math.Ceil(1 / float64(p.rate))))
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if !p.allow(clientKey(r), time.Now()) {
			w.Header().Set("Retry-After", retryAfter)
			problem.Write(w, http.StatusTooManyRequests, "too many requests, try again later")
			return
		}
		next.ServeHTTP(w, r)
	})
}

func (p *PerIP) allow(key string, now time.Time) bool {
	p.mu.Lock()
	defer p.mu.Unlock()
	lim, ok := p.clients[key]
	if !ok {
		if len(p.clients) >= p.maxClients {
			p.forgetRefilled(now)
		}
		if len(p.clients) >= p.maxClients {
			// Still full means every known client is mid-burst; refusing newcomers keeps memory bounded.
			return false
		}
		lim = rate.NewLimiter(p.rate, p.burst)
		p.clients[key] = lim
	}
	return lim.AllowN(now, 1)
}

// forgetRefilled drops full buckets; a new limiter for that client starts full, so no limit is lost.
func (p *PerIP) forgetRefilled(now time.Time) {
	for key, lim := range p.clients {
		if lim.TokensAt(now) >= float64(p.burst) {
			delete(p.clients, key)
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
