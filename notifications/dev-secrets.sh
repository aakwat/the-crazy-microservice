#!/bin/sh
# Generates local development secrets for this plugin; never overwrites existing ones.
# The auth client secret and bus password must match auth and the bus, so the root script copies them in.
set -eu
cd "$(dirname "$0")"
umask 077
mkdir -p secrets

for name in NOTIFICATIONS_DB_SUPERUSER_PASSWORD NOTIFICATIONS_DB_MIGRATOR_PASSWORD NOTIFICATIONS_DB_APP_PASSWORD; do
  [ -f "secrets/$name" ] || openssl rand -hex 32 | tr -d '\n' >"secrets/$name"
done

# Keeps other host users out, since the files themselves must be world-readable.
chmod 700 secrets
# Rootless containers see these files as root-owned, so the non-root service users need read access.
chmod 644 secrets/*
echo "secrets ready in $(pwd)/secrets"
