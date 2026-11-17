package main

import (
	"cmp"
	"net/http"

	"github.com/felixge/httpsnoop"
	"go.opentelemetry.io/otel"
	"go.opentelemetry.io/otel/attribute"
	"go.opentelemetry.io/otel/codes"
	"go.opentelemetry.io/otel/propagation"
	"go.opentelemetry.io/otel/trace"

	"github.com/aakwat/the-crazy-microservice/gateway/internal/bff"
)

// securityHeaders go on every response; the gateway's values replace any that an upstream sends.
var securityHeaders = map[string]string{
	"X-Content-Type-Options": "nosniff",
	"Referrer-Policy":        "no-referrer",
	"X-Frame-Options":        "DENY",
}

// edge runs before every route: a fresh trace, baseline security headers and a body cap.
func edge(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		// A client's trace context is untrusted, so every request starts a new trace.
		r.Header.Del("Traceparent")
		r.Header.Del("Tracestate")
		ctx, span := tracer.Start(r.Context(), r.Method, trace.WithSpanKind(trace.SpanKindServer))
		defer span.End()
		r = r.WithContext(ctx)
		otel.GetTextMapPropagator().Inject(ctx, propagation.HeaderCarrier(r.Header))

		for name, value := range securityHeaders {
			w.Header().Set(name, value)
		}
		metrics := httpsnoop.CaptureMetrics(capBody(next), w, r)
		recordRoute(span, r, metrics.Code)
	})
}

func capBody(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.ContentLength > maxBodyBytes {
			bff.Problem(w, http.StatusRequestEntityTooLarge, "request body is too large")
			return
		}
		r.Body = http.MaxBytesReader(w, r.Body, maxBodyBytes)
		next.ServeHTTP(w, r)
	})
}

// recordRoute names the span after the matched route, never the raw URL, and records no client IP or headers.
func recordRoute(span trace.Span, r *http.Request, status int) {
	span.SetName(cmp.Or(r.Pattern, r.Method))
	span.SetAttributes(
		attribute.String("http.request.method", r.Method),
		attribute.String("http.route", r.Pattern),
		attribute.Int("http.response.status_code", status),
	)
	if status >= http.StatusInternalServerError {
		span.SetStatus(codes.Error, http.StatusText(status))
	}
}
