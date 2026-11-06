package com.crazyms.auth.account.domain;

import java.time.Instant;
import java.util.Objects;
import java.util.UUID;

/** lockedUntil is null when the credential has never been locked or a successful login cleared the lock. */
public record PasswordCredential(UUID accountId, String passwordHash, Instant lockedUntil) {

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
