// Command gateway is the only public entry point: browser login (BFF), token mapping and a proxy to the services.
package main

import (
	"cmp"
	"context"
	"fmt"
	"log/slog"
	"net"
	"net/http"
	"net/url"
	"os"
	"os/signal"
	"syscall"
	"time"

	"github.com/aakwat/the-crazy-microservice/gateway/internal/bff"
	"github.com/redis/go-redis/v9"
)

const (
	// audience is the aud every service checks in access tokens (CLAUDE.md, identity).
	audience          = "crazy-ms"
	defaultListen     = ":8080"
	defaultSecretsDir = "/run/secrets"
	maxBodyBytes      = 1 << 20
	upstreamTimeout   = 15 * time.Second
	authCallTimeout   = 10 * time.Second
	shutdownGrace     = 10 * time.Second
	registerPerMin    = 5
	loginPerMin       = 20
)

func main() {
	if len(os.Args) > 1 && os.Args[1] == "healthcheck" {
		os.Exit(healthcheck())
	}
	log := slog.New(slog.NewJSONHandler(os.Stdout, nil))
	if err := run(log); err != nil {
		log.Error("gateway stopped", "err", err)
		os.Exit(1)
	}
}

func run(log *slog.Logger) error {
	cfg, err := loadConfig(os.Getenv)
	if err != nil {
		return err
	}
	authURL, err := url.Parse(cfg.bff.AuthURL)
	if err != nil {
		return err
	}
	ticketsURL, err := url.Parse(cfg.ticketsURL)
	if err != nil {
		return err
	}
	publicURL, err := url.Parse(cfg.bff.PublicURL)
	if err != nil {
		return err
	}
	ctx, stop := signal.NotifyContext(context.Background(), os.Interrupt, syscall.SIGTERM)
	defer stop()

	rdb := redis.NewClient(&redis.Options{Addr: cfg.redisAddr, Username: cfg.redisUser, Password: cfg.redisPassword})
	defer rdb.Close()
	if err := rdb.Ping(ctx).Err(); err != nil {
		return fmt.Errorf("connecting to redis: %w", err)
	}
	cfg.bff.HTTPClient = authClient()
	handler := routes(bff.New(ctx, cfg.bff, rdb, log), newProxy(authURL, publicURL, log), newProxy(ticketsURL, publicURL, log))

	srv := &http.Server{
		Addr:              cfg.listen,
		Handler:           handler,
		ReadHeaderTimeout: 5 * time.Second,
		ReadTimeout:       30 * time.Second,
		WriteTimeout:      60 * time.Second,
		IdleTimeout:       120 * time.Second,
		// The gateway's own cookies are small, so the 1 MiB default would only help header floods.
		MaxHeaderBytes: 32 << 10,
		ErrorLog:       slog.NewLogLogger(log.Handler(), slog.LevelWarn),
	}
	errc := make(chan error, 1)
	go func() { errc <- srv.ListenAndServe() }()
	log.Info("gateway listening", "addr", cfg.listen)

	select {
	case err := <-errc:
		return err
	case <-ctx.Done():
	}
	shutdownCtx, cancel := context.WithTimeout(context.Background(), shutdownGrace)
	defer cancel()
	return srv.Shutdown(shutdownCtx)
}

// authClient talks to auth directly over the private network, never through an HTTP_PROXY.
func authClient() *http.Client {
	transport := http.DefaultTransport.(*http.Transport).Clone()
	transport.Proxy = nil
	return &http.Client{Transport: transport, Timeout: authCallTimeout}
}

// healthcheck is the container probe; the distroless image has no curl, so the binary checks itself.
func healthcheck() int {
	_, port, err := net.SplitHostPort(listenAddr(os.Getenv))
	if err != nil {
		return 1
	}
	ctx, cancel := context.WithTimeout(context.Background(), 2*time.Second)
	defer cancel()
	req, err := http.NewRequestWithContext(ctx, http.MethodGet, "http://127.0.0.1:"+port+"/healthz", nil)
	if err != nil {
		return 1
	}
	res, err := http.DefaultClient.Do(req)
	if err != nil {
		return 1
	}
	_ = res.Body.Close()
	if res.StatusCode >= http.StatusMultipleChoices {
		return 1
	}
	return 0
}

// listenAddr is shared with healthcheck, which must probe the port the server listens on.
func listenAddr(getenv func(string) string) string {
	return cmp.Or(getenv("GATEWAY_LISTEN_ADDR"), defaultListen)
}
