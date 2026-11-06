#!/bin/sh
# Runs once as the bootstrap superuser on an empty data directory, before the service applies its migrations.
set -eu

psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" \
  -v db="$POSTGRES_DB" \
  -v migrator_password="$(cat /run/secrets/AUTH_DB_MIGRATOR_PASSWORD)" \
  -v app_password="$(cat /run/secrets/AUTH_DB_APP_PASSWORD)" <<'SQL'
-- Two roles, least privilege: auth_migrator owns the schema and runs DDL; auth_app is DML only.
CREATE ROLE auth_migrator WITH
  LOGIN PASSWORD :'migrator_password'
  NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION;
CREATE ROLE auth_app WITH
  LOGIN PASSWORD :'app_password'
  NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION;

-- Pin search_path to defend against CVE-2018-1058 (function shadowing).
ALTER ROLE auth_migrator SET search_path = public;
ALTER ROLE auth_app      SET search_path = public;

-- One bad query cannot pin a connection, hold a lock, or leak a transaction forever.
ALTER ROLE auth_app SET statement_timeout                   = '30s';
ALTER ROLE auth_app SET lock_timeout                        = '5s';
ALTER ROLE auth_app SET idle_in_transaction_session_timeout = '60s';

REVOKE ALL    ON DATABASE :"db" FROM PUBLIC;
GRANT CONNECT ON DATABASE :"db" TO auth_migrator, auth_app;

REVOKE CREATE ON SCHEMA public FROM PUBLIC;
GRANT  USAGE, CREATE ON SCHEMA public TO auth_migrator;
GRANT  USAGE         ON SCHEMA public TO auth_app;

-- Tables and sequences auth_migrator creates later are DML-only for auth_app, never DDL.
ALTER DEFAULT PRIVILEGES FOR ROLE auth_migrator IN SCHEMA public
  GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES    TO auth_app;
ALTER DEFAULT PRIVILEGES FOR ROLE auth_migrator IN SCHEMA public
  GRANT USAGE, SELECT                  ON SEQUENCES TO auth_app;
SQL
