package com.crazyms.notifications.notification.service;

import com.crazyms.notifications.notification.config.SentLogProperties;
import com.crazyms.notifications.notification.domain.port.SentNotifications;
import java.time.Clock;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.scheduling.annotation.Scheduled;
import org.springframework.stereotype.Component;

/** sent_notification is append-only; this sweep is what keeps it bounded. */
@Component
class SentLogRetention {

  private static final Logger log = LoggerFactory.getLogger(SentLogRetention.class);

  private final SentNotifications sent;
  private final SentLogProperties props;
  private final Clock clock;

  SentLogRetention(SentNotifications sent, SentLogProperties props, Clock clock) {
    this.sent = sent;
    this.props = props;
    this.clock = clock;
  }

  // Every replica runs it; the DELETE is idempotent, so overlapping sweeps only waste a query.
  @Scheduled(fixedDelayString = "PT24H", initialDelayString = "PT10M")
  int purgeExpired() {
    int removed = sent.deleteSentBefore(clock.instant().minus(props.retention()));
    if (removed > 0) {
      log.atInfo().addKeyValue("removed", removed).log("sent_log.purged");
    }
    return removed;
  }
}
