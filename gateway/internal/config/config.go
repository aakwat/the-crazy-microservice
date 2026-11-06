// Package config reads the gateway's settings from env and its secrets from files, never from env.
package config

import (
	"fmt"
	"net/url"
	"os"
	"path/filepath"
	"strings"
)

const (
	defaultListenAddr = ":8080"
	defaultSecretsDir = "/run/secrets"
)

// Config is everything that differs between deployments; fixed values live as constants in main.
type Config struct {
	ListenAddr    string
	PublicURL     string
	Issuer        string
	AuthURL       string
	ClientID      string
	ClientSecret  string
	RedisAddr     string
	RedisUser     string
	RedisPassword string
}

// Load reports every missing or malformed setting at once, so a misconfigured replica never starts.
func Load(getenv func(string) string) (Config, error) {
	var problems []string
	need := func(name string) string {
		v := getenv(name)
		if v == "" {
			problems = append(problems, name+" is not set")
		}
		return v
	}
	needURL := func(name string) string {
		v := need(name)
		if u, err := url.Parse(v); v != "" && (err != nil || u.Scheme == "" || u.Host == "") {
			problems = append(problems, name+" is not an absolute URL")
		}
		return strings.TrimSuffix(v, "/")
	}
	secretsDir := getenv("SECRETS_DIR")
	if secretsDir == "" {
		secretsDir = defaultSecretsDir
	}
	secret := func(name string) string {
		path := filepath.Join(secretsDir, name)
		b, err := os.ReadFile(path)
		v := strings.TrimSpace(string(b))
		if err != nil || v == "" {
			problems = append(problems, path+" is missing or empty")
		}
		return v
	}

	cfg := Config{
		ListenAddr:    ListenAddr(getenv),
		PublicURL:     needURL("GATEWAY_PUBLIC_URL"),
		Issuer:        needURL("AUTH_ISSUER"),
		AuthURL:       needURL("AUTH_INTERNAL_URL"),
		ClientID:      need("GATEWAY_CLIENT_ID"),
		ClientSecret:  secret("GATEWAY_CLIENT_SECRET"),
		RedisAddr:     need("GATEWAY_REDIS_ADDR"),
		RedisUser:     need("GATEWAY_REDIS_USER"),
		RedisPassword: secret("GATEWAY_REDIS_PASSWORD"),
	}
	if len(problems) > 0 {
		return Config{}, fmt.Errorf("invalid configuration: %s", strings.Join(problems, "; "))
	}
	return cfg, nil
}

// ListenAddr is shared with the healthcheck, which must probe the port the server listens on.
func ListenAddr(getenv func(string) string) string {
	if v := getenv("GATEWAY_LISTEN_ADDR"); v != "" {
		return v
	}
	return defaultListenAddr
}
