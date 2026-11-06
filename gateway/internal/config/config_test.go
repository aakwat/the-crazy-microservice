package config

import (
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func TestLoadReadsEnvAndSecretFiles(t *testing.T) {
	dir := t.TempDir()
	writeSecret(t, dir, "GATEWAY_CLIENT_SECRET", "client-secret\n")
	writeSecret(t, dir, "GATEWAY_REDIS_PASSWORD", "redis-password")
	env := map[string]string{
		"SECRETS_DIR":        dir,
		"GATEWAY_PUBLIC_URL": "http://localhost:8080/",
		"AUTH_ISSUER":        "http://localhost:8080",
		"AUTH_INTERNAL_URL":  "http://auth:8080",
		"GATEWAY_CLIENT_ID":  "gateway",
		"GATEWAY_REDIS_ADDR": "gateway-redis:6379",
		"GATEWAY_REDIS_USER": "gateway",
	}

	cfg, err := Load(func(k string) string { return env[k] })
	if err != nil {
		t.Fatal(err)
	}
	if cfg.ClientSecret != "client-secret" || cfg.RedisPassword != "redis-password" {
		t.Fatalf("secrets must come from files, trimmed: %+v", cfg)
	}
	if cfg.PublicURL != "http://localhost:8080" || cfg.ListenAddr != ":8080" {
		t.Fatalf("want trailing slash trimmed and default listen address, got %+v", cfg)
	}
}

func TestLoadReportsEveryProblemAtOnce(t *testing.T) {
	env := map[string]string{"SECRETS_DIR": t.TempDir(), "AUTH_INTERNAL_URL": "auth:8080"}

	_, err := Load(func(k string) string { return env[k] })

	if err == nil {
		t.Fatal("want an error")
	}
	for _, want := range []string{"GATEWAY_PUBLIC_URL is not set", "AUTH_INTERNAL_URL is not an absolute URL",
		"GATEWAY_CLIENT_SECRET is missing", "GATEWAY_REDIS_USER is not set"} {
		if !strings.Contains(err.Error(), want) {
			t.Errorf("error does not mention %q: %v", want, err)
		}
	}
}

func writeSecret(t *testing.T, dir, name, value string) {
	t.Helper()
	if err := os.WriteFile(filepath.Join(dir, name), []byte(value), 0o600); err != nil {
		t.Fatal(err)
	}
}
