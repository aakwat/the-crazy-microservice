package com.crazyms.auth.account.repository;

import com.crazyms.auth.account.domain.Account;
import com.crazyms.auth.account.domain.Role;
import com.crazyms.auth.account.repository.jpa.AccountEntity;
import com.crazyms.auth.account.repository.jpa.AccountJpaRepository;
import java.util.Optional;
import java.util.UUID;
import org.hibernate.exception.ConstraintViolationException;
import org.springframework.dao.DataIntegrityViolationException;
import org.springframework.stereotype.Repository;

@Repository
public class AccountRepository {

    static final String EMAIL_UNIQUE = "account_email_key";

    private final AccountJpaRepository jpa;

    AccountRepository(AccountJpaRepository jpa) {
        this.jpa = jpa;
    }

    /** Returns false when the email is already registered. */
    public boolean insertIfEmailFree(Account account) {
        try {
            // Flush now so the unique constraint fires inside this try, not at commit.
            jpa.saveAndFlush(toEntity(account));
            return true;
        } catch (DataIntegrityViolationException e) {
            if (e.getCause() instanceof ConstraintViolationException cve
                    && EMAIL_UNIQUE.equals(cve.getConstraintName())) {
                return false;
            }
            throw e;
        }
    }

    public Optional<Account> findByEmail(String normalizedEmail) {
        return jpa.findByEmail(normalizedEmail).map(AccountRepository::toDomain);
    }

    public Optional<Account> findById(UUID id) {
        return jpa.findById(id).map(AccountRepository::toDomain);
    }

    private static AccountEntity toEntity(Account a) {
        return new AccountEntity(a.id(), a.email(), a.passwordHash(), a.role().name(), a.createdAt());
    }

    private static Account toDomain(AccountEntity e) {
        return new Account(e.getId(), e.getEmail(), e.getPasswordHash(), new Role(e.getRole()), e.getCreatedAt());
    }
}
