package com.crazyms.auth;

import static org.assertj.core.api.Assertions.assertThat;

import java.util.Map;
import org.junit.jupiter.api.Test;

class ClientCredentialsTest extends AuthTestSupport {

  @Test
  void discoveryAdvertisesTheFlowsOtherServicesRelyOn() {
    var res = get("/.well-known/openid-configuration", null);

    assertThat(res.status()).isEqualTo(200);
    assertThat(res.body()).containsEntry("issuer", TestClients.ISSUER);
    assertThat(res.body().get("jwks_uri")).isEqualTo(TestClients.ISSUER + "/oauth2/jwks");
    assertThat(res.body().get("grant_types_supported").toString())
        .contains("authorization_code", "client_credentials", "refresh_token");
    assertThat(res.body().get("code_challenge_methods_supported").toString()).contains("S256");
  }

  @Test
  void workerGetsAServiceTokenWithAudienceAndNoUserRoles() throws Exception {
    var res =
        token(
            TestClients.JOBS,
            TestClients.JOBS_SECRET,
            Map.of("grant_type", "client_credentials", "scope", "tickets.read"));

    assertThat(res.status()).isEqualTo(200);
    assertThat(res.body()).containsEntry("token_type", "Bearer").doesNotContainKey("refresh_token");
    var claims = verifyAgainstJwks((String) res.body().get("access_token"));
    assertThat(claims.getSubject()).isEqualTo(TestClients.JOBS);
    assertThat(claims.getAudience()).containsExactly("crazy-ms");
    assertThat(claims.getClaim("scope").toString()).contains("tickets.read");
    assertThat(claims.getClaim("roles")).isNull();
  }

  @Test
  void wrongSecretOrUnknownScopeIsRejected() {
    var wrongSecret =
        token(TestClients.JOBS, "not-the-secret", Map.of("grant_type", "client_credentials"));
    var unknownScope =
        token(
            TestClients.JOBS,
            TestClients.JOBS_SECRET,
            Map.of("grant_type", "client_credentials", "scope", "tickets.admin"));

    // OAuth endpoints keep RFC 6749 errors, not problem+json: OAuth libraries parse them.
    assertThat(wrongSecret.status()).isEqualTo(401);
    assertThat(wrongSecret.body()).containsEntry("error", "invalid_client");
    assertThat(unknownScope.status()).isEqualTo(400);
    assertThat(unknownScope.body()).containsEntry("error", "invalid_scope");
  }
}
