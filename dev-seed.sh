#!/bin/sh
# Dev-only test accounts, so a fresh stack is usable at once; run it after `docker compose up`.
set -eu
cd "$(dirname "$0")"

GATEWAY=http://127.0.0.1:8080
PASSWORD='Password@1234'

for email in admin@example.test helper@example.test me@example.test; do
  # 409 means the account already exists, which is fine on a re-run.
  curl -s -o /dev/null -w "register $email %{http_code}\n" \
    -H 'Content-Type: application/json' -H 'Sec-Fetch-Site: same-origin' \
    -d "{\"email\":\"$email\",\"password\":\"$PASSWORD\"}" "$GATEWAY/auth/register"
done

# Roles set straight in the database: a dev shortcut, so the role API's audit log has no rows for them.
docker-compose exec -T auth-db psql -U postgres -d auth -q -v ON_ERROR_STOP=1 \
  -c "UPDATE account SET role = 'admin' WHERE email = 'admin@example.test'" \
  -c "UPDATE account SET role = 'agent' WHERE email = 'me@example.test'"

echo "seeded: admin@example.test (admin), me@example.test (agent), helper@example.test (customer)"
echo "password: $PASSWORD"
