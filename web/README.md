# web

The browser UI (Next.js). It is served only through the gateway, holds no session or token, and calls the
gateway with the browser's gateway cookie from server actions.

`src/app/` route shells, `src/modules/<name>/{domain,application,infrastructure,components}`, `src/shared/`.
A new module starts as a copy of `tickets`.

```fish
pnpm install
pnpm typecheck; pnpm lint; pnpm test
pnpm contracts          # regenerate API types from ../contracts/openapi
docker compose up -d --build --wait web
```
