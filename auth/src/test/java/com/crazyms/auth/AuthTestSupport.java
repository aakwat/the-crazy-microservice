package com.crazyms.auth;

import static org.assertj.core.api.Assertions.assertThat;

import com.nimbusds.jose.JWSAlgorithm;
import com.nimbusds.jose.crypto.ECDSAVerifier;
import com.nimbusds.jose.jwk.ECKey;
import com.nimbusds.jose.jwk.JWKSet;
import com.nimbusds.jwt.JWTClaimsSet;
import com.nimbusds.jwt.SignedJWT;
import java.util.Map;
import java.util.UUID;
import org.junit.jupiter.api.BeforeEach;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.boot.test.context.SpringBootTest;
import org.springframework.context.annotation.Import;
import org.springframework.core.ParameterizedTypeReference;
import org.springframework.http.HttpHeaders;
import org.springframework.http.MediaType;
import org.springframework.util.LinkedMultiValueMap;
import org.springframework.web.client.RestClient;

/** One Spring context and one Postgres container shared by every test class that extends this. */
@Import(TestcontainersConfiguration.class)
@SpringBootTest(webEnvironment = SpringBootTest.WebEnvironment.RANDOM_PORT)
abstract class AuthTestSupport {

    static final String PASSWORD = "correct horse battery";
    static final ParameterizedTypeReference<Map<String, Object>> JSON = new ParameterizedTypeReference<>() {};

    @Value("${local.server.port}")
    int port;

    RestClient http;

    record Res(int status, Map<String, Object> body, HttpHeaders headers) {}

    @BeforeEach
    void setUpClient() {
        http = RestClient.builder().baseUrl(baseUrl()).build();
    }

    String baseUrl() {
        return "http://localhost:" + port;
    }

    static String uniqueEmail() {
        return "user-" + UUID.randomUUID() + "@example.com";
    }

    Res register(String email, String password) {
        return postJson("/auth/register", Map.of("email", email, "password", password));
    }

    Res postJson(String path, Object body) {
        return http.post().uri(path).contentType(MediaType.APPLICATION_JSON).body(body)
                .exchange((req, res) -> new Res(res.getStatusCode().value(), res.bodyTo(JSON), res.getHeaders()));
    }

    Res get(String path, String bearer) {
        return http.get().uri(path)
                .headers(h -> {
                    if (bearer != null) {
                        h.setBearerAuth(bearer);
                    }
                })
                .exchange((req, res) -> new Res(res.getStatusCode().value(), res.bodyTo(JSON), res.getHeaders()));
    }

    /** POST /oauth2/token as a confidential client (client_secret_basic). */
    Res token(String clientId, String secret, Map<String, String> form) {
        var body = new LinkedMultiValueMap<String, String>();
        form.forEach(body::add);
        return http.post().uri("/oauth2/token")
                .headers(h -> h.setBasicAuth(clientId, secret))
                .contentType(MediaType.APPLICATION_FORM_URLENCODED)
                .body(body)
                .exchange((req, res) -> new Res(res.getStatusCode().value(), res.bodyTo(JSON), res.getHeaders()));
    }

    /** Verifies a token exactly as another service will: ES256 against the published JWKS only. */
    JWTClaimsSet verifyAgainstJwks(String token) throws Exception {
        var jwt = SignedJWT.parse(token);
        assertThat(jwt.getHeader().getAlgorithm()).isEqualTo(JWSAlgorithm.ES256);
        var jwks = JWKSet.parse(get("/oauth2/jwks", null).body());
        var key = (ECKey) jwks.getKeyByKeyId(jwt.getHeader().getKeyID());
        assertThat(key).as("signing key is published in JWKS").isNotNull();
        assertThat(key.isPrivate()).as("JWKS must not leak the private key").isFalse();
        assertThat(jwt.verify(new ECDSAVerifier(key))).isTrue();
        assertThat(jwt.getJWTClaimsSet().getIssuer()).isEqualTo(TestClients.ISSUER);
        return jwt.getJWTClaimsSet();
    }
}
