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
| tickets  | Rust   | tickets in Postgres, outbox table and its relay        |
| image    | Go     | attachment thumbnails, objects in MinIO                |
| realtime | Node   | WebSocket push of ticket events                        |
| jobs     | Python | scheduled work: SLA escalation, auto-close             |
| ai       | Python | auto-triage of new tickets via a local model (Ollama)  |

Build order, one scope at a time: auth, gateway, tickets, outbox relay,
realtime, image, jobs, ai, k8s.

### Contracts between services

- Auth signs JWTs with a private key and publishes `/.well-known/jwks.json`.
  The gateway verifies locally; no per-request call to auth.
- The gateway strips any inbound `X-User-Id` and sets its own from the token.
  Services trust that header only because nothing but the gateway reaches them.
- Sync traffic is JSON over HTTP. No gRPC: there is no service-to-service sync
  call that needs it.
- Async traffic is Redis Streams with consumer groups (polyglot; BullMQ is
  Node-only). Tickets writes events to its outbox in the same transaction as
  the change; the relay claims rows with `FOR UPDATE SKIP LOCKED` so replicas
  never double-publish. Consumers must be idempotent: delivery is at-least-once.
- Realtime fans out across replicas via Redis pub/sub.
- Image decoding treats input as hostile: check dimensions with
  `image.DecodeConfig` before decoding.
- AI output is untrusted (ticket text can carry prompt injection). It may only
  set allow-listed values (category enum, priority 1-4), validated before the
  write; it never triggers actions or writes free text.

## Commands

Toolchains come from the flake: `direnv allow` (or `nix develop`). Format
TS/JSON/Markdown/TOML/YAML with `dprint fmt`. Per-service commands are added
here as each service lands.

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
