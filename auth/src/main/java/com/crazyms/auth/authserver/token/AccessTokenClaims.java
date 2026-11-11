package com.crazyms.auth.authserver.token;

import com.crazyms.auth.account.service.AccountService;
import java.util.Collections;
import java.util.UUID;
import org.springframework.security.oauth2.core.AuthorizationGrantType;
import org.springframework.security.oauth2.server.authorization.token.JwtEncodingContext;
import org.springframework.stereotype.Component;

/** What resource servers read from an access token: the audience and, for users, roles. */
@Component
class AccessTokenClaims {

  private final TokenProperties tokens;
  private final AccountService accounts;

  AccessTokenClaims(TokenProperties tokens, AccountService accounts) {
    this.tokens = tokens;
    this.accounts = accounts;
  }

  // SAS reads stored claims back through an allow-list that rejects List.of and toList().
  void apply(JwtEncodingContext context) {
    // Resource servers check this audience, not each caller's client id.
    context.getClaims().audience(Collections.singletonList(tokens.audience()));
    // RFC 9068 requires client_id: which client the token was issued to, separate from the user.
    context.getClaims().claim("client_id", context.getRegisteredClient().getClientId());
    if (AuthorizationGrantType.CLIENT_CREDENTIALS.equals(context.getAuthorizationGrantType())) {
      return;
    }
    // Read now, not from the login: a refresh must carry the role an admin set since.
    accounts
        .find(UUID.fromString(context.getPrincipal().getName()))
        .ifPresent(
            a -> context.getClaims().claim("roles", Collections.singletonList(a.role().name())));
  }
}
