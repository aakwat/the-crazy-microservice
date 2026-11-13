package com.crazyms.notifications.notification.service;

import com.crazyms.notifications.notification.domain.Notification;
import com.crazyms.notifications.notification.domain.TicketEvent;
import com.crazyms.notifications.notification.domain.port.AccountDirectory;
import com.crazyms.notifications.notification.domain.port.Mailer;
import com.crazyms.notifications.notification.domain.port.SentNotifications;
import java.time.Clock;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.stereotype.Service;

/** Emails everyone an event concerns, once each; never stores or logs an address. */
@Service
public class NotificationService {

  private static final Logger log = LoggerFactory.getLogger(NotificationService.class);

  private final SentNotifications sent;
  private final AccountDirectory directory;
  private final Mailer mailer;
  private final Clock clock;

  NotificationService(
      SentNotifications sent, AccountDirectory directory, Mailer mailer, Clock clock) {
    this.sent = sent;
    this.directory = directory;
    this.mailer = mailer;
    this.clock = clock;
  }

  /** Throws when a dependency is down, so the caller leaves the event to be delivered again. */
  public void notify(TicketEvent event) {
    for (var notification : event.notifications()) {
      if (!sent.contains(notification.eventId(), notification.recipientId())) {
        deliver(notification);
      }
    }
  }

  // Send before record: a crash in between costs one duplicate email, never a lost one.
  private void deliver(Notification notification) {
    var email = directory.emailOf(notification.recipientId());
    if (email.isEmpty()) {
      log.atWarn()
          .addKeyValue("event_id", notification.eventId())
          .addKeyValue("recipient_id", notification.recipientId())
          .log("notification.recipient_unknown");
      return;
    }

    mailer.send(notification, email.get());
    sent.record(notification, clock.instant());

    log.atInfo()
        .addKeyValue("event_id", notification.eventId())
        .addKeyValue("recipient_id", notification.recipientId())
        .addKeyValue("reason", notification.reason())
        .log("notification.sent");
  }
}
