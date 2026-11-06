package com.crazyms.auth.account.repository;

import com.crazyms.auth.account.domain.Account;
import com.crazyms.auth.account.domain.AccountRepository;
import com.crazyms.auth.account.domain.Role;
import com.crazyms.auth.account.repository.jpa.AccountEntity;
import com.crazyms.auth.account.repository.jpa.AccountJpaRepository;
import java.util.Optional;
import java.util.UUID;
import org.hibernate.exception.ConstraintViolationException;
import org.springframework.dao.DataIntegrityViolationException;
import org.springframework.stereotype.Repository;

@Repository
class JpaAccountRepository implements AccountRepository {

    static final String EMAIL_UNIQUE = "account_email_key";

    private final AccountJpaRepository jpa;

    JpaAccountRepository(AccountJpaRepository jpa) {
        this.jpa = jpa;
    }


    @Override
    public boolean insertIfEmailFree(Account account) {
        try {
            // Flush now so the unique constraint fires inside this try, not at commit.
            jpa.saveAndFlush(new AccountEntity(account.id(), account.email(), account.role().name(), account.createdAt()));
            return true;
        } catch (DataIntegrityViolationException e) {
            if (e.getCause() instanceof ConstraintViolationException cve
                    && EMAIL_UNIQUE.equals(cve.getConstraintName())) {
                return false;
            }
            throw e;
        }
    }

    @Override
    public Optional<Account> findByEmail(String normalizedEmail) {
        return jpa.findByEmail(normalizedEmail).map(JpaAccountRepository::toDomain);
    }

    @Override
    public Optional<Account> findById(UUID id) {
        return jpa.findById(id).map(JpaAccountRepository::toDomain);
    }

    private static Account toDomain(AccountEntity e) {
        return new Account(e.getId(), e.getEmail(), new Role(e.getRole()), e.getCreatedAt());
    }
}
