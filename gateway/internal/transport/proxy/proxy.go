// Package proxy forwards requests to private services; upstream failures become problem responses.
package proxy

import (
	"context"
	"errors"
	"log/slog"
	"net"
	"net/http"
	"net/http/httputil"
	"net/url"
	"time"

	"github.com/aakwat/the-crazy-microservice/gateway/internal/transport/problem"
)

const dialTimeout = 5 * time.Second

// New proxies to target without retries; public is the origin browsers use, timeout bounds the header wait.
func New(target, public *url.URL, timeout time.Duration, log *slog.Logger) *httputil.ReverseProxy {
	transport := http.DefaultTransport.(*http.Transport).Clone()
	// Private hops go direct; an HTTP_PROXY in the container must not reroute them.
	transport.Proxy = nil
	transport.DialContext = (&net.Dialer{Timeout: dialTimeout}).DialContext
	transport.ResponseHeaderTimeout = timeout

	return &httputil.ReverseProxy{
		Rewrite: func(pr *httputil.ProxyRequest) {
			pr.SetURL(target)
			// Rewrite already dropped the client's X-Forwarded-*; X-Forwarded-For comes from the real connection.
			pr.SetXForwarded()
			// Host is client-controlled and TLS may end before the gateway, so the public origin comes from config.
			pr.Out.Header.Set("X-Forwarded-Host", public.Host)
			pr.Out.Header.Set("X-Forwarded-Proto", public.Scheme)
		},
		Transport:    transport,
		ErrorHandler: errorHandler(log),
	}
}

func errorHandler(log *slog.Logger) func(http.ResponseWriter, *http.Request, error) {
	return func(w http.ResponseWriter, r *http.Request, err error) {
		var tooLarge *http.MaxBytesError
		var netErr net.Error
		switch {
		case errors.As(err, &tooLarge):
			problem.Write(w, http.StatusRequestEntityTooLarge, "request body is too large")
		case errors.Is(err, context.Canceled):
			// The client hung up, so nobody is left to read an answer.
		case errors.As(err, &netErr) && netErr.Timeout():
			log.WarnContext(r.Context(), "upstream timed out", "err", err, "traceparent", r.Header.Get("Traceparent"))
			problem.Write(w, http.StatusGatewayTimeout, "upstream service timed out")
		default:
			log.ErrorContext(r.Context(), "upstream failed", "err", err, "traceparent", r.Header.Get("Traceparent"))
			problem.Write(w, http.StatusBadGateway, "upstream service is unavailable")
		}
	}
}
