package main

import (
	"context"
	"os"
	"testing"
)

// TestMain installs the real tracer, without an exporter, so the edge hands upstreams a traceparent as in production.
func TestMain(m *testing.M) {
	if _, err := setupTracing(context.Background()); err != nil {
		panic(err)
	}
	os.Exit(m.Run())
}
