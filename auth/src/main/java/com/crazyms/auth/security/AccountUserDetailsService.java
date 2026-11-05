package com.crazyms.auth.security;

import com.crazyms.auth.account.service.AccountService;
import org.springframework.security.core.userdetails.User;
import org.springframework.security.core.userdetails.UserDetails;
import org.springframework.security.core.userdetails.UserDetailsService;
import org.springframework.security.core.userdetails.UsernameNotFoundException;
import org.springframework.stereotype.Component;

/** Lets the login form authenticate against accounts; the form's username field is the email. */
@Component
class AccountUserDetailsService implements UserDetailsService {

    private final AccountService accounts;

    AccountUserDetailsService(AccountService accounts) {
        this.accounts = accounts;
    }

    @Override
    public UserDetails loadUserByUsername(String email) {
        return accounts.findByEmail(email)
                // Principal name is the account id, so tokens carry a stable, non-personal sub instead of the email.
                .map(a -> User.withUsername(a.id().toString())
                        .password(a.passwordHash())
                        .roles(a.role().name())
                        .build())
                .orElseThrow(() -> new UsernameNotFoundException("unknown account"));
    }
}
