package com.crazyms.auth.authserver.client;

import com.crazyms.auth.authserver.client.ClientProperties.Registration;
import com.crazyms.auth.authserver.token.TokenProperties;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.springframework.security.crypto.password.PasswordEncoder;
import org.springframework.security.oauth2.core.AuthorizationGrantType;
import org.springframework.security.oauth2.core.ClientAuthenticationMethod;
import org.springframework.security.oauth2.jose.jws.SignatureAlgorithm;
import org.springframework.security.oauth2.server.authorization.client.InMemoryRegisteredClientRepository;
import org.springframework.security.oauth2.server.authorization.client.RegisteredClient;
import org.springframework.security.oauth2.server.authorization.client.RegisteredClientRepository;
import org.springframework.security.oauth2.server.authorization.settings.ClientSettings;
import org.springframework.security.oauth2.server.authorization.settings.TokenSettings;

/** Clients come from config, identical on every replica, so an in-memory repository is safe to scale. */
@Configuration(proxyBeanMethods = false)
class ClientRegistrationConfig {

    @Bean
    RegisteredClientRepository registeredClientRepository(
            ClientProperties clients, TokenProperties tokens, PasswordEncoder encoder) {
        return new InMemoryRegisteredClientRepository(
                clients.registrations().stream().map(c -> toRegisteredClient(c, tokens, encoder)).toList());
    }

    private static RegisteredClient toRegisteredClient(Registration c, TokenProperties tokens, PasswordEncoder encoder) {
        // The internal id must match across replicas: stored authorizations reference it.
        var builder = RegisteredClient.withId(c.id())
                .clientId(c.id())
                .clientSecret(encoder.encode(c.secret()))
                .clientAuthenticationMethod(ClientAuthenticationMethod.CLIENT_SECRET_BASIC);
        c.grantTypes().forEach(g -> builder.authorizationGrantType(new AuthorizationGrantType(g)));
        c.redirectUris().forEach(builder::redirectUri);
        c.scopes().forEach(builder::scope);
        return builder
                .clientSettings(ClientSettings.builder()
                        .requireProofKey(true)
                        .requireAuthorizationConsent(false)
                        .build())
                .tokenSettings(TokenSettings.builder()
                        .accessTokenTimeToLive(tokens.accessTtl())
                        .refreshTokenTimeToLive(tokens.refreshTtl())
                        // Rotate refresh tokens: each use returns a new one, so a stolen token works at most once.
                        .reuseRefreshTokens(false)
                        .idTokenSignatureAlgorithm(SignatureAlgorithm.ES256)
                        .build())
                .build();
    }
}
