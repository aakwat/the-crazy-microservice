-- Runs once as the bootstrap superuser on an empty data directory, before the service migrates.
-- Least privilege: tickets_migrator owns the schema and runs DDL; tickets_app only reads and writes rows.

\set migrator_password `cat /run/secrets/TICKETS_DB_MIGRATOR_PASSWORD`
\set app_password      `cat /run/secrets/TICKETS_DB_APP_PASSWORD`

CREATE ROLE tickets_migrator LOGIN PASSWORD :'migrator_password';
CREATE ROLE tickets_app      LOGIN PASSWORD :'app_password';

-- A pinned search_path stops function shadowing (CVE-2018-1058).
ALTER ROLE tickets_migrator SET search_path = public;
ALTER ROLE tickets_app      SET search_path = public;

-- One bad query cannot pin a connection, hold a lock or leak a transaction.
ALTER ROLE tickets_app SET statement_timeout                   = '30s';
ALTER ROLE tickets_app SET lock_timeout                        = '5s';
ALTER ROLE tickets_app SET idle_in_transaction_session_timeout = '60s';

REVOKE ALL     ON DATABASE :"DBNAME" FROM PUBLIC;
GRANT  CONNECT ON DATABASE :"DBNAME" TO tickets_migrator, tickets_app;

REVOKE CREATE        ON SCHEMA public FROM PUBLIC;
GRANT  USAGE, CREATE ON SCHEMA public TO tickets_migrator;
GRANT  USAGE         ON SCHEMA public TO tickets_app;

-- Whatever tickets_migrator creates later is DML-only for tickets_app.
ALTER DEFAULT PRIVILEGES FOR ROLE tickets_migrator IN SCHEMA public
  GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO tickets_app;
ALTER DEFAULT PRIVILEGES FOR ROLE tickets_migrator IN SCHEMA public
  GRANT USAGE, SELECT ON SEQUENCES TO tickets_app;
