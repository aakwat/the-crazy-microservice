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

## Format

```fish
dprint fmt
```
