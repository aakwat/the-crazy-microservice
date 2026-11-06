package com.crazyms.auth.account.domain;

import java.time.Instant;
import java.util.Objects;
import java.util.UUID;

/** lockedUntil is null if never locked or a successful login cleared it. */
public record PasswordCredential(UUID accountId, String passwordHash, Instant lockedUntil) {

  /** NIST SP 800-63B: length over composition rules; the cap bounds what Argon2 has to hash. */
  public static final int MIN_PASSWORD_LENGTH = 12;

  public static final int MAX_PASSWORD_LENGTH = 128;

  public PasswordCredential {
    Objects.requireNonNull(accountId);
    if (passwordHash == null || passwordHash.isEmpty()) {
      throw new IllegalArgumentException("password hash is required");
    }
  }

  public static PasswordCredential create(UUID accountId, String passwordHash) {
    return new PasswordCredential(accountId, passwordHash, null);
  }

  public boolean isLockedAt(Instant now) {
    return lockedUntil != null && now.isBefore(lockedUntil);
  }
}
