package com.crazyms.auth;

import static org.assertj.core.api.Assertions.assertThat;

import org.junit.jupiter.api.Test;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.jdbc.core.simple.JdbcClient;

class LoginInputLimitTest extends AuthTestSupport {

  @Autowired JdbcClient jdbc;

  @Test
  void oversizedPasswordFailsBeforeAuthenticationRuns() throws Exception {
    String email = uniqueEmail();
    register(email, PASSWORD);

    var res = new TestBrowser().submitLogin(baseUrl(), email, "x".repeat(129));

    assertThat(res.statusCode()).isEqualTo(302);
    assertThat(TestBrowser.location(baseUrl(), res).toString()).endsWith("/login?error");
    assertThat(attemptsFor(email)).as("rejected before Spring Security saw it").isZero();
  }

  @Test
  void oversizedUsernameFailsBeforeAuthenticationRuns() throws Exception {
    String huge = "a".repeat(250) + "@example.com";

    var res = new TestBrowser().submitLogin(baseUrl(), huge, PASSWORD);

    assertThat(TestBrowser.location(baseUrl(), res).toString()).endsWith("/login?error");
    assertThat(attemptsFor(huge.substring(0, 50))).isZero();
  }

  @Test
  void passwordAtTheLimitStillReachesAuthentication() throws Exception {
    String email = uniqueEmail();
    register(email, PASSWORD);

    new TestBrowser().submitLogin(baseUrl(), email, "x".repeat(128));

    assertThat(attemptsFor(email)).as("a 128-character password is a normal failed login").isOne();
  }

  private long attemptsFor(String identifierPrefix) {
    return jdbc.sql(
            "SELECT COUNT(*) FROM authentication_attempts WHERE submitted_identifier LIKE :p")
        .param("p", identifierPrefix + "%")
        .query(Long.class)
        .single();
  }
}
