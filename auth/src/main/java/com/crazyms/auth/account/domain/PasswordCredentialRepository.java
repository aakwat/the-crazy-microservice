package com.crazyms.auth.account.domain;

import java.time.Instant;
import java.util.Optional;
import java.util.UUID;

/** Port: password credentials and their lockout state. */
public interface PasswordCredentialRepository {

    void insert(PasswordCredential credential);

    Optional<PasswordCredential> findByAccountId(UUID accountId);

    /** Counts a failed password atomically and, on reaching maxFailures, locks the credential until lockUntil. */
    void recordFailedLogin(UUID accountId, int maxFailures, Instant lockUntil, Instant now);

    void clearFailedLogins(UUID accountId, Instant now);
}
