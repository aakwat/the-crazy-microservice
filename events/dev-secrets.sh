#!/bin/sh
# Generates this plugin's local development secrets; never overwrites existing ones.
set -eu
cd "$(dirname "$0")"
umask 077
mkdir -p secrets

for name in EVENTS_REDIS_TICKETS_PASSWORD EVENTS_REDIS_OPS_PASSWORD EVENTS_REDIS_NOTIFICATIONS_PASSWORD; do
  [ -f "secrets/$name" ] || openssl rand -hex 32 | tr -d '\n' >"secrets/$name"
done

# Keeps other host users out, since the files themselves must be world-readable.
chmod 700 secrets
# Rootless containers see these files as root-owned, so the non-root service users need read access.
chmod 644 secrets/*
echo "secrets ready in $(pwd)/secrets"
