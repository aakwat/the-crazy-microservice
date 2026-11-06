package com.crazyms.auth.authserver.token;

import com.nimbusds.jose.JOSEObjectType;
import com.nimbusds.jose.JWSAlgorithm;
import com.nimbusds.jose.jwk.source.JWKSource;
import com.nimbusds.jose.proc.DefaultJOSEObjectTypeVerifier;
import com.nimbusds.jose.proc.JWSVerificationKeySelector;
import com.nimbusds.jose.proc.SecurityContext;
import com.nimbusds.jwt.proc.DefaultJWTProcessor;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.springframework.security.oauth2.jwt.JwtDecoder;
import org.springframework.security.oauth2.jwt.JwtValidators;
import org.springframework.security.oauth2.jwt.NimbusJwtDecoder;
import org.springframework.security.oauth2.server.authorization.settings.AuthorizationServerSettings;

/** Verifies this server's own access tokens (for /userinfo) the way every service does. */
@Configuration(proxyBeanMethods = false)
class JwtDecoderConfig {

  @Bean
  JwtDecoder jwtDecoder(
      JWKSource<SecurityContext> jwkSource,
      AuthorizationServerSettings settings,
      TokenProperties tokens) {
    var processor = new DefaultJWTProcessor<SecurityContext>();
    processor.setJWSKeySelector(new JWSVerificationKeySelector<>(JWSAlgorithm.ES256, jwkSource));
    processor.setJWSTypeVerifier(
        new DefaultJOSEObjectTypeVerifier<>(new JOSEObjectType(TokenCustomizer.ACCESS_TOKEN_TYPE)));
    var decoder = new NimbusJwtDecoder(processor);
    // RFC 9068 profile: typ at+jwt, issuer, audience, timestamps and the required claims.
    decoder.setJwtValidator(
        JwtValidators.createAtJwtValidator()
            .issuer(settings.getIssuer())
            .audience(tokens.audience())
            .build());
    return decoder;
  }
}
