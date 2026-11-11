package com.crazyms.auth.authserver.service;

import com.crazyms.auth.authserver.domain.port.ExpiredAuthorizationRepository;
import java.time.Clock;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.scheduling.annotation.Scheduled;
import org.springframework.stereotype.Component;

/** Every login and refresh leaves an oauth2_authorization row; this keeps it bounded. */
@Component
public class AuthorizationRetention {

  private static final Logger log = LoggerFactory.getLogger(AuthorizationRetention.class);

  private final ExpiredAuthorizationRepository authorizations;
  private final Clock clock;

  AuthorizationRetention(ExpiredAuthorizationRepository authorizations, Clock clock) {
    this.authorizations = authorizations;
    this.clock = clock;
  }

  // Every replica runs it; the DELETE is idempotent, so overlapping sweeps only waste a query.
  @Scheduled(fixedDelayString = "PT1H", initialDelayString = "PT5M")
  public int purgeExpired() {
    int removed = authorizations.deleteExpiredBefore(clock.instant());

    if (removed > 0) {
      log.info("purged {} expired oauth2 authorizations", removed);
    }

    return removed;
  }
}
