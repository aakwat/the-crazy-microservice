package com.crazyms.auth.account.domain;

import java.time.Instant;
import java.util.Objects;
import java.util.UUID;

/** One audited role change: who changed whose role, from what to what, and when. */
public record RoleChange(UUID accountId, UUID actorId, Role from, Role to, Instant changedAt) {

  public RoleChange {
    Objects.requireNonNull(accountId);
    Objects.requireNonNull(actorId);
    Objects.requireNonNull(from);
    Objects.requireNonNull(to);
    Objects.requireNonNull(changedAt);
    if (from.equals(to)) {
      throw new IllegalArgumentException("a role change must change the role");
    }
  }
}
