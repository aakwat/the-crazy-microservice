package com.crazyms.notifications.inbox.worker;

import java.time.Clock;
import java.time.Duration;
import java.time.Instant;
import org.springframework.boot.health.contributor.Health;
import org.springframework.boot.health.contributor.HealthIndicator;
import org.springframework.stereotype.Component;

/** DOWN only when the worker stalls: a restart cannot fix Redis or Postgres being down. */
@Component("inbox")
class InboxLiveness implements HealthIndicator {

  /** Longer than the longest backoff plus one slow round. */
  private static final Duration MAX_SILENCE = Duration.ofSeconds(60);

  private final Clock clock;
  private volatile Instant lastRound;

  InboxLiveness(Clock clock) {
    this.clock = clock;
    this.lastRound = clock.instant();
  }

  void roundFinished() {
    lastRound = clock.instant();
  }

  @Override
  public Health health() {
    var silence = Duration.between(lastRound, clock.instant());
    return silence.compareTo(MAX_SILENCE) > 0 ? Health.down().build() : Health.up().build();
  }
}
