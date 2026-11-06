package com.crazyms.auth.authserver.repository;

import com.crazyms.auth.authserver.domain.ExpiredAuthorizationRepository;
import java.time.Instant;
import java.time.OffsetDateTime;
import java.time.ZoneOffset;
import org.springframework.jdbc.core.simple.JdbcClient;
import org.springframework.stereotype.Repository;

@Repository
class JdbcExpiredAuthorizationRepository implements ExpiredAuthorizationRepository {

  private final JdbcClient jdbc;

  JdbcExpiredAuthorizationRepository(JdbcClient jdbc) {
    this.jdbc = jdbc;
  }

  // Must match the idx_oauth2_authorization_expires expression, or Postgres skips the index.
  @Override
  public int deleteExpiredBefore(Instant cutoff) {
    return jdbc.sql(
            """
            DELETE FROM oauth2_authorization
            WHERE GREATEST(
              authorization_code_expires_at, access_token_expires_at, refresh_token_expires_at,
              oidc_id_token_expires_at, user_code_expires_at, device_code_expires_at) < :cutoff
            """)
        .param("cutoff", OffsetDateTime.ofInstant(cutoff, ZoneOffset.UTC))
        .update();
  }
}
