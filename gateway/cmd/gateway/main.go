// Command gateway is the only public entry point: BFF login, token mapping and the proxy to private services.
package main

import (
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

	"github.com/aakwat/the-crazy-microservice/gateway/internal/adapters/oidc"
	"github.com/aakwat/the-crazy-microservice/gateway/internal/adapters/redisstore"
	"github.com/aakwat/the-crazy-microservice/gateway/internal/app"
	"github.com/aakwat/the-crazy-microservice/gateway/internal/config"
	"github.com/aakwat/the-crazy-microservice/gateway/internal/domain"
	"github.com/aakwat/the-crazy-microservice/gateway/internal/transport/bff"
	"github.com/aakwat/the-crazy-microservice/gateway/internal/transport/edge"
	"github.com/aakwat/the-crazy-microservice/gateway/internal/transport/proxy"
	"github.com/redis/go-redis/v9"
	"golang.org/x/time/rate"
)

const (
	// audience is the contract every service checks in access tokens (CLAUDE.md, identity).
	audience         = "crazy-ms"
	maxBodyBytes     = 1 << 20
	upstreamTimeout  = 15 * time.Second
	authCallTimeout  = 10 * time.Second
	shutdownGrace    = 10 * time.Second
	registerPerMin   = 5
	healthcheckLimit = 2 * time.Second
)

var lifetime = domain.Lifetime{Idle: 8 * time.Hour, Absolute: 12 * time.Hour}

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
	cfg, err := config.Load(os.Getenv)
	if err != nil {
		return err
	}
	authURL, err := url.Parse(cfg.AuthURL)
	if err != nil {
		return err
	}
	publicURL, err := url.Parse(cfg.PublicURL)
	if err != nil {
		return err
	}
	ctx, stop := signal.NotifyContext(context.Background(), os.Interrupt, syscall.SIGTERM)
	defer stop()

	rdb := redis.NewClient(&redis.Options{Addr: cfg.RedisAddr, Username: cfg.RedisUser, Password: cfg.RedisPassword})
	defer rdb.Close()
	if err := rdb.Ping(ctx).Err(); err != nil {
		return fmt.Errorf("redis: %w", err)
	}

	provider := oidc.New(ctx, oidc.Config{
		Issuer:        cfg.Issuer,
		InternalURL:   cfg.AuthURL,
		ClientID:      cfg.ClientID,
		ClientSecret:  cfg.ClientSecret,
		RedirectURL:   cfg.PublicURL + "/bff/callback",
		PostLogoutURL: cfg.PublicURL + "/bff/logged-out",
		Audience:      audience,
		HTTPClient:    authClient(),
	})
	svc := app.New(redisstore.New(rdb), provider, lifetime, time.Now)
	register := edge.NewPerIP(rate.Every(time.Minute/registerPerMin), registerPerMin)
	handler := routes(bff.New(svc, log), proxy.New(authURL, publicURL, upstreamTimeout, log), register)

	srv := &http.Server{
		Addr:              cfg.ListenAddr,
		Handler:           handler,
		ReadHeaderTimeout: 5 * time.Second,
		ReadTimeout:       30 * time.Second,
		WriteTimeout:      60 * time.Second,
		IdleTimeout:       120 * time.Second,
		ErrorLog:          slog.NewLogLogger(log.Handler(), slog.LevelWarn),
	}
	errc := make(chan error, 1)
	go func() { errc <- srv.ListenAndServe() }()
	log.Info("gateway listening", "addr", cfg.ListenAddr)

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
	_, port, err := net.SplitHostPort(config.ListenAddr(os.Getenv))
	if err != nil {
		return 1
	}
	client := http.Client{Timeout: healthcheckLimit}
	res, err := client.Get("http://127.0.0.1:" + port + "/healthz")
	if err != nil {
		return 1
	}
	_ = res.Body.Close()
	if res.StatusCode >= http.StatusMultipleChoices {
		return 1
	}
	return 0
}
