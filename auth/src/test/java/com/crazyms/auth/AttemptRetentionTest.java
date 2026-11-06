package com.crazyms.auth;

import static org.assertj.core.api.Assertions.assertThat;

import com.crazyms.auth.login.domain.AuthenticationAttempt;
import com.crazyms.auth.login.domain.AuthenticationAttemptRepository;
import com.crazyms.auth.login.domain.ClientInfo;
import com.crazyms.auth.login.domain.FailureReason;
import com.crazyms.auth.login.service.AttemptRetention;
import java.time.Clock;
import java.time.Duration;
import org.junit.jupiter.api.Test;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.jdbc.core.simple.JdbcClient;

class AttemptRetentionTest extends AuthTestSupport {

    @Autowired
    AttemptRetention retention;

    @Autowired
    AuthenticationAttemptRepository attempts;

    @Autowired
    JdbcClient jdbc;

    @Autowired
    Clock clock;

    @Test
    void purgesOnlyAttemptsOlderThanTheRetention() {
        String expired = uniqueEmail();
        String recent = uniqueEmail();
        var now = clock.instant();
        attempts.insert(AuthenticationAttempt.failed(expired, null, FailureReason.UNKNOWN_IDENTIFIER, ClientInfo.UNKNOWN,
                now.minus(Duration.ofDays(91))));
        attempts.insert(AuthenticationAttempt.failed(recent, null, FailureReason.UNKNOWN_IDENTIFIER, ClientInfo.UNKNOWN,
                now.minus(Duration.ofDays(89))));

        assertThat(retention.purgeExpired()).isPositive();

        assertThat(count(expired)).isZero();
        assertThat(count(recent)).isOne();
    }

    private long count(String identifier) {
        return jdbc.sql("SELECT COUNT(*) FROM authentication_attempts WHERE submitted_identifier = :id")
                .param("id", identifier)
                .query(Long.class)
                .single();
    }
}
