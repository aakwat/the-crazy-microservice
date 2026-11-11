package com.crazyms.auth;

import static org.assertj.core.api.Assertions.assertThat;

import com.crazyms.auth.authserver.domain.port.ExpiredAuthorizationRepository;
import java.time.Clock;
import java.time.Duration;
import java.util.Map;
import org.junit.jupiter.api.Test;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.jdbc.core.simple.JdbcClient;

class AuthorizationRetentionTest extends AuthTestSupport {

  @Autowired ExpiredAuthorizationRepository authorizations;

  @Autowired JdbcClient jdbc;

  @Autowired Clock clock;

  @Test
  void removesAuthorizationsOnlyOnceEveryTokenInThemHasExpired() throws Exception {
    // A service token: one 15-minute access token, nothing else.
    token(TestClients.JOBS, TestClients.JOBS_SECRET, Map.of("grant_type", "client_credentials"));

    // A browser login: the access token expires in 15 minutes, but the refresh token lives 8 hours.
    String email = uniqueEmail();
    String accountId = (String) register(email, PASSWORD).body().get("id");
    var browser = new TestBrowser();
    var pkce = TestBrowser.pkce();
    browser.startAuthorization(baseUrl(), pkce);
    var afterLogin = browser.submitLogin(baseUrl(), email, PASSWORD);
    String code = browser.followToCode(TestBrowser.location(baseUrl(), afterLogin));
    assertThat(exchange(http, code, pkce.verifier()).status()).isEqualTo(200);

    // 16 minutes later every access token is dead, but the refresh token is not.
    authorizations.deleteExpiredBefore(clock.instant().plus(Duration.ofMinutes(16)));

    assertThat(rowsFor(TestClients.JOBS)).isZero();
    assertThat(rowsFor(accountId)).as("a live refresh token keeps its authorization").isOne();
  }

  private long rowsFor(String principal) {
    return jdbc.sql("SELECT COUNT(*) FROM oauth2_authorization WHERE principal_name = :p")
        .param("p", principal)
        .query(Long.class)
        .single();
  }
}
