#!/bin/sh
# Project wiring: each plugin makes its own secrets, then values two plugins share are copied across.
set -eu
cd "$(dirname "$0")"
./auth/dev-secrets.sh
./gateway/dev-secrets.sh

# The gateway's OAuth client secret is issued by auth's registration, so both must hold the same value.
cp auth/secrets/AUTH_CLIENT_GATEWAY_SECRET gateway/secrets/GATEWAY_CLIENT_SECRET
chmod 644 gateway/secrets/GATEWAY_CLIENT_SECRET
