// Package edge holds the middleware every request passes before routing: headers, limits and tracing.
package edge

import "net/http"

// defaultHeaders is a floor for every response; auth's own pages may set stricter or different values.
var defaultHeaders = map[string]string{
	"X-Content-Type-Options": "nosniff",
	"Referrer-Policy":        "no-referrer",
	"X-Frame-Options":        "DENY",
}

// SecurityHeaders fills in defaultHeaders that the handler did not set, so proxied headers are never duplicated.
func SecurityHeaders(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		next.ServeHTTP(&headerWriter{ResponseWriter: w}, r)
	})
}

type headerWriter struct {
	http.ResponseWriter
	done bool
}

func (w *headerWriter) WriteHeader(status int) {
	w.fillDefaults()
	w.ResponseWriter.WriteHeader(status)
}

func (w *headerWriter) Write(b []byte) (int, error) {
	w.fillDefaults()
	return w.ResponseWriter.Write(b)
}

// Unwrap lets http.ResponseController reach Flush and Hijack, which the WebSocket proxy needs.
func (w *headerWriter) Unwrap() http.ResponseWriter {
	return w.ResponseWriter
}

func (w *headerWriter) fillDefaults() {
	if w.done {
		return
	}
	w.done = true
	h := w.Header()
	for name, value := range defaultHeaders {
		if h.Get(name) == "" {
			h.Set(name, value)
		}
	}
}
