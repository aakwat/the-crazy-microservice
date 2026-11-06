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
import java.util.Map;
import java.util.regex.Pattern;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

/** Drives the real browser flow over HTTP: authorize, login form with CSRF, code, token with the PKCE verifier. */
class AuthorizationCodeFlowTest extends AuthTestSupport {

    private static final Pattern CSRF = Pattern.compile("name=\"_csrf\"[^>]*value=\"([^\"]+)\"");
    private static final Pattern CODE = Pattern.compile("[?&]code=([^&]+)");

    HttpClient browser;
    String email;
    String accountId;

    record Pkce(String verifier, String challenge) {}

    @BeforeEach
    void setUpBrowserAndAccount() {
        browser = HttpClient.newBuilder()
                .cookieHandler(new CookieManager())
                .followRedirects(HttpClient.Redirect.NEVER)
                .build();
        email = uniqueEmail();
        accountId = (String) register(email, PASSWORD).body().get("id");
    }

    @Test
    void browserLoginIssuesVerifiableTokensForTheAccount() throws Exception {
        var pkce = pkce();
        var tokens = exchange(loginAndAuthorize(pkce), pkce.verifier());

        assertThat(tokens.status()).isEqualTo(200);
        assertThat(tokens.body()).containsKeys("access_token", "refresh_token", "id_token");
        assertThat(tokens.headers().getCacheControl()).contains("no-store");

        var access = verifyAgainstJwks((String) tokens.body().get("access_token"));
        assertThat(access.getSubject()).as("sub is the account id, never the email").isEqualTo(accountId);
        assertThat(access.getAudience()).containsExactly("crazy-ms");
        assertThat(access.getStringListClaim("roles")).containsExactly("customer");

        var id = verifyAgainstJwks((String) tokens.body().get("id_token"));
        assertThat(id.getSubject()).isEqualTo(accountId);
        assertThat(id.getStringClaim("email")).isEqualTo(email);

        var userinfo = get("/userinfo", (String) tokens.body().get("access_token"));
        assertThat(userinfo.status()).isEqualTo(200);
        assertThat(userinfo.body()).containsEntry("sub", accountId).containsEntry("email", email);
    }

    @Test
    void refreshTokensRotateAndCannotBeReplayed() throws Exception {
        var pkce = pkce();
        String first = (String) exchange(loginAndAuthorize(pkce), pkce.verifier()).body().get("refresh_token");

        var refreshed = refresh(first);
        assertThat(refreshed.status()).isEqualTo(200);
        assertThat(refreshed.body().get("refresh_token")).isNotNull().isNotEqualTo(first);

        var replay = refresh(first);
        assertThat(replay.status()).isEqualTo(400);
        assertThat(replay.body()).containsEntry("error", "invalid_grant");
    }

    @Test
    void codeIsUselessWithoutTheMatchingVerifier() throws Exception {
        String code = loginAndAuthorize(pkce());

        var stolen = exchange(code, pkce().verifier());

        assertThat(stolen.status()).isEqualTo(400);
        assertThat(stolen.body()).containsEntry("error", "invalid_grant");
    }

    @Test
    void wrongPasswordStaysOnTheLoginPage() throws Exception {
        startAuthorization(pkce());
        var res = submitLogin(email, "wrong password!!");

        assertThat(res.statusCode()).isEqualTo(302);
        assertThat(location(res).toString()).endsWith("/login?error");
    }

    @Test
    void userinfoRejectsMissingAndForgedTokens() throws Exception {
        var pkce = pkce();
        String token = (String) exchange(loginAndAuthorize(pkce), pkce.verifier()).body().get("access_token");

        assertThat(get("/userinfo", null).status()).isEqualTo(401);
        assertThat(get("/userinfo", forgeRole(token, "agent")).status()).isEqualTo(401);
    }

    private String loginAndAuthorize(Pkce pkce) throws Exception {
        startAuthorization(pkce);
        var afterLogin = submitLogin(email, PASSWORD);
        assertThat(afterLogin.statusCode()).isEqualTo(302);
        var callback = send(HttpRequest.newBuilder(location(afterLogin)).GET().build());
        assertThat(callback.statusCode()).isEqualTo(302);
        var redirect = location(callback).toString();
        assertThat(redirect).startsWith(TestClients.REDIRECT_URI).contains("state=xyz");
        var matcher = CODE.matcher(redirect);
        assertThat(matcher.find()).isTrue();
        return URLDecoder.decode(matcher.group(1), StandardCharsets.UTF_8);
    }

    private void startAuthorization(Pkce pkce) throws Exception {
        var authorize = URI.create(baseUrl() + "/oauth2/authorize?response_type=code"
                + "&client_id=" + TestClients.GATEWAY
                + "&scope=" + enc("openid email")
                + "&redirect_uri=" + enc(TestClients.REDIRECT_URI)
                + "&state=xyz"
                + "&code_challenge=" + pkce.challenge()
                + "&code_challenge_method=S256");
        var res = send(HttpRequest.newBuilder(authorize).header("Accept", "text/html").GET().build());
        assertThat(res.statusCode()).isEqualTo(302);
        assertThat(location(res).getPath()).isEqualTo("/login");
    }

    private HttpResponse<String> submitLogin(String username, String password) throws Exception {
        var page = send(HttpRequest.newBuilder(URI.create(baseUrl() + "/login")).GET().build());
        var csrf = CSRF.matcher(page.body());
        assertThat(csrf.find()).as("login page carries a CSRF token").isTrue();
        String form = "username=" + enc(username) + "&password=" + enc(password) + "&_csrf=" + enc(csrf.group(1));
        return send(HttpRequest.newBuilder(URI.create(baseUrl() + "/login"))
                .header("Content-Type", "application/x-www-form-urlencoded")
                .POST(HttpRequest.BodyPublishers.ofString(form))
                .build());
    }

    private Res exchange(String code, String verifier) {
        return token(TestClients.GATEWAY, TestClients.GATEWAY_SECRET, Map.of(
                "grant_type", "authorization_code",
                "code", code,
                "redirect_uri", TestClients.REDIRECT_URI,
                "code_verifier", verifier));
    }

    private Res refresh(String refreshToken) {
        return token(TestClients.GATEWAY, TestClients.GATEWAY_SECRET,
                Map.of("grant_type", "refresh_token", "refresh_token", refreshToken));
    }

    private HttpResponse<String> send(HttpRequest request) throws Exception {
        return browser.send(request, HttpResponse.BodyHandlers.ofString());
    }

    private URI location(HttpResponse<?> res) {
        return URI.create(baseUrl()).resolve(res.headers().firstValue("Location").orElseThrow());
    }

    private static Pkce pkce() throws Exception {
        byte[] random = new byte[32];
        new SecureRandom().nextBytes(random);
        var b64 = Base64.getUrlEncoder().withoutPadding();
        String verifier = b64.encodeToString(random);
        byte[] digest = MessageDigest.getInstance("SHA-256").digest(verifier.getBytes(StandardCharsets.US_ASCII));
        return new Pkce(verifier, b64.encodeToString(digest));
    }

    // Rewrites the payload to escalate privileges while keeping the original signature.
    private static String forgeRole(String token, String role) {
        String[] parts = token.split("\\.");
        String payload = new String(Base64.getUrlDecoder().decode(parts[1]), StandardCharsets.UTF_8)
                .replace("\"customer\"", "\"" + role + "\"");
        String forged = Base64.getUrlEncoder().withoutPadding().encodeToString(payload.getBytes(StandardCharsets.UTF_8));
        return parts[0] + "." + forged + "." + parts[2];
    }

    private static String enc(String value) {
        return URLEncoder.encode(value, StandardCharsets.UTF_8);
    }
}
