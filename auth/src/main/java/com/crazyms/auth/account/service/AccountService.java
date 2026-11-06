package com.crazyms.auth.account.service;

import com.crazyms.auth.account.config.AccountProperties;
import com.crazyms.auth.account.domain.Account;
import com.crazyms.auth.account.domain.AccountCredential;
import com.crazyms.auth.account.domain.AccountRepository;
import com.crazyms.auth.account.domain.PasswordCredential;
import com.crazyms.auth.account.domain.PasswordCredentialRepository;
import com.crazyms.auth.account.domain.Role;
import java.time.Clock;
import java.time.Instant;
import java.util.Optional;
import java.util.UUID;
import org.springframework.security.crypto.password.PasswordEncoder;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

/** The account module's whole public surface; other modules never touch its repositories. */
@Service
public class AccountService {

  private final AccountRepository accounts;
  private final PasswordCredentialRepository credentials;
  private final PasswordEncoder encoder;
  private final Clock clock;
  private final Role defaultRole;

  AccountService(
      AccountRepository accounts,
      PasswordCredentialRepository credentials,
      PasswordEncoder encoder,
      Clock clock,
      AccountProperties props) {
    this.accounts = accounts;
    this.credentials = credentials;
    this.encoder = encoder;
    this.clock = clock;
    this.defaultRole = new Role(props.defaultRole());
  }

  @Transactional
  public Account register(String email, String rawPassword) {
    var account = Account.register(email, defaultRole, clock.instant());
    if (!accounts.insertIfEmailFree(account)) {
      throw new EmailTakenException();
    }
    credentials.insert(PasswordCredential.create(account.id(), encoder.encode(rawPassword)));
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

  @Transactional(readOnly = true)
  public Optional<AccountCredential> findForLogin(String email) {
    return findByEmail(email)
        .flatMap(a -> credentials.findByAccountId(a.id()).map(c -> new AccountCredential(a, c)));
  }

  @Transactional
  public void recordFailedLogin(UUID accountId, int maxFailures, Instant lockUntil, Instant now) {
    credentials.recordFailedLogin(accountId, maxFailures, lockUntil, now);
  }

  @Transactional
  public void clearFailedLogins(UUID accountId, Instant now) {
    credentials.clearFailedLogins(accountId, now);
  }
}
