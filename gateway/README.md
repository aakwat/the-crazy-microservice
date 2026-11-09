# gateway

Public entry point: browser login (BFF), sessions in Redis, proxy to private services.

## Routes

| Route                                                                                                     | What            |
| --------------------------------------------------------------------------------------------------------- | --------------- |
| `GET /bff/login`                                                                                          | start login     |
| `GET /bff/callback`                                                                                       | finish login    |
| `GET /bff/logout`                                                                                         | logout form     |
| `POST /bff/logout`                                                                                        | logout          |
| `GET /bff/logged-out`                                                                                     | after logout    |
| `GET /bff/user`                                                                                           | current user    |
| `/login`, `/oauth2/authorize`, `/connect/logout`, `/.well-known/*`, `/oauth2/jwks`, `POST /auth/register` | proxied to auth |
| `GET /healthz`                                                                                            | liveness        |

## Configuration

| Variable                    | Default           |
| --------------------------- | ----------------- |
| `GATEWAY_PUBLIC_URL`        | required          |
| `AUTH_ISSUER`               | required          |
| `AUTH_INTERNAL_URL`         | required          |
| `GATEWAY_CLIENT_ID`         | required          |
| `GATEWAY_REDIS_ADDR`        | required          |
| `GATEWAY_REDIS_USER`        | required          |
| `GATEWAY_LISTEN_ADDR`       | `:8080`           |
| `GATEWAY_AFTER_LOGIN_PATH`  | `/bff/user`       |
| `GATEWAY_AFTER_LOGOUT_PATH` | `/bff/logged-out` |
| `SECRETS_DIR`               | `/run/secrets`    |

Secret files: `GATEWAY_CLIENT_SECRET`, `GATEWAY_REDIS_PASSWORD`.

## Run

```fish
./dev-secrets.sh
go run ./cmd/gateway
```

In the stack: `docker compose up -d --build --wait` from the repo root.

## Test, format, lint

```fish
go test -race ./...
golangci-lint run ./...
golangci-lint fmt ./...
go run golang.org/x/vuln/cmd/govulncheck@latest ./...
```
