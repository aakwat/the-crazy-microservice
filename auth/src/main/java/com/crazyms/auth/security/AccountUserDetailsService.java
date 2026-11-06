package com.crazyms.auth.security;

import com.crazyms.auth.account.service.AccountService;
import java.time.Clock;
import org.springframework.security.core.userdetails.User;
import org.springframework.security.core.userdetails.UserDetails;
import org.springframework.security.core.userdetails.UserDetailsService;
import org.springframework.security.core.userdetails.UsernameNotFoundException;
import org.springframework.stereotype.Component;

/** Lets the login form authenticate against accounts; the form's username field is the email. */
@Component
class AccountUserDetailsService implements UserDetailsService {

    private final AccountService accounts;
    private final Clock clock;

    AccountUserDetailsService(AccountService accounts, Clock clock) {
        this.accounts = accounts;
        this.clock = clock;
    }

    @Override
    public UserDetails loadUserByUsername(String email) {
        return accounts.findForLogin(email)
                // A locked account answers like an unknown one: same message, same dummy-hash timing, nothing to probe.
                .filter(c -> !c.password().isLockedAt(clock.instant()))
                // Principal name is the account id, so tokens carry a stable, non-personal sub instead of the email.
                .map(c -> User.withUsername(c.account().id().toString())
                        .password(c.password().passwordHash())
                        .roles(c.account().role().name())
                        .build())
                .orElseThrow(() -> new UsernameNotFoundException("unknown or locked account"));
    }
}
