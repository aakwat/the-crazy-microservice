-- ticket: one support request. Only this service writes it; other services change it by event.
--
-- Limits and statuses mirror domain::Ticket, so a bug there still cannot store a bad row.

-- Short numbers customers quote (T-1042); the service takes one before it stores the ticket.
CREATE SEQUENCE ticket_number_seq;

CREATE TABLE ticket (
  id            UUID         PRIMARY KEY DEFAULT uuidv7(),
  number        BIGINT       NOT NULL UNIQUE CHECK (number > 0),
  requester_id  UUID         NOT NULL,
  assignee_id   UUID,
  title         TEXT         NOT NULL CHECK (char_length(title) BETWEEN 1 AND 200),
  body          TEXT         NOT NULL CHECK (char_length(body) BETWEEN 1 AND 10000),
  status        TEXT         NOT NULL DEFAULT 'new'
                             CHECK (status IN ('new', 'open', 'pending', 'resolved', 'closed')),
  close_reason  TEXT         CHECK (close_reason IN ('confirmed', 'withdrawn')),
  reopen_count  INTEGER      NOT NULL DEFAULT 0 CHECK (reopen_count >= 0),
  created_at    TIMESTAMPTZ  NOT NULL DEFAULT now(),
  updated_at    TIMESTAMPTZ  NOT NULL DEFAULT now(),
  CHECK (updated_at >= created_at),
  CHECK ((status = 'closed') = (close_reason IS NOT NULL))
);

-- A customer's own list, newest first: uuidv7 ids sort by creation time.
CREATE INDEX idx_ticket_requester ON ticket (requester_id, id DESC);

-- ticket_message: the conversation; append-only, never edited.
CREATE TABLE ticket_message (
  id          UUID         PRIMARY KEY DEFAULT uuidv7(),
  ticket_id   UUID         NOT NULL REFERENCES ticket (id),
  author_id   UUID         NOT NULL,
  author      TEXT         NOT NULL CHECK (author IN ('customer', 'staff')),
  visibility  TEXT         NOT NULL CHECK (visibility IN ('public', 'internal')),
  body        TEXT         NOT NULL CHECK (char_length(body) BETWEEN 1 AND 10000),
  created_at  TIMESTAMPTZ  NOT NULL DEFAULT now(),
  -- Customers cannot write internal notes.
  CHECK (author = 'staff' OR visibility = 'public')
);

-- One ticket's thread, oldest first.
CREATE INDEX idx_ticket_message_ticket ON ticket_message (ticket_id, id);
