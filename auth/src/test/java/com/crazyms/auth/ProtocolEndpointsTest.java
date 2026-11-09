package com.crazyms.auth;

import static org.assertj.core.api.Assertions.assertThat;

import com.nimbusds.jwt.SignedJWT;
import java.net.URI;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;
import java.util.Map;
import org.junit.jupiter.api.Test;

/** What Spring Authorization Server provides by default, pinned so an upgrade cannot drop it. */
class ProtocolEndpointsTest extends AuthTestSupport {

  @Test
  void revokedRefreshTokenCannotBeUsed() throws Exception {
    String refresh = (String) browserLoginTokens().get("refresh_token");

    var revoke =
        postForm(
            "/oauth2/revoke",
            TestClients.GATEWAY,
            TestClients.GATEWAY_SECRET,
            Map.of("token", refresh, "token_type_hint", "refresh_token"));

    assertThat(revoke.status()).isEqualTo(200);
    var reuse = refresh(http, refresh);
    assertThat(reuse.status()).isEqualTo(400);
    assertThat(reuse.body()).containsEntry("error", "invalid_grant");
  }

  @Test
  void introspectionReportsActiveThenInactiveAfterRevocation() throws Exception {
    String access = (String) browserLoginTokens().get("access_token");
    var form = Map.of("token", access);

    var before =
        postForm("/oauth2/introspect", TestClients.GATEWAY, TestClients.GATEWAY_SECRET, form);
    postForm("/oauth2/revoke", TestClients.GATEWAY, TestClients.GATEWAY_SECRET, form);
    var after =
        postForm("/oauth2/introspect", TestClients.GATEWAY, TestClients.GATEWAY_SECRET, form);

    assertThat(before.body()).containsEntry("active", true);
    assertThat(after.body()).containsEntry("active", false);
  }

  @Test
  void authorizationServerMetadataAdvertisesRevocationAndIntrospection() {
    var res = get("/.well-known/oauth-authorization-server", null);

    assertThat(res.status()).isEqualTo(200);
    assertThat(res.body())
        .containsEntry("issuer", TestClients.ISSUER)
        .containsEntry("revocation_endpoint", TestClients.ISSUER + "/oauth2/revoke")
        .containsEntry("introspection_endpoint", TestClients.ISSUER + "/oauth2/introspect");
  }

  @Test
  void accessTokensAreTypedAndIdTokensAreNotAcceptedAsAccessTokens() throws Exception {
    var tokens = browserLoginTokens();
    var access = SignedJWT.parse((String) tokens.get("access_token"));

    assertThat(access.getHeader().getType()).hasToString("at+jwt");
    // RFC 9068: a validly signed ID token must still be refused where an access token is required.
    assertThat(get("/userinfo", (String) tokens.get("id_token")).status()).isEqualTo(401);
    assertThat(get("/userinfo", (String) tokens.get("access_token")).status()).isEqualTo(200);
  }

  @Test
  void logoutRedirectsOnlyToARegisteredPostLogoutUri() throws Exception {
    var login = browserLogin();
    String idToken = (String) login.tokens().get("id_token");

    var unregistered = login.browser().get(logoutUrl(idToken, "http://evil.test/"));
    var registered = login.browser().get(logoutUrl(idToken, TestClients.POST_LOGOUT_URI));

    assertThat(unregistered.statusCode()).isEqualTo(400);
    assertThat(registered.statusCode()).isEqualTo(302);
    assertThat(registered.headers().firstValue("Location")).hasValue(TestClients.POST_LOGOUT_URI);
  }

  @Test
  void redirectsUseTheForwardedPublicOrigin() throws Exception {
    var pkce = TestBrowser.pkce();
    var authorize =
        URI.create(
            baseUrl()
                + "/oauth2/authorize?response_type=code&scope=openid&state=s&client_id="
                + TestClients.GATEWAY
                + "&redirect_uri="
                + TestBrowser.enc(TestClients.REDIRECT_URI)
                + "&code_challenge_method=S256&code_challenge="
                + pkce.challenge());

    var res =
        HttpClient.newHttpClient()
            .send(
                HttpRequest.newBuilder(authorize)
                    .header("Accept", "text/html")
                    .header("X-Forwarded-Host", "127.0.0.1:8080")
                    .header("X-Forwarded-Proto", "http")
                    .GET()
                    .build(),
                HttpResponse.BodyHandlers.discarding());

    assertThat(res.statusCode()).isEqualTo(302);
    assertThat(res.headers().firstValue("Location")).hasValue("http://127.0.0.1:8080/login");
  }

  private String logoutUrl(String idTokenHint, String postLogoutUri) {
    return baseUrl()
        + "/connect/logout?id_token_hint="
        + TestBrowser.enc(idTokenHint)
        + "&post_logout_redirect_uri="
        + TestBrowser.enc(postLogoutUri);
  }

  @Test
  void loginPageSendsSecurityHeaders() throws Exception {
    var res =
        HttpClient.newHttpClient()
            .send(
                HttpRequest.newBuilder(URI.create(baseUrl() + "/login")).GET().build(),
                HttpResponse.BodyHandlers.discarding());

    assertThat(res.headers().firstValue("X-Content-Type-Options")).hasValue("nosniff");
    assertThat(res.headers().firstValue("X-Frame-Options")).hasValue("DENY");
    assertThat(res.headers().firstValue("Cache-Control").orElse("")).contains("no-store");
  }
}
