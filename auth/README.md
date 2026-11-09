# auth

OAuth 2.1 / OIDC authorization server (Spring Authorization Server) with its own Postgres.

## Endpoints

| Path                                        | What                     |
| ------------------------------------------- | ------------------------ |
| `GET /.well-known/openid-configuration`     | discovery                |
| `GET /oauth2/jwks`                          | signing keys             |
| `GET /oauth2/authorize`, `/login`           | browser login            |
| `POST /oauth2/token`                        | tokens                   |
| `POST /oauth2/revoke`                       | revocation (RFC 7009)    |
| `POST /oauth2/introspect`                   | introspection (RFC 7662) |
| `GET /userinfo`                             | user info                |
| `/connect/logout`                           | logout                   |
| `POST /auth/register`                       | create an account        |
| `GET /actuator/health/{liveness,readiness}` | probes                   |

## Configuration

| Variable                    | Default    |
| --------------------------- | ---------- |
| `AUTH_ISSUER`               | required   |
| `AUTH_DB_URL`               | required   |
| `AUTH_DB_APP_USER`          | required   |
| `AUTH_DB_MIGRATOR_USER`     | required   |
| `AUTH_PRIVATE_KEY_PATH`     | required   |
| `AUTH_PUBLIC_KEY_PATH`      | required   |
| `AUTH_AUDIENCE`             | `crazy-ms` |
| `AUTH_ACCESS_TOKEN_TTL`     | `15m`      |
| `AUTH_REFRESH_TOKEN_TTL`    | `8h`       |
| `AUTH_DEFAULT_ROLE`         | `customer` |
| `AUTH_LOCKOUT_MAX_FAILURES` | `5`        |
| `AUTH_LOCKOUT_DURATION`     | `15m`      |
| `AUTH_ATTEMPT_RETENTION`    | `90d`      |

Secret files in `/run/secrets`: `AUTH_DB_APP_PASSWORD`, `AUTH_DB_MIGRATOR_PASSWORD`, `AUTH_CLIENT_<ID>_SECRET`.
OAuth clients: `config/clients.yaml`.

## Run

```fish
./dev-secrets.sh
docker compose up -d --build --wait
docker compose down -v
```

## Test, format, scan

```fish
mvn clean verify
mvn spotless:apply
mvn dependency-check:check
```
