-- Runs once as the bootstrap superuser on an empty data directory, before the service migrates.
-- Least privilege: notifications_migrator owns the schema and runs DDL; notifications_app only reads and writes rows.

\set migrator_password `cat /run/secrets/NOTIFICATIONS_DB_MIGRATOR_PASSWORD`
\set app_password      `cat /run/secrets/NOTIFICATIONS_DB_APP_PASSWORD`

CREATE ROLE notifications_migrator LOGIN PASSWORD :'migrator_password';
CREATE ROLE notifications_app      LOGIN PASSWORD :'app_password';

-- A pinned search_path stops function shadowing (CVE-2018-1058).
ALTER ROLE notifications_migrator SET search_path = public;
ALTER ROLE notifications_app      SET search_path = public;

-- One bad query cannot pin a connection, hold a lock or leak a transaction.
ALTER ROLE notifications_app SET statement_timeout                   = '30s';
ALTER ROLE notifications_app SET lock_timeout                        = '5s';
ALTER ROLE notifications_app SET idle_in_transaction_session_timeout = '60s';

REVOKE ALL     ON DATABASE :"DBNAME" FROM PUBLIC;
GRANT  CONNECT ON DATABASE :"DBNAME" TO notifications_migrator, notifications_app;

REVOKE CREATE        ON SCHEMA public FROM PUBLIC;
GRANT  USAGE, CREATE ON SCHEMA public TO notifications_migrator;
GRANT  USAGE         ON SCHEMA public TO notifications_app;

-- Whatever notifications_migrator creates later is DML-only for notifications_app.
ALTER DEFAULT PRIVILEGES FOR ROLE notifications_migrator IN SCHEMA public
  GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO notifications_app;
ALTER DEFAULT PRIVILEGES FOR ROLE notifications_migrator IN SCHEMA public
  GRANT USAGE, SELECT ON SEQUENCES TO notifications_app;
