-- outbox: events written in the same transaction as the ticket change, published later by the relay.
--
-- The payload is thin (ids and status, never ticket text) to keep personal data out of the event stream.
CREATE TABLE outbox (
  id            UUID         PRIMARY KEY,
  stream        TEXT         NOT NULL,
  type          TEXT         NOT NULL,
  version       SMALLINT     NOT NULL CHECK (version > 0),
  subject_id    UUID         NOT NULL,
  traceparent   TEXT,
  payload       JSONB        NOT NULL,
  occurred_at   TIMESTAMPTZ  NOT NULL,
  published_at  TIMESTAMPTZ
);

-- The relay only scans unpublished rows, oldest first.
CREATE INDEX idx_outbox_unpublished ON outbox (occurred_at) WHERE published_at IS NULL;
