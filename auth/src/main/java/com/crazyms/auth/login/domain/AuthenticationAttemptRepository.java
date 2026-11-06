package com.crazyms.auth.login.domain;

import java.time.Instant;

/** Port: the append-only record of login outcomes. */
public interface AuthenticationAttemptRepository {

    void insert(AuthenticationAttempt attempt);

    /** Returns how many rows the retention sweep removed. */
    int deleteOlderThan(Instant cutoff);
}
