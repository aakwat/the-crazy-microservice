#!/bin/sh
# Runs once, on an empty data directory, before the service applies its Flyway migrations.
set -eu

psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" \
    -v app_password="$(cat /run/secrets/AUTH_DB_APP_PASSWORD)" <<'SQL'
create role auth_app login password :'app_password';

revoke all on database auth from public;
grant connect on database auth to auth_app;

revoke all on schema public from public;
grant usage on schema public to auth_app;

-- Tables Flyway creates later as auth_owner become row-level read/write for the app, never DDL.
alter default privileges for role auth_owner in schema public
    grant select, insert, update, delete on tables to auth_app;
SQL
