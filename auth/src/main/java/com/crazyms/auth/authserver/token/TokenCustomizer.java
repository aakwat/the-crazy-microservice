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

  static final String ACCESS_TOKEN_TYPE = "at+jwt";

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
      // RFC 9068: the type header is what stops an ID token being replayed as an access token.
      context.getJwsHeader().type(ACCESS_TOKEN_TYPE);
      accessTokenClaims.apply(context);
    } else if (OidcParameterNames.ID_TOKEN.equals(context.getTokenType().getValue())) {
      idTokenClaims.apply(context);
    }
  }
}
