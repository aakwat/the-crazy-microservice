-- spring-security-oauth2-authorization-server 7.1.2 schema with its Postgres notes applied (blob->text, timestamptz).
-- No oauth2_registered_client table: clients come from config.
CREATE TABLE oauth2_authorization (
  id                            VARCHAR(100)  NOT NULL PRIMARY KEY,
  registered_client_id          VARCHAR(100)  NOT NULL,
  principal_name                VARCHAR(200)  NOT NULL,
  authorization_grant_type      VARCHAR(100)  NOT NULL,
  authorized_scopes             VARCHAR(1000) DEFAULT NULL,
  attributes                    TEXT          DEFAULT NULL,
  state                         VARCHAR(500)  DEFAULT NULL,
  authorization_code_value      TEXT          DEFAULT NULL,
  authorization_code_issued_at  TIMESTAMPTZ   DEFAULT NULL,
  authorization_code_expires_at TIMESTAMPTZ   DEFAULT NULL,
  authorization_code_metadata   TEXT          DEFAULT NULL,
  access_token_value            TEXT          DEFAULT NULL,
  access_token_issued_at        TIMESTAMPTZ   DEFAULT NULL,
  access_token_expires_at       TIMESTAMPTZ   DEFAULT NULL,
  access_token_metadata         TEXT          DEFAULT NULL,
  access_token_type             VARCHAR(100)  DEFAULT NULL,
  access_token_scopes           VARCHAR(1000) DEFAULT NULL,
  oidc_id_token_value           TEXT          DEFAULT NULL,
  oidc_id_token_issued_at       TIMESTAMPTZ   DEFAULT NULL,
  oidc_id_token_expires_at      TIMESTAMPTZ   DEFAULT NULL,
  oidc_id_token_metadata        TEXT          DEFAULT NULL,
  refresh_token_value           TEXT          DEFAULT NULL,
  refresh_token_issued_at       TIMESTAMPTZ   DEFAULT NULL,
  refresh_token_expires_at      TIMESTAMPTZ   DEFAULT NULL,
  refresh_token_metadata        TEXT          DEFAULT NULL,
  user_code_value               TEXT          DEFAULT NULL,
  user_code_issued_at           TIMESTAMPTZ   DEFAULT NULL,
  user_code_expires_at          TIMESTAMPTZ   DEFAULT NULL,
  user_code_metadata            TEXT          DEFAULT NULL,
  device_code_value             TEXT          DEFAULT NULL,
  device_code_issued_at         TIMESTAMPTZ   DEFAULT NULL,
  device_code_expires_at        TIMESTAMPTZ   DEFAULT NULL,
  device_code_metadata          TEXT          DEFAULT NULL
);
-- The server looks authorizations up by exact token value; hash indexes fit equality on long values.
CREATE INDEX idx_oauth2_authorization_state    ON oauth2_authorization USING HASH (state);
CREATE INDEX idx_oauth2_authorization_code     ON oauth2_authorization USING HASH (authorization_code_value);
CREATE INDEX idx_oauth2_authorization_access   ON oauth2_authorization USING HASH (access_token_value);
CREATE INDEX idx_oauth2_authorization_refresh  ON oauth2_authorization USING HASH (refresh_token_value);
CREATE INDEX idx_oauth2_authorization_id_token ON oauth2_authorization USING HASH (oidc_id_token_value);

CREATE TABLE oauth2_authorization_consent (
  registered_client_id VARCHAR(100)  NOT NULL,
  principal_name       VARCHAR(200)  NOT NULL,
  authorities          VARCHAR(1000) NOT NULL,
  PRIMARY KEY (registered_client_id, principal_name)
);
