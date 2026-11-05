package com.crazyms.auth.account.service;

import com.crazyms.auth.account.domain.Account;
import com.crazyms.auth.account.domain.Role;
import com.crazyms.auth.account.repository.AccountRepository;
import com.crazyms.auth.config.AuthProperties;
import java.time.Clock;
import java.util.Optional;
import java.util.UUID;
import org.springframework.security.crypto.password.PasswordEncoder;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

@Service
public class AccountService {

    private final AccountRepository accounts;
    private final PasswordEncoder encoder;
    private final Clock clock;
    private final Role defaultRole;

    AccountService(AccountRepository accounts, PasswordEncoder encoder, Clock clock, AuthProperties props) {
        this.accounts = accounts;
        this.encoder = encoder;
        this.clock = clock;
        this.defaultRole = new Role(props.defaultRole());
    }

    @Transactional
    public Account register(String email, String rawPassword) {
        var account = Account.register(email, encoder.encode(rawPassword), defaultRole, clock.instant());
        if (!accounts.insertIfEmailFree(account)) {
            throw new EmailTakenException();
        }
        return account;
    }

    @Transactional(readOnly = true)
    public Optional<Account> findByEmail(String email) {
        return accounts.findByEmail(Account.normalizeEmail(email));
    }

    @Transactional(readOnly = true)
    public Optional<Account> find(UUID id) {
        return accounts.findById(id);
    }
}
