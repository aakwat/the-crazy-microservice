package com.crazyms.auth;

import static org.assertj.core.api.Assertions.assertThat;

import java.net.http.HttpResponse;
import java.util.List;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.jdbc.core.simple.JdbcClient;

/** Test config locks after 3 failures for 2 seconds; production defaults are 5 and 15 minutes. */
class LoginLockoutTest extends AuthTestSupport {

    private static final String WRONG = "wrong password!!";

    @Autowired
    JdbcClient jdbc;

    String email;

    @BeforeEach
    void registerAccount() {
        email = uniqueEmail();
        register(email, PASSWORD);
    }

    @Test
    void locksAfterMaxFailuresEvenForTheRightPassword() throws Exception {
        var browser = new TestBrowser();
        for (int i = 0; i < 3; i++) {
            assertFailed(browser.submitLogin(baseUrl(), email, WRONG));
        }

        assertFailed(browser.submitLogin(baseUrl(), email, PASSWORD));
        assertThat(failureReasons(email))
                .containsExactly("bad_password", "bad_password", "bad_password", "account_locked");
    }

    @Test
    void lockedWrongPasswordAndUnknownEmailLookIdentical() throws Exception {
        var locked = new TestBrowser();
        for (int i = 0; i < 3; i++) {
            locked.submitLogin(baseUrl(), email, WRONG);
        }
        locked.submitLogin(baseUrl(), email, PASSWORD);
        var wrongPassword = new TestBrowser();
        String other = uniqueEmail();
        register(other, PASSWORD);
        wrongPassword.submitLogin(baseUrl(), other, WRONG);
        var unknown = new TestBrowser();
        unknown.submitLogin(baseUrl(), uniqueEmail(), PASSWORD);

        var pages = List.of(locked, wrongPassword, unknown).stream()
                .map(b -> withoutCsrf(page(b, "/login?error")))
                .toList();

        assertThat(pages.getFirst()).contains("Invalid credentials");
        assertThat(pages).as("locked, wrong password and unknown email render the same page").containsOnly(pages.getFirst());
    }

    private String page(TestBrowser browser, String path) {
        try {
            return browser.page(baseUrl() + path);
        } catch (Exception e) {
            throw new IllegalStateException(e);
        }
    }

    private static String withoutCsrf(String html) {
        return html.replaceAll("name=\"_csrf\"[^>]*value=\"[^\"]*\"", "name=\"_csrf\"");
    }

    @Test
    void lockExpiresAndASuccessClearsTheCounter() throws Exception {
        var browser = new TestBrowser();
        for (int i = 0; i < 3; i++) {
            browser.submitLogin(baseUrl(), email, WRONG);
        }
        Thread.sleep(2_200);

        assertSucceeded(browser.submitLogin(baseUrl(), email, PASSWORD));
        // The success reset the count, so two more failures stay under the limit of three.
        browser.submitLogin(baseUrl(), email, WRONG);
        browser.submitLogin(baseUrl(), email, WRONG);
        assertSucceeded(browser.submitLogin(baseUrl(), email, PASSWORD));
    }

    @Test
    void unknownIdentifiersAreRecordedWithoutAnAccount() throws Exception {
        String ghost = uniqueEmail();

        assertFailed(new TestBrowser().submitLogin(baseUrl(), ghost, PASSWORD));

        var row = jdbc.sql("""
                        SELECT account_id IS NULL AS no_account, failure_reason, ip_address IS NOT NULL AS has_ip
                        FROM authentication_attempts WHERE submitted_identifier = :id
                        """)
                .param("id", ghost)
                .query((rs, n) -> List.of(rs.getBoolean("no_account"), rs.getString("failure_reason"),
                        rs.getBoolean("has_ip")))
                .single();
        assertThat(row).containsExactly(true, "unknown_identifier", true);
    }

    private List<String> failureReasons(String identifier) {
        return jdbc.sql("SELECT failure_reason FROM authentication_attempts WHERE submitted_identifier = :id ORDER BY id")
                .param("id", identifier)
                .query(String.class)
                .list();
    }

    private void assertFailed(HttpResponse<String> res) {
        assertThat(res.statusCode()).isEqualTo(302);
        assertThat(TestBrowser.location(baseUrl(), res).toString()).endsWith("/login?error");
    }

    private void assertSucceeded(HttpResponse<String> res) {
        assertThat(res.statusCode()).isEqualTo(302);
        assertThat(TestBrowser.location(baseUrl(), res).toString()).doesNotContain("error");
    }
}
