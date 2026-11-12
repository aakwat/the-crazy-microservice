#!/bin/sh
# Generates local development secrets for this plugin; never overwrites existing ones.
# TICKETS_EVENTS_REDIS_PASSWORD is not made here: it must match the user the event bus creates.
set -eu
cd "$(dirname "$0")"
umask 077
mkdir -p secrets

for name in TICKETS_DB_SUPERUSER_PASSWORD TICKETS_DB_MIGRATOR_PASSWORD TICKETS_DB_APP_PASSWORD; do
    [ -f "secrets/$name" ] || openssl rand -hex 32 | tr -d '\n' >"secrets/$name"
done

# Keeps other host users out, since the files themselves must be world-readable.
chmod 700 secrets
# Rootless containers see these files as root-owned, so the non-root service user needs read access.
chmod 644 secrets/*
echo "secrets ready in $(pwd)/secrets"
