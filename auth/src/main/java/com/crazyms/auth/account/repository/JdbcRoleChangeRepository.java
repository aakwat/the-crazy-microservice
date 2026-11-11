package com.crazyms.auth.account.repository;

import com.crazyms.auth.account.domain.RoleChange;
import com.crazyms.auth.account.domain.port.RoleChangeRepository;
import java.time.Instant;
import java.time.OffsetDateTime;
import java.time.ZoneOffset;
import org.springframework.jdbc.core.simple.JdbcClient;
import org.springframework.stereotype.Repository;

/** Append-only inserts and an age-based purge; no entity, since rows are never updated. */
@Repository
class JdbcRoleChangeRepository implements RoleChangeRepository {

  private final JdbcClient jdbc;

  JdbcRoleChangeRepository(JdbcClient jdbc) {
    this.jdbc = jdbc;
  }

  @Override
  public void insert(RoleChange change) {
    jdbc.sql(
            """
            INSERT INTO account_role_change (account_id, actor_id, from_role, to_role, changed_at)
            VALUES (:accountId, :actorId, :from, :to, :changedAt)
            """)
        .param("accountId", change.accountId())
        .param("actorId", change.actorId())
        .param("from", change.from().name())
        .param("to", change.to().name())
        .param("changedAt", utc(change.changedAt()))
        .update();
  }

  @Override
  public int deleteOlderThan(Instant cutoff) {
    return jdbc.sql("DELETE FROM account_role_change WHERE changed_at < :cutoff")
        .param("cutoff", utc(cutoff))
        .update();
  }

  // The Postgres driver binds OffsetDateTime to timestamptz natively; Instant it does not.
  private static OffsetDateTime utc(Instant instant) {
    return OffsetDateTime.ofInstant(instant, ZoneOffset.UTC);
  }
}
