package com.crazyms.auth.authserver;

import com.crazyms.auth.account.service.AccountService;
import com.crazyms.auth.config.AuthProperties;
import java.util.ArrayList;
import java.util.Collections;
import java.util.UUID;
import java.util.stream.Collectors;
import org.springframework.security.core.GrantedAuthority;
import org.springframework.security.oauth2.core.oidc.OidcScopes;
import org.springframework.security.oauth2.core.oidc.endpoint.OidcParameterNames;
import org.springframework.security.oauth2.jose.jws.SignatureAlgorithm;
import org.springframework.security.oauth2.server.authorization.OAuth2TokenType;
import org.springframework.security.oauth2.server.authorization.token.JwtEncodingContext;
import org.springframework.security.oauth2.server.authorization.token.OAuth2TokenCustomizer;
import org.springframework.stereotype.Component;

/** Shapes every token this server signs: ES256, the shared audience, roles, and email when consented. */
@Component
class TokenClaimsCustomizer implements OAuth2TokenCustomizer<JwtEncodingContext> {

    private static final String ROLE_PREFIX = "ROLE_";

    private final AuthProperties props;
    private final AccountService accounts;

    TokenClaimsCustomizer(AuthProperties props, AccountService accounts) {
        this.props = props;
        this.accounts = accounts;
    }

    @Override
    public void customize(JwtEncodingContext context) {
        context.getJwsHeader().algorithm(SignatureAlgorithm.ES256);

        // Stored claims are read back through an allow-list that rejects List.of and toList() collections.
        if (OAuth2TokenType.ACCESS_TOKEN.equals(context.getTokenType())) {
            // Resource servers check this audience, not each caller's client id.
            context.getClaims().audience(Collections.singletonList(props.audience()));
            var roles = context.getPrincipal().getAuthorities().stream()
                    .map(GrantedAuthority::getAuthority)
                    .filter(a -> a.startsWith(ROLE_PREFIX))
                    .map(a -> a.substring(ROLE_PREFIX.length()))
                    .collect(Collectors.toCollection(ArrayList::new));
            if (!roles.isEmpty()) {
                context.getClaims().claim("roles", roles);
            }
        }

        if (OidcParameterNames.ID_TOKEN.equals(context.getTokenType().getValue())
                && context.getAuthorizedScopes().contains(OidcScopes.EMAIL)) {
            accounts.find(UUID.fromString(context.getPrincipal().getName()))
                    .ifPresent(a -> context.getClaims().claim("email", a.email()));
        }
    }
}
