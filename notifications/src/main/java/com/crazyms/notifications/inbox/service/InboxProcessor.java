package com.crazyms.notifications.inbox.service;

import com.crazyms.notifications.inbox.domain.InboxMessage;
import com.crazyms.notifications.inbox.domain.port.EventInbox;
import com.crazyms.notifications.notification.service.NotificationService;
import io.micrometer.tracing.Span;
import io.micrometer.tracing.Tracer;
import io.micrometer.tracing.propagation.Propagator;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.stereotype.Service;

/** One pass over the inbox; a failing message is retried later and never blocks the others. */
@Service
public class InboxProcessor {

  private static final Logger log = LoggerFactory.getLogger(InboxProcessor.class);

  /** After this many deliveries a message goes to the dead-letter stream (CLAUDE.md, events). */
  static final long MAX_DELIVERIES = 5;

  private static final String STREAM = "tickets.events";
  private static final String TRACEPARENT = "traceparent";

  private final EventInbox inbox;
  private final NotificationService notifications;
  private final Tracer tracer;
  private final Propagator propagator;

  InboxProcessor(
      EventInbox inbox, NotificationService notifications, Tracer tracer, Propagator propagator) {
    this.inbox = inbox;
    this.notifications = notifications;
    this.tracer = tracer;
    this.propagator = propagator;
  }

  /** Throws only when the inbox itself cannot be reached. */
  public void processOnce() {
    inbox.reclaimStuck().forEach(this::process);
    inbox.receive().forEach(this::process);
  }

  /** Each message continues the trace of the ticket write that produced it. */
  private void process(InboxMessage message) {
    var span = consumerSpan(message);
    try (var scope = tracer.withSpan(span)) {
      deliver(message, span);
    } finally {
      span.end();
    }
  }

  private void deliver(InboxMessage message, Span span) {
    if (message.deliveries() > MAX_DELIVERIES) {
      inbox.deadLetter(message);
      span.tag("messaging.outcome", "dead_lettered");
      return;
    }
    try {
      notifications.notify(message.event());
    } catch (RuntimeException e) {
      // Left unacknowledged: reclaimStuck hands it out again once it has sat idle long enough.
      log.atWarn()
          .addKeyValue("message_id", message.id())
          .addKeyValue("deliveries", message.deliveries())
          // Class name only: mail errors can quote the recipient's address.
          .addKeyValue("error", e.getClass().getSimpleName())
          .log("inbox.delivery_failed");
      // Not span.error(e): that records the message, which can quote an address too.
      span.tag("error.type", e.getClass().getSimpleName());
      return;
    }
    inbox.acknowledge(message);
  }

  private Span consumerSpan(InboxMessage message) {
    return propagator
        .extract(message, (m, key) -> TRACEPARENT.equals(key) ? m.traceparent() : null)
        .name("process " + STREAM)
        .kind(Span.Kind.CONSUMER)
        .tag("messaging.system", "redis")
        .tag("messaging.destination.name", STREAM)
        .tag("messaging.message.id", message.id())
        .tag("messaging.message.delivery_count", message.deliveries())
        .start();
  }
}
