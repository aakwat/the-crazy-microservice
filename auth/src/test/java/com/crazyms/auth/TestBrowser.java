package com.crazyms.auth;

import static org.assertj.core.api.Assertions.assertThat;

import java.net.CookieManager;
import java.net.URI;
import java.net.URLDecoder;
import java.net.URLEncoder;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;
import java.nio.charset.StandardCharsets;
import java.security.MessageDigest;
import java.security.SecureRandom;
import java.util.Base64;
import java.util.regex.Pattern;

/** A test browser: keeps cookies, never follows redirects, knows the login page's steps. */
final class TestBrowser {

  private static final Pattern CSRF = Pattern.compile("name=\"_csrf\"[^>]*value=\"([^\"]+)\"");
  private static final Pattern CODE = Pattern.compile("[?&]code=([^&]+)");

  record Pkce(String verifier, String challenge) {}

  private final HttpClient client =
      HttpClient.newBuilder()
          .cookieHandler(new CookieManager())
          .followRedirects(HttpClient.Redirect.NEVER)
          .build();

  /** GET /oauth2/authorize as a browser; an unauthenticated user must land on the login page. */
  void startAuthorization(String baseUrl, Pkce pkce) throws Exception {
    var authorize =
        URI.create(
            baseUrl
                + "/oauth2/authorize?response_type=code"
                + "&client_id="
                + TestClients.GATEWAY
                + "&scope="
                + enc("openid email")
                + "&redirect_uri="
                + enc(TestClients.REDIRECT_URI)
                + "&state=xyz"
                + "&code_challenge="
                + pkce.challenge()
                + "&code_challenge_method=S256");
    var res = send(HttpRequest.newBuilder(authorize).header("Accept", "text/html").GET().build());
    assertThat(res.statusCode()).isEqualTo(302);
    assertThat(location(baseUrl, res).getPath()).isEqualTo("/login");
  }

  /** Fetches the login page for its CSRF token, then posts the form, exactly as a person would. */
  HttpResponse<String> submitLogin(String baseUrl, String email, String password) throws Exception {
    var page = send(HttpRequest.newBuilder(URI.create(baseUrl + "/login")).GET().build());
    var csrf = CSRF.matcher(page.body());
    assertThat(csrf.find()).as("login page carries a CSRF token").isTrue();
    String form =
        "username=" + enc(email) + "&password=" + enc(password) + "&_csrf=" + enc(csrf.group(1));
    return send(
        HttpRequest.newBuilder(URI.create(baseUrl + "/login"))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .POST(HttpRequest.BodyPublishers.ofString(form))
            .build());
  }

  /** Follows the post-login redirect to /oauth2/authorize; returns the callback's code. */
  String followToCode(URI resumeAuthorization) throws Exception {
    var callback = send(HttpRequest.newBuilder(resumeAuthorization).GET().build());
    assertThat(callback.statusCode()).isEqualTo(302);
    var redirect = callback.headers().firstValue("Location").orElseThrow();
    assertThat(redirect).startsWith(TestClients.REDIRECT_URI).contains("state=xyz");
    var matcher = CODE.matcher(redirect);
    assertThat(matcher.find()).isTrue();
    return URLDecoder.decode(matcher.group(1), StandardCharsets.UTF_8);
  }

  String page(String url) throws Exception {
    var res = send(HttpRequest.newBuilder(URI.create(url)).GET().build());
    assertThat(res.statusCode())
        .as("GET %s -> %s", url, res.headers().firstValue("Location").orElse(""))
        .isEqualTo(200);
    return res.body();
  }

  HttpResponse<String> get(String url) throws Exception {
    return send(HttpRequest.newBuilder(URI.create(url)).GET().build());
  }

  static URI location(String baseUrl, HttpResponse<?> res) {
    return URI.create(baseUrl).resolve(res.headers().firstValue("Location").orElseThrow());
  }

  static Pkce pkce() throws Exception {
    byte[] random = new byte[32];
    new SecureRandom().nextBytes(random);
    var b64 = Base64.getUrlEncoder().withoutPadding();
    String verifier = b64.encodeToString(random);
    byte[] digest =
        MessageDigest.getInstance("SHA-256").digest(verifier.getBytes(StandardCharsets.US_ASCII));
    return new Pkce(verifier, b64.encodeToString(digest));
  }

  private HttpResponse<String> send(HttpRequest request) throws Exception {
    return client.send(request, HttpResponse.BodyHandlers.ofString());
  }

  static String enc(String value) {
    return URLEncoder.encode(value, StandardCharsets.UTF_8);
  }
}
