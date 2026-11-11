package com.crazyms.auth;

import static org.assertj.core.api.Assertions.assertThat;

import java.util.Map;
import java.util.UUID;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.http.MediaType;
import org.springframework.jdbc.core.simple.JdbcClient;

/** Role management through /auth/accounts: admins only, audited, live at the next refresh. */
class AccountAdminTest extends AuthTestSupport {

  @Autowired JdbcClient jdbc;

  UUID adminId;
  Map<String, Object> adminTokens;

  @BeforeEach
  void logInAsAdmin() throws Exception {
    String email = uniqueEmail();
    adminId = UUID.fromString((String) register(email, PASSWORD).body().get("id"));
    // Only the first admin is made with SQL; every later role change goes through the API.
    jdbc.sql("UPDATE account SET role = 'admin' WHERE id = :id").param("id", adminId).update();
    adminTokens = logIn(email).tokens();
  }

  @Test
  void adminPromotesAndTheNewRoleArrivesWithTheNextRefresh() throws Exception {
    String email = uniqueEmail();
    String id = (String) register(email, PASSWORD).body().get("id");
    var customer = logIn(email);

    var found = get("/auth/accounts?email=" + email.toUpperCase(), admin());
    var changed = putRole(id, Map.of("role", "agent"), admin());
    var refreshed = refresh(http, (String) customer.tokens().get("refresh_token"));

    assertThat(found.status()).isEqualTo(200);
    assertThat(found.body()).containsEntry("id", id).containsEntry("role", "customer");
    assertThat(changed.status()).isEqualTo(200);
    assertThat(changed.body()).containsEntry("id", id).containsEntry("role", "agent");
    var access = verifyAgainstJwks((String) refreshed.body().get("access_token"));
    assertThat(access.getStringListClaim("roles")).containsExactly("agent");
    var audit =
        jdbc.sql(
                "SELECT actor_id, from_role, to_role FROM account_role_change WHERE account_id = :id")
            .param("id", UUID.fromString(id))
            .query()
            .singleRow();
    assertThat(audit)
        .containsEntry("actor_id", adminId)
        .containsEntry("from_role", "customer")
        .containsEntry("to_role", "agent");
  }

  @Test
  void onlyAdminAccessTokensGetIn() throws Exception {
    String customer = (String) browserLoginTokens().get("access_token");
    String anyId = UUID.randomUUID().toString();
    var agent = Map.of("role", "agent");

    assertThat(putRole(anyId, agent, customer).status()).isEqualTo(403);
    assertThat(get("/auth/accounts?email=a@example.com", customer).status()).isEqualTo(403);
    assertThat(putRole(anyId, agent, null).status()).isEqualTo(401);
    // RFC 9068: an ID token carries the same signature but must never pass as an access token.
    assertThat(putRole(anyId, agent, (String) adminTokens.get("id_token")).status()).isEqualTo(401);
  }

  @Test
  void refusedChangesAreProblems() {
    String id = (String) register(uniqueEmail(), PASSWORD).body().get("id");

    assertProblem(putRole(id, Map.of("role", "root"), admin()), 422);
    assertProblem(putRole(UUID.randomUUID().toString(), Map.of("role", "agent"), admin()), 404);
    assertProblem(putRole(adminId.toString(), Map.of("role", "customer"), admin()), 409);
    assertProblem(putRole(id, Map.of("role", "agent", "email", "x@example.com"), admin()), 400);
    assertProblem(putRole("not-a-uuid", Map.of("role", "agent"), admin()), 400);
    assertProblem(get("/auth/accounts?email=nobody@example.com", admin()), 404);
  }

  private String admin() {
    return (String) adminTokens.get("access_token");
  }

  private Res putRole(String id, Map<String, String> body, String bearer) {
    return http.put()
        .uri("/auth/accounts/{id}/role", id)
        .headers(h -> {
          if (bearer != null) {
            h.setBearerAuth(bearer);
          }
        })
        .contentType(MediaType.APPLICATION_JSON)
        .body(body)
        .exchange(
            (req, res) -> new Res(res.getStatusCode().value(), res.bodyTo(JSON), res.getHeaders()));
  }

  private static void assertProblem(Res res, int status) {
    assertThat(res.status()).isEqualTo(status);
    assertThat(res.headers().getContentType()).isEqualTo(MediaType.APPLICATION_PROBLEM_JSON);
  }
}
