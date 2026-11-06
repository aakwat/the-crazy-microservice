package com.crazyms.auth.account.domain;

import java.util.Optional;
import java.util.UUID;

/** Port: what the account use cases need from storage, in domain types only. */
public interface AccountRepository {

    /** Returns false when the email is already registered. */
    boolean insertIfEmailFree(Account account);

    Optional<Account> findByEmail(String normalizedEmail);

    Optional<Account> findById(UUID id);
}
