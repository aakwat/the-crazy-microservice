# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

Rules for agents working in this repo. If a rule needs a paragraph to justify
itself, it is probably wrong.

## Project

Polyglot support-ticket PoC. One public gateway; every other service is
private (ClusterIP + default-deny NetworkPolicy on k3d). Every service is
stateless so it can run as several replicas.

| Service  | Lang   | Owns                                                   |
| -------- | ------ | ------------------------------------------------------ |
| gateway  | Go     | routing, JWT verify, WebSocket upgrade proxy           |
| auth     | Java   | users, password hashing, JWT signing, JWKS             |
| tickets  | Rust   | tickets, attachment records, originals bucket, outbox  |
| image    | Go     | thumbs bucket; worker only, no HTTP route              |
| realtime | Node   | live sockets; pushes "ticket changed" notifications    |
| jobs     | Python | scheduled work: SLA escalation, auto-close             |
| ai       | Python | auto-triage of new tickets via a local model (Ollama)  |

Build order, one scope at a time: auth, gateway, tickets, outbox relay,
realtime, image, jobs, ai, k8s.

Layers: L1 edge (Traefik TLS, gateway), L2 request services (auth, tickets,
realtime), L3 messaging (Redis Streams), L4 workers (image, ai, jobs), L5 data
(Postgres per service, MinIO, Ollama). Inside every service: transport → app
→ domain ← adapters; the domain imports no framework, DB or transport code.
Diagrams: `docs/architecture.html`; change it with the contracts below.

### Contracts between services

Three channels, never mixed. Schemas live in `contracts/openapi/` (HTTP) and
`contracts/events/` (JSON Schema); they are the source of truth, not the code.

**Identity (zero trust).** Auth signs ES256 JWTs (15 min) and publishes
`/.well-known/jwks.json`. `POST /auth/login` returns only the RFC 6749 §5.1
JSON body (`access_token`, `token_type`, `expires_in`, `no-store`); auth never
sets cookies. The gateway owns the browser cookie (sets it, maps it to
`Authorization` upstream, clears it on logout). Every service verifies the JWT
against cached JWKS (refetch on unknown `kid`); none trusts an identity
header. Internal callers (jobs, ai) use a service token issued by auth.

**Sync HTTP (JSON), kept minimal.** Only: gateway → auth, tickets (proxy) and
→ realtime (WS), services → auth JWKS, jobs and ai → tickets `GET /internal/*`
(reads only), ai → Ollama. Permission checks live in the owning service. The
gateway returns 404 for `/internal/*`, caps bodies, sets timeouts, and never
retries POST. Discovery is DNS (compose / k8s Service names). No gRPC.

**Events (Redis Streams).** Envelope:
`{id: uuidv7, type, version, occurred_at: RFC3339, subject_id, traceparent, payload}`.

| Stream               | Producer         | Consumers               |
| -------------------- | ---------------- | ----------------------- |
| `tickets.events`     | tickets (outbox) | realtime, ai, image     |
| `attachments.events` | image            | tickets                 |
| `triage.events`      | ai               | tickets                 |
| `jobs.commands`      | jobs             | tickets                 |

- Events are thin: IDs, type, status, `requester_id`, `assignee_id`. Never
  ticket text or other personal data (GDPR minimisation); fetch it from
  tickets `/internal` if needed.
- Only tickets writes ticket data; other services change it by event.
- Uploads go to tickets (permission check, original to MinIO, outbox
  `attachment.uploaded`); image consumes that and never serves HTTP.
- Only tickets has an outbox: it is the only DB-write-plus-publish. The relay
  claims rows with `FOR UPDATE SKIP LOCKED` so replicas never double-publish.
- Workers use consumer groups. Delivery is at-least-once, so every consumer
  dedupes on envelope `id`. `XAUTOCLAIM` stuck messages; after 5 deliveries move
  to `<stream>.dlq`. Trim with `MAXLEN ~`.
- Realtime does NOT use a consumer group: every replica must see every event,
  so each replica `XREAD BLOCK`s from `$`. It pushes only "ticket {id}
  changed" to the requester and assignee; the client refetches via REST, so
  authorisation stays in tickets. WS auth is the gateway's httpOnly cookie on
  upgrade, never a query token.

**Other invariants.**

- `traceparent` on every HTTP call and in every envelope.
- Image decoding treats input as hostile: check dimensions with
  `image.DecodeConfig` before decoding.
- AI output is untrusted (ticket text can carry prompt injection). It may only
  set allow-listed values (category enum, priority 1-4), validated before the
  write; it never triggers actions or writes free text.

### Services are plugins

Each service directory is self-contained and reusable in another project:
its own `compose.yaml` (service + its own Postgres), `Dockerfile`,
`dev-secrets.sh`, env-only config, and standard contracts (JWKS, RFC 6749,
RFC 9457). The root `compose.yaml` only `include:`s them; nothing in a plugin
may reference another plugin's internals.

### Inbound / outbound (compose networks now, NetworkPolicy later)

All networks are `internal: true` (no internet egress); only the gateway will
publish a port. Secrets are files under `/run/secrets`, never env vars
(Spring reads them via `SPRING_CONFIG_IMPORT=configtree:/run/secrets/`,
Postgres via `*_FILE`).

| Service | Inbound from        | Outbound to |
| ------- | ------------------- | ----------- |
| auth    | `app` network (8080) | auth-db     |
| auth-db | auth only           | none        |
| client  | none (tools profile) | `app`       |

### Database rules

Postgres 18, one database per service. Two roles: `<svc>_owner` runs Flyway
(DDL), `<svc>_app` is the runtime user with only select/insert/update/delete
(granted by `db/init/01-roles.sh` default privileges). Ids are UUIDv7 assigned
in the domain, with `default uuidv7()` as a fallback for other writers.
Invariants are also DB constraints. Hibernate runs with `ddl-auto=validate`.

## Commands

Toolchains come from the flake: `direnv allow` (or `nix develop`); it also
points Testcontainers at rootless podman. Format with `dprint fmt`.

```fish
./auth/dev-secrets.sh                               # once: keys + DB passwords in auth/secrets/
docker compose up -d --build --wait                 # start every included plugin
docker compose run --rm client -s http://auth:8080/.well-known/jwks.json
docker compose down -v                              # stop and wipe volumes
cd auth; mvn verify                                 # unit + Testcontainers tests (needs podman socket)
cd auth; mvn test -Dtest=AuthFlowTest#duplicateEmailInAnyCaseIsConflict
```

Auth layout: feature modules `account/` and `token/`, each
`domain/ service/ repository/ web/` (`repository/jpa` holds entities and
Spring Data interfaces, `web/dto` the wire records). Domain classes import
nothing from Spring or JPA. No Lombok: records cover DTOs and domain.

## Working agreement

- Verify before claiming. Read the file, run the command, quote the output.
  Never report something as working that you have not seen work.
- Do not run builds, test suites or migrations unprompted. Name the command and
  let me run it.
- State the blast radius before editing: which files, which callers.
- Finish the task you were given. If you find a second problem, name it; do not
  silently widen scope.
- Ambiguity that changes the work is worth one question. Ambiguity that does not
  is worth a default and a sentence saying which you picked.
- Do not trust existing code or comments. They were right once.

## Comments

- One line. If it needs three, fix the code instead.
- Explain why, never what. `// increment i` is noise.
- One comment, one reason. Do not staple two unrelated facts together.
- Comment the non-obvious: a workaround, an ordering requirement, a constraint
  that is not visible locally, a deliberate omission.
- No commented-out code. Git remembers it.
- A stale comment is worse than none. Change it with the code or delete it.

## Commits

- One-line subject, imperative, no trailing period.
- No body unless the change is genuinely non-obvious. No trailers, no
  Co-Authored-By, no emoji.
- One logical change per commit. Use `git commit -- <path>` rather than sweeping
  in unrelated files.
- Never commit working-tree changes you did not make.

## Structure

- Single responsibility at every level: function, module, layer.
- No single large file. Split a file when it starts doing two things, not after
  it does four.
- Layer by layer. Dependencies point inward: the core knows nothing about
  transport, storage or frameworks.
- Module by module. A module exposes a small surface and hides the rest.
- Before adding an abstraction, say what breaks without it. One implementation
  needs no interface; a value that never changes needs no config.
- Deleting code is a valid change, and usually the better one.
- Shortest correct change in the right place. A small diff in the wrong place is
  a second bug.
- Re-think before you build: is this worth its own file, layer, or dependency?
  Most of the time the answer is no.

## Java / Spring Boot

### Layering

```
domain      entities and invariants; no Spring annotations, no JPA, no DTOs
service     use cases; transactional boundary lives here
repository  persistence ports; MyBatis mappers or JPA repos behind them
web         controllers; HTTP in, DTO out, nothing else
```

A controller that calls a repository has skipped a layer. A domain class that
imports `org.springframework` has leaked one.

### Spring specifics

- Constructor injection only. No `@Autowired` on fields — it hides the
  dependency list and makes the class untestable without a container.
- `@Transactional` on the service method, never on the controller or the
  repository. Know that it is a proxy: a self-call inside the same bean does not
  open a transaction.
- DTOs at the HTTP boundary. Never serialise an entity straight out; it couples
  your wire format to your schema and leaks fields you did not mean to expose.
- Validate with `@Valid` at the controller edge. The service may assume valid
  input only because something else guaranteed it.
- No business logic in `@Component` singletons that hold state. They are shared.

### Style

- `google-java-format` or the Spring conventions, applied by the build, not by
  hand. Pick one and let it be non-negotiable.
- Package by feature, not by layer-at-the-top. `order/` containing its
  controller, service and repo beats four packages each holding a slice of
  every feature.
- `Optional` for return values that may be absent. Never for fields or
  parameters.
- Checked exceptions only where the caller can genuinely act on them.
