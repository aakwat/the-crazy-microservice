//! `Message` — one entry in a ticket's conversation, never edited once written.

use time::OffsetDateTime;
use uuid::Uuid;

use super::stored::{fits, stored_precision};
use super::{Author, TicketError, Visibility};

/// Longest message in characters; the database enforces the same limit.
pub const MAX_MESSAGE_CHARS: usize = 10_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
  id: Uuid,
  ticket_id: Uuid,
  author_id: Uuid,
  author: Author,
  visibility: Visibility,
  body: String,
  created_at: OffsetDateTime,
}

impl Message {
  /// Only `Ticket` writes messages, so every one comes with the transition it causes.
  pub(super) fn write(
    ticket_id: Uuid,
    author_id: Uuid,
    author: Author,
    visibility: Visibility,
    body: &str,
    now: OffsetDateTime,
  ) -> Result<Self, TicketError> {
    let body = body.trim();
    if !fits(body, MAX_MESSAGE_CHARS) {
      return Err(TicketError::Message);
    }
    Ok(Self {
      id: Uuid::now_v7(),
      ticket_id,
      author_id,
      author,
      visibility,
      body: body.to_owned(),
      created_at: stored_precision(now),
    })
  }

  /// Rebuilds a stored message; only the persistence adapter calls this.
  pub fn from_persistence(
    id: Uuid,
    ticket_id: Uuid,
    author_id: Uuid,
    author: Author,
    visibility: Visibility,
    body: String,
    created_at: OffsetDateTime,
  ) -> Self {
    Self {
      id,
      ticket_id,
      author_id,
      author,
      visibility,
      body,
      created_at,
    }
  }

  pub fn id(&self) -> Uuid {
    self.id
  }

  pub fn ticket_id(&self) -> Uuid {
    self.ticket_id
  }

  pub fn author_id(&self) -> Uuid {
    self.author_id
  }

  pub fn author(&self) -> Author {
    self.author
  }

  pub fn visibility(&self) -> Visibility {
    self.visibility
  }

  pub fn body(&self) -> &str {
    &self.body
  }

  pub fn created_at(&self) -> OffsetDateTime {
    self.created_at
  }
}
