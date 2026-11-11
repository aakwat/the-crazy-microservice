FROM docker.io/library/rust:1.99-slim-trixie@sha256:f6c4a35d21dbb3c89d7a8d056c193e94fc09037f2b58dd858b09c39a47dd02d7 AS build
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY domain domain
COPY application application
COPY adapters adapters
COPY api api
# target/ lives in a cache mount, so the binary is copied out in the same step.
RUN --mount=type=cache,target=/usr/local/cargo/registry --mount=type=cache,target=/src/target \
    cargo build --release --locked -p api && cp target/release/tickets /tickets

FROM gcr.io/distroless/cc-debian13:nonroot@sha256:cc66d222f0a7ec996b78a99fe76a969bc80b539a5016ad866c14d1949bc6d0ab
LABEL org.opencontainers.image.title="crazy-ms-tickets" \
      org.opencontainers.image.description="Tickets, their status and the outbox of ticket events"
COPY --from=build /tickets /tickets
USER nonroot:nonroot
EXPOSE 8080
ENTRYPOINT ["/tickets"]
