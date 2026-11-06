package com.crazyms.auth.authserver.token;

import java.util.ArrayList;
import java.util.Collections;
import java.util.stream.Collectors;
import org.springframework.security.core.GrantedAuthority;
import org.springframework.security.oauth2.server.authorization.token.JwtEncodingContext;
import org.springframework.stereotype.Component;

/** What resource servers read from an access token: the audience and, for users, roles. */
@Component
class AccessTokenClaims {

  private static final String ROLE_PREFIX = "ROLE_";

  private final TokenProperties tokens;

  AccessTokenClaims(TokenProperties tokens) {
    this.tokens = tokens;
  }

  // SAS reads stored claims back through an allow-list that rejects List.of and toList().
  void apply(JwtEncodingContext context) {
    // Resource servers check this audience, not each caller's client id.
    context.getClaims().audience(Collections.singletonList(tokens.audience()));
    var roles =
        context.getPrincipal().getAuthorities().stream()
            .map(GrantedAuthority::getAuthority)
            .filter(a -> a.startsWith(ROLE_PREFIX))
            .map(a -> a.substring(ROLE_PREFIX.length()))
            .collect(Collectors.toCollection(ArrayList::new));
    if (!roles.isEmpty()) {
      context.getClaims().claim("roles", roles);
    }
  }
}
