FROM docker.io/library/node:24.22.3-alpine@sha256:116b800c3754b440ee0aa3ed2f20f5bf3d99a554bfd9adeea09ea8017f6e85f5 AS build
WORKDIR /src
RUN corepack enable
COPY package.json pnpm-lock.yaml pnpm-workspace.yaml ./
RUN --mount=type=cache,target=/root/.local/share/pnpm/store pnpm install --frozen-lockfile
COPY . .
# Env is read at runtime; the build only needs to skip the check.
RUN SKIP_ENV_VALIDATION=1 NEXT_TELEMETRY_DISABLED=1 pnpm build

FROM gcr.io/distroless/nodejs24-debian13:nonroot@sha256:6abfcb367ebc291572c774c1592a986be2ab26a2486f73093401327495a5962a
LABEL org.opencontainers.image.title="crazy-ms-web" \
      org.opencontainers.image.description="Browser UI, served only through the gateway"
WORKDIR /app
COPY --from=build /src/.next/standalone ./
COPY --from=build /src/.next/static ./.next/static
ENV NODE_ENV=production HOSTNAME=0.0.0.0 PORT=3000 NEXT_TELEMETRY_DISABLED=1
USER nonroot:nonroot
EXPOSE 3000
CMD ["server.js"]
