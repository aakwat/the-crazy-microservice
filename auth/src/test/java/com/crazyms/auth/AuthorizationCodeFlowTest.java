package com.crazyms.auth;

import static org.assertj.core.api.Assertions.assertThat;

import com.crazyms.auth.TestBrowser.Pkce;
import java.nio.charset.StandardCharsets;
import java.util.Base64;
import java.util.Map;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

/** Drives the real browser flow over HTTP: authorize, login form with CSRF, code, token with the PKCE verifier. */
class AuthorizationCodeFlowTest extends AuthTestSupport {

    TestBrowser browser;
    String email;
    String accountId;

    @BeforeEach
    void setUpBrowserAndAccount() {
        browser = new TestBrowser();
        email = uniqueEmail();
        accountId = (String) register(email, PASSWORD).body().get("id");
    }

    @Test
    void browserLoginIssuesVerifiableTokensForTheAccount() throws Exception {
        var pkce = TestBrowser.pkce();
        var tokens = exchange(http, loginAndAuthorize(pkce), pkce.verifier());

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
        var pkce = TestBrowser.pkce();
        String first = (String) exchange(http, loginAndAuthorize(pkce), pkce.verifier()).body().get("refresh_token");

        var refreshed = refresh(http, first);
        assertThat(refreshed.status()).isEqualTo(200);
        assertThat(refreshed.body().get("refresh_token")).isNotNull().isNotEqualTo(first);

        var replay = refresh(http, first);
        assertThat(replay.status()).isEqualTo(400);
        assertThat(replay.body()).containsEntry("error", "invalid_grant");
    }

    @Test
    void codeIsUselessWithoutTheMatchingVerifier() throws Exception {
        String code = loginAndAuthorize(TestBrowser.pkce());

        var stolen = exchange(http, code, TestBrowser.pkce().verifier());

        assertThat(stolen.status()).isEqualTo(400);
        assertThat(stolen.body()).containsEntry("error", "invalid_grant");
    }

    @Test
    void codeCanBeRedeemedOnlyOnce() throws Exception {
        var pkce = TestBrowser.pkce();
        String code = loginAndAuthorize(pkce);
        assertThat(exchange(http, code, pkce.verifier()).status()).isEqualTo(200);

        var replay = exchange(http, code, pkce.verifier());

        assertThat(replay.status()).isEqualTo(400);
        assertThat(replay.body()).containsEntry("error", "invalid_grant").doesNotContainKey("access_token");
    }

    @Test
    void codeIssuedToTheGatewayCannotBeRedeemedByAnotherClient() throws Exception {
        var pkce = TestBrowser.pkce();
        String code = loginAndAuthorize(pkce);

        var res = token(TestClients.JOBS, TestClients.JOBS_SECRET, Map.of(
                "grant_type", "authorization_code",
                "code", code,
                "redirect_uri", TestClients.REDIRECT_URI,
                "code_verifier", pkce.verifier()));

        assertThat(res.status()).isEqualTo(400);
        assertThat(res.body()).doesNotContainKey("access_token");
        assertThat((String) res.body().get("error")).isIn("invalid_grant", "unauthorized_client");
    }

    @Test
    void wrongPasswordStaysOnTheLoginPage() throws Exception {
        browser.startAuthorization(baseUrl(), TestBrowser.pkce());
        var res = browser.submitLogin(baseUrl(), email, "wrong password!!");

        assertThat(res.statusCode()).isEqualTo(302);
        assertThat(TestBrowser.location(baseUrl(), res).toString()).endsWith("/login?error");
    }

    @Test
    void userinfoRejectsMissingAndForgedTokens() throws Exception {
        var pkce = TestBrowser.pkce();
        String token = (String) exchange(http, loginAndAuthorize(pkce), pkce.verifier()).body().get("access_token");

        String forged = forgeRole(token, "agent");

        assertThat(forged).as("the forgery must actually change the payload").isNotEqualTo(token);
        assertThat(get("/userinfo", null).status()).isEqualTo(401);
        assertThat(get("/userinfo", forged).status()).isEqualTo(401);
    }

    private String loginAndAuthorize(Pkce pkce) throws Exception {
        browser.startAuthorization(baseUrl(), pkce);
        var afterLogin = browser.submitLogin(baseUrl(), email, PASSWORD);
        assertThat(afterLogin.statusCode()).isEqualTo(302);
        return browser.followToCode(TestBrowser.location(baseUrl(), afterLogin));
    }

    // Rewrites the payload to escalate privileges while keeping the original signature.
    private static String forgeRole(String token, String role) {
        String[] parts = token.split("\\.");
        String payload = new String(Base64.getUrlDecoder().decode(parts[1]), StandardCharsets.UTF_8)
                .replace("\"customer\"", "\"" + role + "\"");
        String forged = Base64.getUrlEncoder().withoutPadding().encodeToString(payload.getBytes(StandardCharsets.UTF_8));
        return parts[0] + "." + forged + "." + parts[2];
    }
}
