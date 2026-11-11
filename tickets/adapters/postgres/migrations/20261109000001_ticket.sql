-- ticket: one support request. Only this service writes it; other services change it by event.
--
-- Limits and statuses mirror domain::Ticket, so a bug there still cannot store a bad row.
CREATE TABLE ticket (
  id            UUID         PRIMARY KEY DEFAULT uuidv7(),
  requester_id  UUID         NOT NULL,
  assignee_id   UUID,
  title         TEXT         NOT NULL CHECK (char_length(title) BETWEEN 1 AND 200),
  body          TEXT         NOT NULL CHECK (char_length(body) BETWEEN 1 AND 10000),
  status        TEXT         NOT NULL DEFAULT 'open'
                             CHECK (status IN ('open', 'in_progress', 'resolved', 'closed')),
  created_at    TIMESTAMPTZ  NOT NULL DEFAULT now(),
  updated_at    TIMESTAMPTZ  NOT NULL DEFAULT now(),
  CHECK (updated_at >= created_at)
);

-- A customer's own list, newest first: uuidv7 ids sort by creation time.
CREATE INDEX idx_ticket_requester ON ticket (requester_id, id DESC);
