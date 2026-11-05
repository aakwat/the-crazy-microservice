#!/bin/sh
# Generates local development secrets for this plugin; never overwrites existing ones.
set -eu
cd "$(dirname "$0")"
umask 077
mkdir -p secrets

[ -f secrets/auth-private.pem ] ||
    openssl genpkey -algorithm EC -pkeyopt ec_paramgen_curve:P-256 -out secrets/auth-private.pem
[ -f secrets/auth-public.pem ] ||
    openssl pkey -in secrets/auth-private.pem -pubout -out secrets/auth-public.pem

for name in AUTH_DB_OWNER_PASSWORD AUTH_DB_APP_PASSWORD; do
    [ -f "secrets/$name" ] || openssl rand -hex 32 | tr -d '\n' >"secrets/$name"
done

# Rootless containers see these files as root-owned, so the non-root service user needs read access;
# the 700 directory keeps other host users out.
chmod 700 secrets
chmod 644 secrets/*
echo "secrets ready in $(pwd)/secrets"
