package com.crazyms.auth.account.service;

import com.crazyms.auth.account.config.AccountProperties;
import com.crazyms.auth.account.domain.Account;
import com.crazyms.auth.account.domain.AccountCredential;
import com.crazyms.auth.account.domain.PasswordCredential;
import com.crazyms.auth.account.domain.Role;
import com.crazyms.auth.account.domain.RoleChange;
import com.crazyms.auth.account.domain.port.AccountRepository;
import com.crazyms.auth.account.domain.port.PasswordCredentialRepository;
import com.crazyms.auth.account.domain.port.RoleChangeRepository;
import java.time.Clock;
import java.time.Instant;
import java.util.Optional;
import java.util.Set;
import java.util.UUID;
import java.util.function.Function;
import java.util.stream.Collectors;
import java.util.stream.Stream;
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
  private final RoleChangeRepository roleChanges;
  private final Role defaultRole;
  private final Set<Role> roles;

  AccountService(
      AccountRepository accounts,
      PasswordCredentialRepository credentials,
      RoleChangeRepository roleChanges,
      PasswordEncoder encoder,
      Clock clock,
      AccountProperties props) {
    this.accounts = accounts;
    this.credentials = credentials;
    this.roleChanges = roleChanges;
    this.encoder = encoder;
    this.clock = clock;
    this.defaultRole = new Role(props.defaultRole());
    this.roles = props.roles().stream().map(Role::new).collect(Collectors.toUnmodifiableSet());
    if (!roles.contains(defaultRole) || !roles.contains(Role.ADMIN)) {
      throw new IllegalStateException("auth.account.roles must hold the default role and admin");
    }
    if (defaultRole.equals(Role.ADMIN)) {
      throw new IllegalStateException("auth.account.default-role must not be admin");
    }
  }

  // The token's roles claim can be up to 15 minutes old; the stored role is the truth.
  private static void requireAdmin(Account actor) {
    if (actor == null || !actor.role().equals(Role.ADMIN)) {
      throw new NotAdminException();
    }
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

  /** Empty when no account has the email. */
  @Transactional(readOnly = true)
  public Optional<Account> findByEmailAsAdmin(UUID actorId, String email) {
    requireAdmin(accounts.findById(actorId).orElse(null));
    return findByEmail(email);
  }

  /** Empty when the account does not exist. */
  @Transactional
  public Optional<Account> changeRole(UUID actorId, UUID accountId, String roleName) {
    var to =
        roles.stream()
            .filter(r -> r.name().equals(roleName))
            .findFirst()
            .orElseThrow(UnknownRoleException::new);
    var locked =
        accounts.findAllByIdForUpdate(Stream.of(actorId, accountId).distinct().toList()).stream()
            .collect(Collectors.toMap(Account::id, Function.identity()));
    requireAdmin(locked.get(actorId));
    return Optional.ofNullable(locked.get(accountId))
        .map(account -> {
          if (account.role().equals(to)) {
            return account;
          }
          // With the locked actor check, this keeps at least one admin.
          if (actorId.equals(accountId) && account.role().equals(Role.ADMIN)) {
            throw new OwnAdminRoleException();
          }
          accounts.updateRole(accountId, to);
          roleChanges.insert(
              new RoleChange(accountId, actorId, account.role(), to, clock.instant()));
          return account.withRole(to);
        });
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
