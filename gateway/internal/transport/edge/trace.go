package edge

import (
	"crypto/rand"
	"encoding/hex"
	"net/http"
)

// Traceparent starts a new W3C trace per request; a client's traceparent is untrusted, so it is replaced.
func Traceparent(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		r.Header.Set("Traceparent", newTraceparent())
		r.Header.Del("Tracestate")
		next.ServeHTTP(w, r)
	})
}

// newTraceparent builds version 00 with a 16-byte trace id, an 8-byte span id and the sampled flag.
func newTraceparent() string {
	var b [24]byte
	_, _ = rand.Read(b[:])
	return "00-" + hex.EncodeToString(b[:16]) + "-" + hex.EncodeToString(b[16:]) + "-01"
}
