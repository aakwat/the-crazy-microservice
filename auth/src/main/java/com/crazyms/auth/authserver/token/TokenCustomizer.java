package com.crazyms.auth.authserver.token;

import org.springframework.security.oauth2.core.oidc.endpoint.OidcParameterNames;
import org.springframework.security.oauth2.jose.jws.SignatureAlgorithm;
import org.springframework.security.oauth2.server.authorization.OAuth2TokenType;
import org.springframework.security.oauth2.server.authorization.token.JwtEncodingContext;
import org.springframework.security.oauth2.server.authorization.token.OAuth2TokenCustomizer;
import org.springframework.stereotype.Component;

/** SAS takes one customizer: it signs ES256 and delegates each token type's claims. */
@Component
class TokenCustomizer implements OAuth2TokenCustomizer<JwtEncodingContext> {

  private final AccessTokenClaims accessTokenClaims;
  private final IdTokenClaims idTokenClaims;

  TokenCustomizer(AccessTokenClaims accessTokenClaims, IdTokenClaims idTokenClaims) {
    this.accessTokenClaims = accessTokenClaims;
    this.idTokenClaims = idTokenClaims;
  }

  @Override
  public void customize(JwtEncodingContext context) {
    context.getJwsHeader().algorithm(SignatureAlgorithm.ES256);
    if (OAuth2TokenType.ACCESS_TOKEN.equals(context.getTokenType())) {
      accessTokenClaims.apply(context);
    } else if (OidcParameterNames.ID_TOKEN.equals(context.getTokenType().getValue())) {
      idTokenClaims.apply(context);
    }
  }
}
