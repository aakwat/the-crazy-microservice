-- sent_notification: one row per email sent; the key is what makes a redelivered event send nothing twice.
--
-- Account ids only, never addresses: auth owns those.
CREATE TABLE sent_notification (
  event_id      UUID         NOT NULL,
  recipient_id  UUID         NOT NULL,
  reason        TEXT         NOT NULL CHECK (reason IN ('ticket_opened', 'ticket_changed', 'assigned_ticket_changed')),
  sent_at       TIMESTAMPTZ  NOT NULL,
  PRIMARY KEY (event_id, recipient_id)
);

-- The retention sweep deletes rows by age.
CREATE INDEX idx_sent_notification_sent_at ON sent_notification (sent_at);
