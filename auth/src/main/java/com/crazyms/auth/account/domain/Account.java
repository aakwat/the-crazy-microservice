package com.crazyms.auth.account.domain;

import com.crazyms.auth.shared.id.UuidV7;
import java.time.Instant;
import java.util.Locale;
import java.util.Objects;
import java.util.UUID;

public record Account(UUID id, String email, String passwordHash, Role role, Instant createdAt) {

    public Account {
        Objects.requireNonNull(id);
        Objects.requireNonNull(passwordHash);
        Objects.requireNonNull(role);
        Objects.requireNonNull(createdAt);
        if (!email.equals(normalizeEmail(email))) {
            throw new IllegalArgumentException("email must be normalized");
        }
    }

    public static Account register(String email, String passwordHash, Role role, Instant now) {
        return new Account(UuidV7.at(now), normalizeEmail(email), passwordHash, role, now);
    }

    public static String normalizeEmail(String email) {
        return email.strip().toLowerCase(Locale.ROOT);
    }
}
