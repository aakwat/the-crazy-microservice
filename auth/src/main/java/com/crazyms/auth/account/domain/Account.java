package com.crazyms.auth.account.domain;

import com.crazyms.auth.shared.id.UuidV7;
import java.time.Instant;
import java.util.Locale;
import java.util.Objects;
import java.util.UUID;

/** Who someone is; how they prove it is a separate credential. */
public record Account(UUID id, String email, Role role, Instant createdAt) {

  /** RFC 5321's path limit; the database enforces the same bound. */
  public static final int MAX_EMAIL_LENGTH = 254;

  public Account {
    Objects.requireNonNull(id);
    Objects.requireNonNull(role);
    Objects.requireNonNull(createdAt);
    if (!email.equals(normalizeEmail(email))) {
      throw new IllegalArgumentException("email must be normalized");
    }
  }

  public static Account register(String email, Role role, Instant now) {
    return new Account(UuidV7.at(now), normalizeEmail(email), role, now);
  }

  public Account withRole(Role newRole) {
    return new Account(id, email, newRole, createdAt);
  }

  public static String normalizeEmail(String email) {
    return email.strip().toLowerCase(Locale.ROOT);
  }
}
