#!/bin/sh
# Generates this plugin's local development secrets; never overwrites existing ones.
# GATEWAY_CLIENT_SECRET is not made here: it must match the client registered at the authorization server.
set -eu
cd "$(dirname "$0")"
umask 077
mkdir -p secrets

[ -f secrets/GATEWAY_REDIS_PASSWORD ] || openssl rand -hex 32 | tr -d '\n' >secrets/GATEWAY_REDIS_PASSWORD

# Keeps other host users out, since the files themselves must be world-readable.
chmod 700 secrets
# Rootless containers see these files as root-owned, so the non-root service users need read access.
chmod 644 secrets/*
echo "secrets ready in $(pwd)/secrets"
