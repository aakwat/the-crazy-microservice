package com.crazyms.auth.login.repository;

import com.crazyms.auth.login.domain.AuthenticationAttempt;
import com.crazyms.auth.login.domain.AuthenticationAttemptRepository;
import java.time.Instant;
import java.time.OffsetDateTime;
import java.time.ZoneOffset;
import org.springframework.jdbc.core.simple.JdbcClient;
import org.springframework.stereotype.Repository;

/** Append-only inserts and an age-based purge; no entity, since rows are never updated. */
@Repository
class JdbcAuthenticationAttemptRepository implements AuthenticationAttemptRepository {

  private final JdbcClient jdbc;

  JdbcAuthenticationAttemptRepository(JdbcClient jdbc) {
    this.jdbc = jdbc;
  }

  @Override
  public void insert(AuthenticationAttempt attempt) {
    jdbc.sql(
            """
            INSERT INTO authentication_attempts
              (submitted_identifier, account_id, was_successful, failure_reason,
               ip_address, user_agent, attempted_at)
            VALUES
              (:identifier, :accountId, :successful, :reason,
               CAST(:ip AS INET), :userAgent, :attemptedAt)
            """)
        .param("identifier", attempt.submittedIdentifier())
        .param("accountId", attempt.accountId())
        .param("successful", attempt.successful())
        .param("reason", attempt.failureReason() == null ? null : attempt.failureReason().code())
        .param("ip", attempt.client().ipAddress())
        .param("userAgent", attempt.client().userAgent())
        .param("attemptedAt", utc(attempt.attemptedAt()))
        .update();
  }

  @Override
  public int deleteOlderThan(Instant cutoff) {
    return jdbc.sql("DELETE FROM authentication_attempts WHERE attempted_at < :cutoff")
        .param("cutoff", utc(cutoff))
        .update();
  }

  // The Postgres driver binds OffsetDateTime to timestamptz natively; Instant it does not.
  private static OffsetDateTime utc(Instant instant) {
    return OffsetDateTime.ofInstant(instant, ZoneOffset.UTC);
  }
}
