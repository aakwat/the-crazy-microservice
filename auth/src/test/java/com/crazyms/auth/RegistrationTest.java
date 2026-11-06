package com.crazyms.auth;

import static org.assertj.core.api.Assertions.assertThat;

import java.util.Map;
import java.util.UUID;
import org.junit.jupiter.api.Test;
import org.springframework.http.MediaType;

class RegistrationTest extends AuthTestSupport {

  @Test
  void registersWithNormalizedEmailDefaultRoleAndV7Id() {
    String email = uniqueEmail();

    var res = register(email.toUpperCase(), PASSWORD);

    assertThat(res.status()).isEqualTo(201);
    assertThat(res.body()).containsOnlyKeys("id", "email", "role");
    assertThat(res.body()).containsEntry("email", email).containsEntry("role", "customer");
    assertThat(UUID.fromString((String) res.body().get("id")).version()).isEqualTo(7);
  }

  @Test
  void duplicateEmailInAnyCaseIsConflict() {
    String email = uniqueEmail();
    assertThat(register(email, PASSWORD).status()).isEqualTo(201);

    var again = register(email.toUpperCase(), PASSWORD);

    assertThat(again.status()).isEqualTo(409);
    assertThat(again.headers().getContentType()).isEqualTo(MediaType.APPLICATION_PROBLEM_JSON);
  }

  @Test
  void invalidRegistrationIsBadRequest() {
    var shortPassword = register(uniqueEmail(), "short");
    assertThat(shortPassword.status()).isEqualTo(400);
    assertThat(shortPassword.headers().getContentType())
        .isEqualTo(MediaType.APPLICATION_PROBLEM_JSON);
    assertThat(shortPassword.body()).containsEntry("status", 400).containsKey("title");

    assertThat(register("not-an-email", PASSWORD).status()).isEqualTo(400);
    assertThat(postJson("/auth/register", Map.of("email", uniqueEmail())).status()).isEqualTo(400);
  }

  @Test
  void unknownFieldsAreRejectedNotIgnored() {
    // A client trying to pick its own role (mass assignment) must fail loudly.
    var res =
        postJson(
            "/auth/register",
            Map.of("email", uniqueEmail(), "password", PASSWORD, "role", "agent"));

    assertThat(res.status()).isEqualTo(400);
    assertThat(res.headers().getContentType()).isEqualTo(MediaType.APPLICATION_PROBLEM_JSON);
  }
}
