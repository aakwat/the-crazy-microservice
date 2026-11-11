package main

import (
	"cmp"
	"fmt"
	"net/url"
	"os"
	"path/filepath"
	"strings"

	"github.com/aakwat/the-crazy-microservice/gateway/internal/bff"
)

// config is everything that differs between deployments; fixed values are constants in main.go.
type config struct {
	listen        string
	redisAddr     string
	redisUser     string
	redisPassword string
	ticketsURL    string
	bff           bff.Config
}

// loadConfig reads settings from env and secrets from files, never from env, and reports every problem at once.
func loadConfig(getenv func(string) string) (config, error) {
	l := &loader{getenv: getenv, secretsDir: cmp.Or(getenv("SECRETS_DIR"), defaultSecretsDir)}
	cfg := config{
		listen:        listenAddr(getenv),
		redisAddr:     l.required("GATEWAY_REDIS_ADDR"),
		redisUser:     l.required("GATEWAY_REDIS_USER"),
		redisPassword: l.secret("GATEWAY_REDIS_PASSWORD"),
		ticketsURL:    l.url("TICKETS_INTERNAL_URL"),
		bff: bff.Config{
			PublicURL:       l.url("GATEWAY_PUBLIC_URL"),
			Issuer:          l.url("AUTH_ISSUER"),
			AuthURL:         l.url("AUTH_INTERNAL_URL"),
			ClientID:        l.required("GATEWAY_CLIENT_ID"),
			ClientSecret:    l.secret("GATEWAY_CLIENT_SECRET"),
			Audience:        audience,
			AfterLoginPath:  l.localPath("GATEWAY_AFTER_LOGIN_PATH", "/bff/user"),
			AfterLogoutPath: l.localPath("GATEWAY_AFTER_LOGOUT_PATH", "/bff/logged-out"),
		},
	}
	if len(l.problems) > 0 {
		return config{}, fmt.Errorf("invalid configuration: %s", strings.Join(l.problems, "; "))
	}
	return cfg, nil
}

// loader collects every configuration problem instead of stopping at the first.
type loader struct {
	getenv     func(string) string
	secretsDir string
	problems   []string
}

func (l *loader) required(name string) string {
	v := l.getenv(name)
	if v == "" {
		l.problems = append(l.problems, name+" is not set")
	}
	return v
}

func (l *loader) url(name string) string {
	v := l.required(name)
	if u, err := url.Parse(v); v != "" && (err != nil || (u.Scheme != "http" && u.Scheme != "https") || u.Host == "") {
		l.problems = append(l.problems, name+" is not an absolute http(s) URL")
	}
	return strings.TrimSuffix(v, "/")
}

// localPath accepts only a path on the gateway's own origin, so a redirect can never point at another site.
func (l *loader) localPath(name, fallback string) string {
	v := cmp.Or(l.getenv(name), fallback)
	if !strings.HasPrefix(v, "/") || strings.HasPrefix(v, "//") || strings.Contains(v, `\`) {
		l.problems = append(l.problems, name+" must be a local path such as /home")
	}
	return v
}

func (l *loader) secret(name string) string {
	path := filepath.Join(l.secretsDir, name)
	data, err := os.ReadFile(path)
	v := strings.TrimSpace(string(data))
	if err != nil || v == "" {
		l.problems = append(l.problems, path+" is missing or empty")
	}
	return v
}
