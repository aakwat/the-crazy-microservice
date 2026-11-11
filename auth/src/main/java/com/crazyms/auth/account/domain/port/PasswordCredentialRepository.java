package com.crazyms.auth.account.domain.port;

import com.crazyms.auth.account.domain.PasswordCredential;
import java.time.Instant;
import java.util.Optional;
import java.util.UUID;

/** Port: password credentials and their lockout state. */
public interface PasswordCredentialRepository {

  void insert(PasswordCredential credential);

  Optional<PasswordCredential> findByAccountId(UUID accountId);

  /** Counts a failure atomically; on reaching maxFailures, locks until lockUntil. */
  void recordFailedLogin(UUID accountId, int maxFailures, Instant lockUntil, Instant now);

  void clearFailedLogins(UUID accountId, Instant now);
}
