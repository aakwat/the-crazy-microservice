package com.crazyms.notifications.notification.domain.port;

import com.crazyms.notifications.notification.domain.Notification;
import java.time.Instant;
import java.util.UUID;

/** Port: the sent log that makes a redelivered event send nothing twice. */
public interface SentNotifications {

  boolean contains(UUID eventId, UUID recipientId);

  void record(Notification notification, Instant sentAt);

  int deleteSentBefore(Instant cutoff);
}
