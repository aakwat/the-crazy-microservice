package com.crazyms.notifications.notification.repository;

import com.crazyms.notifications.notification.domain.Notification;
import com.crazyms.notifications.notification.domain.port.SentNotifications;
import java.time.Instant;
import java.time.OffsetDateTime;
import java.time.ZoneOffset;
import java.util.Locale;
import java.util.UUID;
import org.springframework.jdbc.core.simple.JdbcClient;
import org.springframework.stereotype.Repository;

/** Append-only inserts and an age-based purge; no entity, since rows are never updated. */
@Repository
class JdbcSentNotifications implements SentNotifications {

  private final JdbcClient jdbc;

  JdbcSentNotifications(JdbcClient jdbc) {
    this.jdbc = jdbc;
  }

  @Override
  public boolean contains(UUID eventId, UUID recipientId) {
    return jdbc.sql(
            """
            SELECT EXISTS (
              SELECT 1 FROM sent_notification WHERE event_id = :eventId AND recipient_id = :recipientId
            )
            """)
        .param("eventId", eventId)
        .param("recipientId", recipientId)
        .query(Boolean.class)
        .single();
  }

  // ON CONFLICT: two replicas racing on one redelivered event both sent; the log keeps one row.
  @Override
  public void record(Notification notification, Instant sentAt) {
    jdbc.sql(
            """
            INSERT INTO sent_notification (event_id, recipient_id, reason, sent_at)
            VALUES (:eventId, :recipientId, :reason, :sentAt)
            ON CONFLICT DO NOTHING
            """)
        .param("eventId", notification.eventId())
        .param("recipientId", notification.recipientId())
        .param("reason", notification.reason().name().toLowerCase(Locale.ROOT))
        .param("sentAt", utc(sentAt))
        .update();
  }

  @Override
  public int deleteSentBefore(Instant cutoff) {
    return jdbc.sql("DELETE FROM sent_notification WHERE sent_at < :cutoff")
        .param("cutoff", utc(cutoff))
        .update();
  }

  // The Postgres driver binds OffsetDateTime to timestamptz natively; Instant it does not.
  private static OffsetDateTime utc(Instant instant) {
    return OffsetDateTime.ofInstant(instant, ZoneOffset.UTC);
  }
}
