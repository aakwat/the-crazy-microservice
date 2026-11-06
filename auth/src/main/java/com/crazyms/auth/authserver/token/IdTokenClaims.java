package com.crazyms.auth.authserver.token;

import com.crazyms.auth.account.service.AccountService;
import java.util.UUID;
import org.springframework.security.oauth2.core.oidc.OidcScopes;
import org.springframework.security.oauth2.server.authorization.token.JwtEncodingContext;
import org.springframework.stereotype.Component;

/** Identity claims for the client that logged the user in; email only when the email scope was granted. */
@Component
class IdTokenClaims {

    private final AccountService accounts;

    IdTokenClaims(AccountService accounts) {
        this.accounts = accounts;
    }

    void apply(JwtEncodingContext context) {
        if (!context.getAuthorizedScopes().contains(OidcScopes.EMAIL)) {
            return;
        }
        accounts.find(UUID.fromString(context.getPrincipal().getName()))
                .ifPresent(a -> context.getClaims().claim("email", a.email()));
    }
}
