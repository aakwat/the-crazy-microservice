package com.crazyms.auth.authserver.key;

import com.nimbusds.jose.JOSEException;
import com.nimbusds.jose.jwk.JWKSet;
import com.nimbusds.jose.jwk.source.ImmutableJWKSet;
import com.nimbusds.jose.jwk.source.JWKSource;
import com.nimbusds.jose.proc.SecurityContext;
import java.io.IOException;
import java.security.GeneralSecurityException;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

/** The only place that knows where signing keys live; moving to a KMS or Vault means replacing just this class. */
@Configuration(proxyBeanMethods = false)
class SigningKeyConfig {

    // Loaded, never generated: every replica must sign with, and publish, the same key.
    @Bean
    JWKSource<SecurityContext> jwkSource(KeyProperties keys) throws IOException, GeneralSecurityException, JOSEException {
        return new ImmutableJWKSet<>(new JWKSet(PemKeyPair.load(keys.privatePath(), keys.publicPath())));
    }
}
