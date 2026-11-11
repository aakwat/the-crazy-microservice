package main

import (
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func TestLoadConfigReadsEnvAndSecretFiles(t *testing.T) {
	dir := t.TempDir()
	writeSecret(t, dir, "GATEWAY_CLIENT_SECRET", "client-secret\n")
	writeSecret(t, dir, "GATEWAY_REDIS_PASSWORD", "redis-password")
	env := map[string]string{
		"SECRETS_DIR":              dir,
		"GATEWAY_PUBLIC_URL":       "http://127.0.0.1:8080/",
		"AUTH_ISSUER":              "http://127.0.0.1:8080",
		"AUTH_INTERNAL_URL":        "http://auth:8080",
		"TICKETS_INTERNAL_URL":     "http://tickets:8080",
		"GATEWAY_CLIENT_ID":        "gateway",
		"GATEWAY_REDIS_ADDR":       "gateway-redis:6379",
		"GATEWAY_REDIS_USER":       "gateway",
		"GATEWAY_AFTER_LOGIN_PATH": "/",
	}

	cfg, err := loadConfig(func(k string) string { return env[k] })

	if err != nil {
		t.Fatalf("loadConfig() error = %v", err)
	}
	if cfg.bff.ClientSecret != "client-secret" || cfg.redisPassword != "redis-password" {
		t.Errorf("secrets = %q, %q; want them read from files and trimmed", cfg.bff.ClientSecret, cfg.redisPassword)
	}
	if cfg.bff.PublicURL != "http://127.0.0.1:8080" || cfg.listen != ":8080" {
		t.Errorf("PublicURL, listen = %q, %q; want the trailing slash trimmed and :8080", cfg.bff.PublicURL, cfg.listen)
	}
	if cfg.bff.AfterLoginPath != "/" || cfg.bff.AfterLogoutPath != "/bff/logged-out" {
		t.Errorf("landing paths = %q, %q; want the override and the default", cfg.bff.AfterLoginPath, cfg.bff.AfterLogoutPath)
	}
}

func TestLoadConfigReportsEveryProblemAtOnce(t *testing.T) {
	env := map[string]string{
		"SECRETS_DIR":               t.TempDir(),
		"AUTH_INTERNAL_URL":         "auth:8080",
		"GATEWAY_AFTER_LOGOUT_PATH": "//evil.test",
	}

	_, err := loadConfig(func(k string) string { return env[k] })

	if err == nil {
		t.Fatal("loadConfig() error = nil, want one listing every problem")
	}
	for _, want := range []string{
		"GATEWAY_PUBLIC_URL is not set",
		"AUTH_INTERNAL_URL is not an absolute http(s) URL",
		"GATEWAY_CLIENT_SECRET is missing",
		"GATEWAY_AFTER_LOGOUT_PATH must be a local path",
	} {
		if !strings.Contains(err.Error(), want) {
			t.Errorf("loadConfig() error does not mention %q:\n%v", want, err)
		}
	}
}

func writeSecret(t *testing.T, dir, name, value string) {
	t.Helper()
	if err := os.WriteFile(filepath.Join(dir, name), []byte(value), 0o600); err != nil {
		t.Fatal(err)
	}
}
