package com.crazyms.auth.account.repository.jpa;

import java.time.Instant;
import java.util.UUID;
import org.springframework.data.jpa.repository.JpaRepository;
import org.springframework.data.jpa.repository.Modifying;
import org.springframework.data.jpa.repository.Query;
import org.springframework.data.repository.query.Param;

public interface PasswordCredentialJpaRepository
    extends JpaRepository<PasswordCredentialEntity, UUID> {

  // One atomic statement, so replicas cannot race past the failure limit.
  @Modifying
  @Query(
      nativeQuery = true,
      value =
          """
          UPDATE account_password_credential SET
            failed_login_attempts = CASE WHEN failed_login_attempts + 1 >= :maxFailures
                                         THEN 0 ELSE failed_login_attempts + 1 END,
            locked_until          = CASE WHEN failed_login_attempts + 1 >= :maxFailures
                                         THEN :lockUntil ELSE locked_until END,
            last_failed_at        = :now,
            updated_at            = :now
          WHERE account_id = :accountId
            AND (locked_until IS NULL OR locked_until <= :now)
          """)
  int recordFailedLogin(
      @Param("accountId") UUID accountId,
      @Param("maxFailures") int maxFailures,
      @Param("lockUntil") Instant lockUntil,
      @Param("now") Instant now);

  @Modifying
  @Query(
      nativeQuery = true,
      value =
          """
          UPDATE account_password_credential SET
            failed_login_attempts = 0,
            locked_until          = NULL,
            updated_at            = :now
          WHERE account_id = :accountId
            AND (failed_login_attempts <> 0 OR locked_until IS NOT NULL)
          """)
  int clearFailedLogins(@Param("accountId") UUID accountId, @Param("now") Instant now);
}
