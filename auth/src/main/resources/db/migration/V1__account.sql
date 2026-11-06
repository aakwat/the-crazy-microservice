-- account: identity only; how someone proves it lives in the credential tables.
CREATE TABLE account (
  id         UUID        PRIMARY KEY DEFAULT uuidv7(),
  email      TEXT        NOT NULL CHECK (email = LOWER(email)) CHECK (LENGTH(email) BETWEEN 3 AND 254),
  role       TEXT        NOT NULL CHECK (role ~ '^[a-z][a-z0-9_-]{0,31}$'),
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  CONSTRAINT account_email_key UNIQUE (email)
);
COMMENT ON COLUMN account.id   IS 'UUIDv7 (time-ordered for index locality); the service assigns it, the default covers other writers';
COMMENT ON COLUMN account.role IS 'Role name defined by the deploying project, not by this service';

-- account_password_credential: one row per account; Argon2id hash plus lockout state.
CREATE TABLE account_password_credential (
  account_id            UUID        NOT NULL PRIMARY KEY REFERENCES account(id) ON DELETE CASCADE,
  password_hash         TEXT        NOT NULL CHECK (LENGTH(password_hash) > 0),
  password_algorithm    TEXT        NOT NULL DEFAULT 'argon2id' CHECK (LENGTH(password_algorithm) > 0),
  failed_login_attempts INTEGER     NOT NULL DEFAULT 0 CHECK (failed_login_attempts >= 0),
  locked_until          TIMESTAMPTZ,
  last_failed_at        TIMESTAMPTZ,
  created_at            TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at            TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_credential_locked_until ON account_password_credential (locked_until)
  WHERE locked_until IS NOT NULL;

-- authentication_attempts: one row per login outcome, including identifiers that match no account (CWE-307).
CREATE TABLE authentication_attempts (
  id                   BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  -- What was typed, not who was found: unknown identifiers are the spraying signal.
  submitted_identifier TEXT        NOT NULL CHECK (LENGTH(submitted_identifier) <= 320),
  -- SET NULL on erasure, so the security history outlives the account (GDPR Art. 17 removes the person).
  account_id           UUID        REFERENCES account(id) ON DELETE SET NULL,
  was_successful       BOOLEAN     NOT NULL,
  failure_reason       TEXT,
  ip_address           INET,
  user_agent           TEXT        CHECK (LENGTH(user_agent) <= 500),
  attempted_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  CONSTRAINT authentication_attempts_reason_agrees
    CHECK (was_successful = (failure_reason IS NULL))
);
-- Case-insensitive: the same account gets sprayed under varying case.
CREATE INDEX idx_auth_attempts_identifier
  ON authentication_attempts (LOWER(submitted_identifier), attempted_at DESC);
-- Only failures drive spray decisions, so the index stays small on a table that grows with every login.
CREATE INDEX idx_auth_attempts_ip
  ON authentication_attempts (ip_address, attempted_at DESC)
  WHERE NOT was_successful;
-- The retention sweep deletes by age alone, which neither index above can serve.
CREATE INDEX idx_auth_attempts_attempted_at ON authentication_attempts (attempted_at);
