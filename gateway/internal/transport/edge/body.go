package edge

import (
	"net/http"

	"github.com/aakwat/the-crazy-microservice/gateway/internal/transport/problem"
)

// MaxBody rejects bodies over limit: up front from Content-Length, and while reading for chunked bodies.
func MaxBody(limit int64) func(http.Handler) http.Handler {
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			if r.ContentLength > limit {
				problem.Write(w, http.StatusRequestEntityTooLarge, "request body is too large")
				return
			}
			r.Body = http.MaxBytesReader(w, r.Body, limit)
			next.ServeHTTP(w, r)
		})
	}
}
