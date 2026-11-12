package com.crazyms.auth;

import static org.assertj.core.api.Assertions.assertThat;

import java.util.Map;
import java.util.UUID;
import org.junit.jupiter.api.Test;
import org.springframework.http.MediaType;

/** /internal/accounts: only a service token with accounts.read gets an email, and nothing more. */
class InternalAccountTest extends AuthTestSupport {

  @Test
  void aServiceTokenWithAccountsReadGetsTheEmailOnly() {
    String email = uniqueEmail();
    String id = (String) register(email, PASSWORD).body().get("id");

    var res = get("/internal/accounts/" + id, serviceToken());

    assertThat(res.status()).isEqualTo(200);
    assertThat(res.body()).containsOnlyKeys("id", "email").containsEntry("email", email);
  }

  @Test
  void anUnknownAccountIsAProblem404() {
    var res = get("/internal/accounts/" + UUID.randomUUID(), serviceToken());

    assertThat(res.status()).isEqualTo(404);
    assertThat(res.headers().getContentType()).isEqualTo(MediaType.APPLICATION_PROBLEM_JSON);
  }

  @Test
  void everyOtherCallerIsRefused() throws Exception {
    String path = "/internal/accounts/" + register(uniqueEmail(), PASSWORD).body().get("id");
    var user = browserLoginTokens();
    String otherService =
        (String)
            token(
                    TestClients.JOBS,
                    TestClients.JOBS_SECRET,
                    Map.of("grant_type", "client_credentials", "scope", "tickets.read"))
                .body()
                .get("access_token");

    assertThat(get(path, otherService).status()).isEqualTo(403);
    assertThat(get(path, (String) user.get("access_token")).status()).isEqualTo(403);
    assertThat(get(path, null).status()).isEqualTo(401);
    // RFC 9068: an ID token is signed by the same key but is never an access token.
    assertThat(get(path, (String) user.get("id_token")).status()).isEqualTo(401);
  }

  private String serviceToken() {
    var res =
        token(
            TestClients.NOTIFICATIONS,
            TestClients.NOTIFICATIONS_SECRET,
            Map.of("grant_type", "client_credentials", "scope", "accounts.read"));
    return (String) res.body().get("access_token");
  }
}
