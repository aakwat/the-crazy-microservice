# tickets

Support tickets and their status, in Rust, with its own Postgres.

## Job

- Owns tickets: the only service that writes ticket data; others change it by event.
- Decides who may see or change a ticket (requester or agent).
- Records every change as an outbox event in the same transaction; the relay publishes them.
- Never trusts an identity header: it verifies each access token against auth's keys.

## Layout

| Crate               | Holds                                                    |
| ------------------- | -------------------------------------------------------- |
| `domain`            | ticket rules and ports; no I/O, no framework             |
| `application`       | use cases and permission checks                          |
| `adapters/postgres` | repository, outbox, migrations                           |
| `adapters/jwks`     | access-token verification (RFC 9068) against auth's JWKS |
| `adapters/streams`  | publishes outbox events to Redis Streams                 |
| `api`               | HTTP routes, config, the outbox relay job, `main`        |

The relay runs in every replica; `SKIP LOCKED` keeps them from sending the same
event twice. `/healthz` turns 503 when the relay has not finished a round for
60 s.

## Run alone

```fish
./dev-secrets.sh
docker compose up -d --build --wait
```

Needs `AUTH_ISSUER` and `AUTH_JWKS_URL` pointing at an OIDC server, and
`TICKETS_EVENTS_REDIS_HOST`/`_USER` plus the `TICKETS_EVENTS_REDIS_PASSWORD` secret for the event bus. Contracts:
`contracts/openapi/tickets.yaml`, `contracts/events/tickets.events.schema.json`.

## Test, format, lint

```fish
cargo test --workspace
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
```

The Postgres tests start a real Postgres 18 with Testcontainers and need a container socket.
