package com.crazyms.auth.login.service;

import com.crazyms.auth.login.config.LoginProperties;
import com.crazyms.auth.login.domain.port.AuthenticationAttemptRepository;
import java.time.Clock;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.scheduling.annotation.Scheduled;
import org.springframework.stereotype.Component;

/** authentication_attempts grows by one row per login; this sweep is what keeps it bounded. */
@Component
public class AttemptRetention {

  private static final Logger log = LoggerFactory.getLogger(AttemptRetention.class);

  private final AuthenticationAttemptRepository attempts;
  private final LoginProperties props;
  private final Clock clock;

  AttemptRetention(AuthenticationAttemptRepository attempts, LoginProperties props, Clock clock) {
    this.attempts = attempts;
    this.props = props;
    this.clock = clock;
  }

  // Every replica runs it; the DELETE is idempotent, so overlapping sweeps only waste a query.
  @Scheduled(fixedDelayString = "PT1H", initialDelayString = "PT5M")
  public int purgeExpired() {
    int removed = attempts.deleteOlderThan(clock.instant().minus(props.attemptRetention()));
    if (removed > 0) {
      log.info(
          "purged {} authentication attempts older than {}", removed, props.attemptRetention());
    }
    return removed;
  }
}
