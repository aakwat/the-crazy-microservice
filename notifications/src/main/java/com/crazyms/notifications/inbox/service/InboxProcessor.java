package com.crazyms.notifications.inbox.service;

import com.crazyms.notifications.inbox.domain.InboxMessage;
import com.crazyms.notifications.inbox.domain.port.EventInbox;
import com.crazyms.notifications.notification.service.NotificationService;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.stereotype.Service;

/** One pass over the inbox; a failing message is retried later and never blocks the others. */
@Service
public class InboxProcessor {

  private static final Logger log = LoggerFactory.getLogger(InboxProcessor.class);

  /** After this many deliveries a message goes to the dead-letter stream (CLAUDE.md, events). */
  static final long MAX_DELIVERIES = 5;

  private final EventInbox inbox;
  private final NotificationService notifications;

  InboxProcessor(EventInbox inbox, NotificationService notifications) {
    this.inbox = inbox;
    this.notifications = notifications;
  }

  /** Throws only when the inbox itself cannot be reached. */
  public void processOnce() {
    inbox.reclaimStuck().forEach(this::process);
    inbox.receive().forEach(this::process);
  }

  private void process(InboxMessage message) {
    if (message.deliveries() > MAX_DELIVERIES) {
      inbox.deadLetter(message);
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
      return;
    }
    inbox.acknowledge(message);
  }
}
