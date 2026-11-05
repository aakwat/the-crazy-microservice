package com.crazyms.auth;

import static org.assertj.core.api.Assertions.assertThat;

import com.nimbusds.jose.crypto.ECDSAVerifier;
import com.nimbusds.jose.jwk.ECKey;
import com.nimbusds.jwt.SignedJWT;
import java.nio.charset.StandardCharsets;
import java.util.Base64;
import java.util.List;
import java.util.Map;
import java.util.UUID;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.boot.test.context.SpringBootTest;
import org.springframework.context.annotation.Import;
import org.springframework.core.ParameterizedTypeReference;
import org.springframework.http.HttpHeaders;
import org.springframework.http.MediaType;
import org.springframework.web.client.RestClient;

@Import(TestcontainersConfiguration.class)
@SpringBootTest(webEnvironment = SpringBootTest.WebEnvironment.RANDOM_PORT)
class AuthFlowTest {

    private static final String PASSWORD = "correct horse battery";
    private static final ParameterizedTypeReference<Map<String, Object>> JSON = new ParameterizedTypeReference<>() {};

    @Value("${local.server.port}")
    int port;

    RestClient http;

    record Res(int status, Map<String, Object> body, HttpHeaders headers) {}

    @BeforeEach
    void setUp() {
        http = RestClient.builder().baseUrl("http://localhost:" + port).build();
    }

    @Test
    void registerLoginAndVerifyTokenAgainstJwks() throws Exception {
        String email = uniqueEmail();

        var registered = post("/auth/register", Map.of("email", email.toUpperCase(), "password", PASSWORD));
        assertThat(registered.status()).isEqualTo(201);
        assertThat(registered.body()).containsOnlyKeys("id", "email", "role");
        assertThat(registered.body()).containsEntry("email", email).containsEntry("role", "customer");
        assertThat(UUID.fromString((String) registered.body().get("id")).version()).isEqualTo(7);

        var login = post("/auth/login", Map.of("email", email, "password", PASSWORD));
        assertThat(login.status()).isEqualTo(200);
        assertThat(login.body()).containsOnlyKeys("access_token", "token_type", "expires_in");
        assertThat(login.body()).containsEntry("token_type", "Bearer").containsEntry("expires_in", 900);
        assertThat(login.headers().getCacheControl()).contains("no-store");
        assertThat(login.headers().get(HttpHeaders.SET_COOKIE)).isNull();
        String token = (String) login.body().get("access_token");

        var me = get("/auth/me", token);
        assertThat(me.status()).isEqualTo(200);
        assertThat(me.body()).containsEntry("email", email).containsEntry("id", registered.body().get("id"));

        // Verify exactly as another service will: public key from JWKS only.
        var jwks = get("/.well-known/jwks.json", null);
        @SuppressWarnings("unchecked")
        var keys = (List<Map<String, Object>>) jwks.body().get("keys");
        assertThat(keys).hasSize(1);
        assertThat(keys.getFirst()).containsEntry("kty", "EC").containsEntry("crv", "P-256")
                .containsEntry("alg", "ES256").doesNotContainKey("d");
        var publicKey = ECKey.parse(keys.getFirst());
        var jwt = SignedJWT.parse(token);
        assertThat(jwt.getHeader().getKeyID()).isEqualTo(publicKey.getKeyID());
        assertThat(jwt.verify(new ECDSAVerifier(publicKey))).isTrue();
        assertThat(jwt.getJWTClaimsSet().getIssuer()).isEqualTo("crazy-ms-auth");
        assertThat(jwt.getJWTClaimsSet().getAudience()).containsExactly("crazy-ms");
        assertThat(jwt.getJWTClaimsSet().getSubject()).isEqualTo(registered.body().get("id"));
        assertThat(jwt.getJWTClaimsSet().getStringListClaim("roles")).containsExactly("customer");
    }

    @Test
    void duplicateEmailInAnyCaseIsConflict() {
        String email = uniqueEmail();
        assertThat(post("/auth/register", Map.of("email", email, "password", PASSWORD)).status()).isEqualTo(201);

        var again = post("/auth/register", Map.of("email", email.toUpperCase(), "password", PASSWORD));
        assertThat(again.status()).isEqualTo(409);
        assertThat(again.headers().getContentType()).isEqualTo(MediaType.APPLICATION_PROBLEM_JSON);
    }

    @Test
    void invalidRegistrationIsBadRequest() {
        assertThat(post("/auth/register", Map.of("email", uniqueEmail(), "password", "short")).status()).isEqualTo(400);
        assertThat(post("/auth/register", Map.of("email", "not-an-email", "password", PASSWORD)).status()).isEqualTo(400);
        assertThat(post("/auth/register", Map.of("email", uniqueEmail())).status()).isEqualTo(400);
    }

    @Test
    void wrongPasswordAndUnknownEmailLookIdentical() {
        String email = uniqueEmail();
        post("/auth/register", Map.of("email", email, "password", PASSWORD));

        var wrongPassword = post("/auth/login", Map.of("email", email, "password", "wrong password!!"));
        var unknownEmail = post("/auth/login", Map.of("email", uniqueEmail(), "password", PASSWORD));

        assertThat(wrongPassword.status()).isEqualTo(401);
        assertThat(unknownEmail.status()).isEqualTo(401);
        assertThat(wrongPassword.body().get("detail")).isEqualTo(unknownEmail.body().get("detail"));
    }

    @Test
    void meRejectsMissingAndForgedTokens() {
        String email = uniqueEmail();
        post("/auth/register", Map.of("email", email, "password", PASSWORD));
        String token = (String) post("/auth/login", Map.of("email", email, "password", PASSWORD)).body().get("access_token");

        assertThat(get("/auth/me", null).status()).isEqualTo(401);
        assertThat(get("/auth/me", forgeRole(token, "agent")).status()).isEqualTo(401);
    }

    // Rewrites the payload to escalate privileges while keeping the original signature.
    private static String forgeRole(String token, String role) {
        String[] parts = token.split("\\.");
        var decoder = Base64.getUrlDecoder();
        var encoder = Base64.getUrlEncoder().withoutPadding();
        String payload = new String(decoder.decode(parts[1]), StandardCharsets.UTF_8).replace("customer", role);
        return parts[0] + "." + encoder.encodeToString(payload.getBytes(StandardCharsets.UTF_8)) + "." + parts[2];
    }

    private static String uniqueEmail() {
        return "user-" + UUID.randomUUID() + "@example.com";
    }

    private Res post(String path, Object body) {
        return http.post().uri(path).contentType(MediaType.APPLICATION_JSON).body(body)
                .exchange((req, res) -> new Res(res.getStatusCode().value(), res.bodyTo(JSON), res.getHeaders()));
    }

    private Res get(String path, String bearer) {
        return http.get().uri(path)
                .headers(h -> {
                    if (bearer != null) {
                        h.setBearerAuth(bearer);
                    }
                })
                .exchange((req, res) -> new Res(res.getStatusCode().value(), res.bodyTo(JSON), res.getHeaders()));
    }
}
