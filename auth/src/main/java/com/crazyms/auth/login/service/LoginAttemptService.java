package com.crazyms.auth.login.service;

import com.crazyms.auth.account.service.AccountService;
import com.crazyms.auth.login.config.LoginProperties;
import com.crazyms.auth.login.domain.AuthenticationAttempt;
import com.crazyms.auth.login.domain.AuthenticationAttemptRepository;
import com.crazyms.auth.login.domain.ClientInfo;
import com.crazyms.auth.login.domain.FailureReason;
import java.time.Clock;
import java.util.UUID;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

/** Records every login outcome and applies lockout; counter and record commit together. */
@Service
public class LoginAttemptService {

  private final AccountService accounts;
  private final AuthenticationAttemptRepository attempts;
  private final LoginProperties props;
  private final Clock clock;

  LoginAttemptService(
      AccountService accounts,
      AuthenticationAttemptRepository attempts,
      LoginProperties props,
      Clock clock) {
    this.accounts = accounts;
    this.attempts = attempts;
    this.props = props;
    this.clock = clock;
  }

  @Transactional
  public void recordSuccess(UUID accountId, String submittedIdentifier, ClientInfo client) {
    var now = clock.instant();
    accounts.clearFailedLogins(accountId, now);
    attempts.insert(AuthenticationAttempt.succeeded(submittedIdentifier, accountId, client, now));
  }

  // The reason comes from stored state: locked accounts deliberately fail like unknown ones.
  @Transactional
  public void recordFailure(String submittedIdentifier, ClientInfo client) {
    var now = clock.instant();
    var found = accounts.findForLogin(submittedIdentifier);
    var reason =
        found.isEmpty()
            ? FailureReason.UNKNOWN_IDENTIFIER
            : found.get().password().isLockedAt(now)
                ? FailureReason.ACCOUNT_LOCKED
                : FailureReason.BAD_PASSWORD;
    if (reason == FailureReason.BAD_PASSWORD) {
      var lockout = props.lockout();
      accounts.recordFailedLogin(
          found.get().account().id(), lockout.maxFailures(), now.plus(lockout.duration()), now);
    }
    attempts.insert(
        AuthenticationAttempt.failed(
            submittedIdentifier,
            found.map(c -> c.account().id()).orElse(null),
            reason,
            client,
            now));
  }
}
