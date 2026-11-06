package com.crazyms.auth.account.repository.jpa;

import jakarta.persistence.Column;
import jakarta.persistence.Entity;
import jakarta.persistence.Id;
import jakarta.persistence.PostLoad;
import jakarta.persistence.PostPersist;
import jakarta.persistence.Table;
import jakarta.persistence.Transient;
import java.time.Instant;
import java.util.UUID;
import org.springframework.data.domain.Persistable;

@Entity
@Table(name = "account_password_credential")
public class PasswordCredentialEntity implements Persistable<UUID> {

  @Id
  @Column(name = "account_id")
  private UUID accountId;

  @Column(name = "password_hash", nullable = false)
  private String passwordHash;

  // Read-only here: only the repository's atomic updates change lockout state.
  @Column(name = "locked_until", insertable = false, updatable = false)
  private Instant lockedUntil;

  @Transient private boolean isNew = true;

  protected PasswordCredentialEntity() {}

  public PasswordCredentialEntity(UUID accountId, String passwordHash) {
    this.accountId = accountId;
    this.passwordHash = passwordHash;
  }

  @PostLoad
  @PostPersist
  void markNotNew() {
    isNew = false;
  }

  @Override
  public UUID getId() {
    return accountId;
  }

  @Override
  public boolean isNew() {
    return isNew;
  }

  public String getPasswordHash() {
    return passwordHash;
  }

  public Instant getLockedUntil() {
    return lockedUntil;
  }
}
