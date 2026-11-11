package com.crazyms.auth.account.service;

import com.crazyms.auth.account.config.AccountProperties;
import com.crazyms.auth.account.domain.port.RoleChangeRepository;
import java.time.Clock;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.scheduling.annotation.Scheduled;
import org.springframework.stereotype.Component;

/** account_role_change is append-only; this sweep is what keeps it bounded. */
@Component
public class RoleChangeRetention {

  private static final Logger log = LoggerFactory.getLogger(RoleChangeRetention.class);

  private final RoleChangeRepository roleChanges;
  private final AccountProperties props;
  private final Clock clock;

  RoleChangeRetention(RoleChangeRepository roleChanges, AccountProperties props, Clock clock) {
    this.roleChanges = roleChanges;
    this.props = props;
    this.clock = clock;
  }

  // Every replica runs it; the DELETE is idempotent, so overlapping sweeps only waste a query.
  @Scheduled(fixedDelayString = "PT24H", initialDelayString = "PT10M")
  public int purgeExpired() {
    int removed = roleChanges.deleteOlderThan(clock.instant().minus(props.roleChangeRetention()));
    if (removed > 0) {
      log.info("purged {} role changes older than {}", removed, props.roleChangeRetention());
    }
    return removed;
  }
}
