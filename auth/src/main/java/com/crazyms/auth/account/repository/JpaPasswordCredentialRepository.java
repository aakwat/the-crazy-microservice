package com.crazyms.auth.account.repository;

import com.crazyms.auth.account.domain.PasswordCredential;
import com.crazyms.auth.account.domain.port.PasswordCredentialRepository;
import com.crazyms.auth.account.repository.jpa.PasswordCredentialEntity;
import com.crazyms.auth.account.repository.jpa.PasswordCredentialJpaRepository;
import java.time.Instant;
import java.util.Optional;
import java.util.UUID;
import org.springframework.stereotype.Repository;

@Repository
class JpaPasswordCredentialRepository implements PasswordCredentialRepository {

  private final PasswordCredentialJpaRepository jpa;

  JpaPasswordCredentialRepository(PasswordCredentialJpaRepository jpa) {
    this.jpa = jpa;
  }

  @Override
  public void insert(PasswordCredential credential) {
    jpa.saveAndFlush(
        new PasswordCredentialEntity(credential.accountId(), credential.passwordHash()));
  }

  @Override
  public Optional<PasswordCredential> findByAccountId(UUID accountId) {
    return jpa.findById(accountId)
        .map(e -> new PasswordCredential(e.getId(), e.getPasswordHash(), e.getLockedUntil()));
  }

  @Override
  public void recordFailedLogin(UUID accountId, int maxFailures, Instant lockUntil, Instant now) {
    jpa.recordFailedLogin(accountId, maxFailures, lockUntil, now);
  }

  @Override
  public void clearFailedLogins(UUID accountId, Instant now) {
    jpa.clearFailedLogins(accountId, now);
  }
}
