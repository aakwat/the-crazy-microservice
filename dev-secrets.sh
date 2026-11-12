#!/bin/sh
# Project wiring: each plugin makes its own secrets, then values two plugins share are copied across.
set -eu
cd "$(dirname "$0")"
./auth/dev-secrets.sh
./events/dev-secrets.sh
./gateway/dev-secrets.sh
./tickets/dev-secrets.sh

# The gateway's OAuth client secret is issued by auth's registration, so both must hold the same value.
cp auth/secrets/AUTH_CLIENT_GATEWAY_SECRET gateway/secrets/GATEWAY_CLIENT_SECRET
chmod 644 gateway/secrets/GATEWAY_CLIENT_SECRET

# Tickets logs in to the event bus as the ACL user the events plugin creates for it.
cp events/secrets/EVENTS_REDIS_TICKETS_PASSWORD tickets/secrets/TICKETS_EVENTS_REDIS_PASSWORD
chmod 644 tickets/secrets/TICKETS_EVENTS_REDIS_PASSWORD
