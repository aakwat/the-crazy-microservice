# the-crazy-microservice

Polyglot support-ticket PoC: one public gateway, every other service private.

| Service                      | Language | Status              |
| ---------------------------- | -------- | ------------------- |
| [auth](auth/README.md)       | Java     | done                |
| [gateway](gateway/README.md) | Go       | running, login next |
| tickets                      | Rust     | planned             |
| realtime                     | Node     | planned             |
| image                        | Go       | planned             |
| jobs                         | Python   | planned             |
| ai                           | Python   | planned             |

Rules and contracts: [CLAUDE.md](CLAUDE.md). Diagrams: [overview](docs/overview.html), [detail](docs/architecture.html).

## Requirements

Nix (flakes) with direnv, rootless Podman.

## Run

```fish
direnv allow
./dev-secrets.sh
docker compose up -d --build --wait
docker compose run --rm client -s http://auth:8080/.well-known/openid-configuration
docker compose down -v
```

Portal `http://127.0.0.1:8080/`, mail `http://127.0.0.1:8025/`, traces `http://127.0.0.1:16686/` (all dev only).

## Dev accounts

`./dev-seed.sh`, run once the stack is up, creates one account per role. Password for all: `Password@1234`.

| Role     | Email                 | Can                                                                     |
| -------- | --------------------- | ----------------------------------------------------------------------- |
| Admin    | `admin@example.test`  | change roles (`/auth/accounts`); tickets treats admins as customers yet |
| Agent    | `me@example.test`     | see every ticket; reply, note; take one, then ask or resolve it         |
| Customer | `helper@example.test` | open tickets, see own, reply, confirm, withdraw                         |

## Format

```fish
dprint fmt
```
