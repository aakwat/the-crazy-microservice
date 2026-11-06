package com.crazyms.auth;

import java.io.IOException;
import java.io.UncheckedIOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.security.GeneralSecurityException;
import java.security.KeyPairGenerator;
import java.security.spec.ECGenParameterSpec;
import java.util.Base64;
import org.springframework.boot.test.context.TestConfiguration;
import org.springframework.boot.testcontainers.service.connection.ServiceConnection;
import org.springframework.context.annotation.Bean;
import org.springframework.test.context.DynamicPropertyRegistrar;
import org.testcontainers.postgresql.PostgreSQLContainer;
import org.testcontainers.utility.DockerImageName;

@TestConfiguration(proxyBeanMethods = false)
class TestcontainersConfiguration {

    @Bean
    @ServiceConnection
    PostgreSQLContainer postgresContainer() {
        // Same major as compose.yaml: uuidv7() needs Postgres 18.
        return new PostgreSQLContainer(DockerImageName.parse("postgres:18-alpine"));
    }

    @Bean
    DynamicPropertyRegistrar authTestProperties(PostgreSQLContainer postgres) {
        Path keys = writeSigningKeys();
        return registry -> {
            registry.add("auth.private-key-path", () -> keys.resolve("private.pem").toString());
            registry.add("auth.public-key-path", () -> keys.resolve("public.pem").toString());
            registry.add("spring.flyway.user", postgres::getUsername);
            registry.add("spring.flyway.password", postgres::getPassword);
            registry.add("spring.security.oauth2.authorizationserver.issuer", () -> TestClients.ISSUER);
            registry.add("auth.clients[0].id", () -> TestClients.GATEWAY);
            registry.add("auth.clients[0].secret", () -> TestClients.GATEWAY_SECRET);
            registry.add("auth.clients[0].grant-types[0]", () -> "authorization_code");
            registry.add("auth.clients[0].grant-types[1]", () -> "refresh_token");
            registry.add("auth.clients[0].redirect-uris[0]", () -> TestClients.REDIRECT_URI);
            registry.add("auth.clients[0].scopes[0]", () -> "openid");
            registry.add("auth.clients[0].scopes[1]", () -> "email");
            registry.add("auth.clients[1].id", () -> TestClients.JOBS);
            registry.add("auth.clients[1].secret", () -> TestClients.JOBS_SECRET);
            registry.add("auth.clients[1].grant-types[0]", () -> "client_credentials");
            registry.add("auth.clients[1].scopes[0]", () -> "tickets.read");
        };
    }

    private static Path writeSigningKeys() {
        try {
            var generator = KeyPairGenerator.getInstance("EC");
            generator.initialize(new ECGenParameterSpec("secp256r1"));
            var pair = generator.generateKeyPair();
            Path dir = Files.createTempDirectory("auth-test-keys");
            Files.writeString(dir.resolve("private.pem"), pem("PRIVATE KEY", pair.getPrivate().getEncoded()));
            Files.writeString(dir.resolve("public.pem"), pem("PUBLIC KEY", pair.getPublic().getEncoded()));
            return dir;
        } catch (IOException e) {
            throw new UncheckedIOException(e);
        } catch (GeneralSecurityException e) {
            throw new IllegalStateException(e);
        }
    }

    private static String pem(String type, byte[] der) {
        var body = Base64.getMimeEncoder(64, "\n".getBytes()).encodeToString(der);
        return "-----BEGIN " + type + "-----\n" + body + "\n-----END " + type + "-----\n";
    }
}
