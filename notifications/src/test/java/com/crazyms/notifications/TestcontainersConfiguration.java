package com.crazyms.notifications;

import java.io.IOException;
import org.springframework.boot.test.context.TestConfiguration;
import org.springframework.context.annotation.Bean;
import org.springframework.test.context.DynamicPropertyRegistrar;
import org.testcontainers.containers.GenericContainer;
import org.testcontainers.containers.wait.strategy.Wait;
import org.testcontainers.images.builder.Transferable;
import org.testcontainers.postgresql.PostgreSQLContainer;
import org.testcontainers.utility.DockerImageName;
import org.testcontainers.utility.MountableFile;

/** Real Postgres, Redis and Mailpit, plus an auth stub; the same images the compose stack runs. */
@TestConfiguration(proxyBeanMethods = false)
class TestcontainersConfiguration {

  static final String APP_PASSWORD = "app-test-password";
  static final String MIGRATOR_PASSWORD = "migrator-test-password";
  static final String REDIS_PASSWORD = "redis-test-password";

  // Runs the real roles script: tests connect as notifications_app and migrate as the migrator.
  @Bean
  PostgreSQLContainer postgresContainer() {
    return new PostgreSQLContainer(DockerImageName.parse("postgres:18-alpine"))
        .withDatabaseName("notifications")
        .withCopyFileToContainer(
            MountableFile.forHostPath("db/init/01-roles.sql"),
            "/docker-entrypoint-initdb.d/01-roles.sql")
        .withCopyToContainer(
            Transferable.of(APP_PASSWORD), "/run/secrets/NOTIFICATIONS_DB_APP_PASSWORD")
        .withCopyToContainer(
            Transferable.of(MIGRATOR_PASSWORD), "/run/secrets/NOTIFICATIONS_DB_MIGRATOR_PASSWORD");
  }

  @Bean
  GenericContainer<?> redisContainer() {
    return new GenericContainer<>("redis:8.10.3-alpine")
        .withCommand("redis-server", "--requirepass", REDIS_PASSWORD)
        .withExposedPorts(6379);
  }

  @Bean
  GenericContainer<?> mailpitContainer() {
    return new GenericContainer<>("axllent/mailpit:v1.33.1")
        .withExposedPorts(1025, 8025)
        .waitingFor(Wait.forHttp("/livez").forPort(8025));
  }

  @Bean(destroyMethod = "close")
  AuthStub authStub() throws IOException {
    return new AuthStub();
  }

  @Bean(destroyMethod = "close")
  TestBus testBus(GenericContainer<?> redisContainer) {
    return new TestBus(
        redisContainer.getHost(), redisContainer.getMappedPort(6379), REDIS_PASSWORD);
  }

  @Bean
  TestMailbox testMailbox(GenericContainer<?> mailpitContainer) {
    return new TestMailbox(
        "http://" + mailpitContainer.getHost() + ":" + mailpitContainer.getMappedPort(8025));
  }

  @Bean
  DynamicPropertyRegistrar notificationsTestProperties(
      PostgreSQLContainer postgresContainer,
      GenericContainer<?> redisContainer,
      GenericContainer<?> mailpitContainer,
      AuthStub authStub) {
    return registry -> {
      registry.add("spring.datasource.url", postgresContainer::getJdbcUrl);
      registry.add("spring.datasource.username", () -> "notifications_app");
      registry.add("spring.datasource.password", () -> APP_PASSWORD);
      registry.add("spring.flyway.user", () -> "notifications_migrator");
      registry.add("spring.flyway.password", () -> MIGRATOR_PASSWORD);
      registry.add("spring.mail.host", mailpitContainer::getHost);
      registry.add("spring.mail.port", () -> mailpitContainer.getMappedPort(1025));
      registry.add(
          "spring.security.oauth2.client.registration.auth.client-secret", () -> "stub-secret");
      registry.add(
          "spring.security.oauth2.client.provider.auth.token-uri",
          () -> authStub.url() + "/oauth2/token");
      registry.add("notifications.auth.url", authStub::url);
      registry.add("notifications.inbox.host", redisContainer::getHost);
      registry.add("notifications.inbox.port", () -> redisContainer.getMappedPort(6379));
      registry.add("notifications.inbox.user", () -> "default");
      registry.add("notifications.inbox.password", () -> REDIS_PASSWORD);
      // Short, so the retry and dead-letter tests finish in seconds instead of minutes.
      registry.add("notifications.inbox.stuck-after", () -> "300ms");
    };
  }
}
