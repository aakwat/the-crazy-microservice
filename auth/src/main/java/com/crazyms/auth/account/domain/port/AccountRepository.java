package com.crazyms.auth.account.domain.port;

import com.crazyms.auth.account.domain.Account;
import com.crazyms.auth.account.domain.Role;
import java.util.Optional;
import java.util.UUID;

/** Port: what the account use cases need from storage, in domain types only. */
public interface AccountRepository {

  /** Returns false when the email is already registered. */
  boolean insertIfEmailFree(Account account);

  Optional<Account> findByEmail(String normalizedEmail);

  Optional<Account> findById(UUID id);

  /** Locks the row until the transaction ends, so concurrent role changes apply one at a time. */
  Optional<Account> findByIdForUpdate(UUID id);

  void updateRole(UUID id, Role role);
}
