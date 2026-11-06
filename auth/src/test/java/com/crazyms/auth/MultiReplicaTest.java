package com.crazyms.auth;

import static org.assertj.core.api.Assertions.assertThat;

import java.net.URI;
import java.util.ArrayList;
import org.junit.jupiter.api.AfterAll;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.boot.builder.SpringApplicationBuilder;
import org.springframework.context.ConfigurableApplicationContext;
import org.springframework.web.client.RestClient;
import org.testcontainers.postgresql.PostgreSQLContainer;

/** Two instances on one database, as behind a load balancer: any step may hit either. */
class MultiReplicaTest extends AuthTestSupport {

  private static ConfigurableApplicationContext replicaB;

  @Autowired PostgreSQLContainer postgres;

  String urlB;
  RestClient httpB;

  @BeforeEach
  void startReplicaB() {
    if (replicaB == null) {
      var args = new ArrayList<String>();
      TestcontainersConfiguration.databaseProperties(postgres)
          .forEach((k, v) -> args.add("--" + k + "=" + v));
      TestcontainersConfiguration.authProperties().forEach((k, v) -> args.add("--" + k + "=" + v));
      args.add("--server.port=0");
      replicaB =
          new SpringApplicationBuilder(AuthApplication.class).run(args.toArray(String[]::new));
    }
    urlB = "http://localhost:" + replicaB.getEnvironment().getProperty("local.server.port");
    httpB = RestClient.builder().baseUrl(urlB).build();
  }

  @AfterAll
  static void stopReplicaB() {
    if (replicaB != null) {
      replicaB.close();
    }
  }

  @Test
  void everyStepOfALoginCanLandOnAnyReplica() throws Exception {
    String email = uniqueEmail();
    String accountId = (String) register(email, PASSWORD).body().get("id");
    var browser = new TestBrowser();
    var pkce = TestBrowser.pkce();

    // Authorize on A, log in on B: saved request and CSRF token live in the shared session.
    browser.startAuthorization(baseUrl(), pkce);
    var afterLogin = browser.submitLogin(urlB, email, PASSWORD);
    assertThat(afterLogin.statusCode()).isEqualTo(302);

    // Resume on B, exchange the code on A: the authorization lives in the shared table.
    var resume = TestBrowser.location(urlB, afterLogin);
    String code =
        browser.followToCode(URI.create(urlB + resume.getRawPath() + "?" + resume.getRawQuery()));
    var tokens = exchange(http, code, pkce.verifier());
    assertThat(tokens.status()).isEqualTo(200);

    // refresh on B, call /userinfo on A with the refreshed token
    var refreshed = refresh(httpB, (String) tokens.body().get("refresh_token"));
    assertThat(refreshed.status()).isEqualTo(200);
    var userinfo = get("/userinfo", (String) refreshed.body().get("access_token"));
    assertThat(userinfo.status()).isEqualTo(200);
    assertThat(userinfo.body()).containsEntry("sub", accountId);
  }
}
