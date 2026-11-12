package com.crazyms.auth;

import java.io.IOException;
import java.io.UncheckedIOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.security.GeneralSecurityException;
import java.security.KeyPairGenerator;
import java.security.spec.ECGenParameterSpec;
import java.util.Base64;
import java.util.LinkedHashMap;
import java.util.Map;
import org.springframework.boot.test.context.TestConfiguration;
import org.springframework.context.annotation.Bean;
import org.springframework.test.context.DynamicPropertyRegistrar;
import org.testcontainers.images.builder.Transferable;
import org.testcontainers.postgresql.PostgreSQLContainer;
import org.testcontainers.utility.DockerImageName;
import org.testcontainers.utility.MountableFile;

@TestConfiguration(proxyBeanMethods = false)
class TestcontainersConfiguration {

  static final String APP_USER = "auth_app";
  static final String APP_PASSWORD = "app-test-password";
  static final String MIGRATOR_USER = "auth_migrator";
  static final String MIGRATOR_PASSWORD = "migrator-test-password";

  // One key pair per JVM, so a second test instance signs exactly like a real replica.
  private static final Path KEYS = writeSigningKeys();

  // Runs the real roles script: tests connect as auth_app and migrate as auth_migrator.
  @Bean
  PostgreSQLContainer postgresContainer() {
    return new PostgreSQLContainer(DockerImageName.parse("postgres:18-alpine"))
        .withDatabaseName("auth")
        .withCopyFileToContainer(
            MountableFile.forHostPath("db/init/01-roles.sh", 0755),
            "/docker-entrypoint-initdb.d/01-roles.sh")
        .withCopyToContainer(Transferable.of(APP_PASSWORD), "/run/secrets/AUTH_DB_APP_PASSWORD")
        .withCopyToContainer(
            Transferable.of(MIGRATOR_PASSWORD), "/run/secrets/AUTH_DB_MIGRATOR_PASSWORD");
  }

  @Bean
  DynamicPropertyRegistrar authTestProperties(PostgreSQLContainer postgres) {
    return registry -> {
      databaseProperties(postgres).forEach((key, value) -> registry.add(key, () -> value));
      authProperties().forEach((key, value) -> registry.add(key, () -> value));
    };
  }

  static Map<String, String> databaseProperties(PostgreSQLContainer postgres) {
    var p = new LinkedHashMap<String, String>();
    p.put("spring.datasource.url", postgres.getJdbcUrl());
    p.put("spring.datasource.username", APP_USER);
    p.put("spring.datasource.password", APP_PASSWORD);
    p.put("spring.flyway.user", MIGRATOR_USER);
    p.put("spring.flyway.password", MIGRATOR_PASSWORD);
    return p;
  }

  /** Everything an auth instance needs besides its database connection. */
  static Map<String, String> authProperties() {
    var p = new LinkedHashMap<String, String>();
    p.put("auth.key.private-path", KEYS.resolve("private.pem").toString());
    p.put("auth.key.public-path", KEYS.resolve("public.pem").toString());
    p.put("spring.security.oauth2.authorizationserver.issuer", TestClients.ISSUER);
    // Small numbers so the lockout tests run in seconds.
    p.put("auth.login.lockout.max-failures", "3");
    p.put("auth.login.lockout.duration", "2s");
    p.put("auth.client.registrations[0].id", TestClients.GATEWAY);
    p.put("auth.client.registrations[0].secret", TestClients.GATEWAY_SECRET);
    p.put("auth.client.registrations[0].grant-types[0]", "authorization_code");
    p.put("auth.client.registrations[0].grant-types[1]", "refresh_token");
    p.put("auth.client.registrations[0].redirect-uris[0]", TestClients.REDIRECT_URI);
    p.put("auth.client.registrations[0].post-logout-redirect-uris[0]", TestClients.POST_LOGOUT_URI);
    p.put("auth.client.registrations[0].scopes[0]", "openid");
    p.put("auth.client.registrations[0].scopes[1]", "email");
    p.put("auth.client.registrations[1].id", TestClients.JOBS);
    p.put("auth.client.registrations[1].secret", TestClients.JOBS_SECRET);
    p.put("auth.client.registrations[1].grant-types[0]", "client_credentials");
    p.put("auth.client.registrations[1].scopes[0]", "tickets.read");
    p.put("auth.client.registrations[2].id", TestClients.NOTIFICATIONS);
    p.put("auth.client.registrations[2].secret", TestClients.NOTIFICATIONS_SECRET);
    p.put("auth.client.registrations[2].grant-types[0]", "client_credentials");
    p.put("auth.client.registrations[2].scopes[0]", "accounts.read");
    return p;
  }

  private static Path writeSigningKeys() {
    try {
      var generator = KeyPairGenerator.getInstance("EC");
      generator.initialize(new ECGenParameterSpec("secp256r1"));
      var pair = generator.generateKeyPair();
      Path dir = Files.createTempDirectory("auth-test-keys");
      Files.writeString(
          dir.resolve("private.pem"), pem("PRIVATE KEY", pair.getPrivate().getEncoded()));
      Files.writeString(
          dir.resolve("public.pem"), pem("PUBLIC KEY", pair.getPublic().getEncoded()));
      return dir;
    } catch (IOException e) {
      throw new UncheckedIOException(e);
    } catch (GeneralSecurityException e) {
      throw new IllegalStateException(e);
    }
  }

  static String pem(String type, byte[] der) {
    var body = Base64.getMimeEncoder(64, "\n".getBytes()).encodeToString(der);
    return "-----BEGIN " + type + "-----\n" + body + "\n-----END " + type + "-----\n";
  }
}
