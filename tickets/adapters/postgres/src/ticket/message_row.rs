//! `MessageRow` — a `ticket_message` row as sqlx reads it.

use domain::{Message, StoreError};
use time::OffsetDateTime;
use uuid::Uuid;

use super::column::{parse_author, parse_visibility};

#[derive(sqlx::FromRow)]
pub(crate) struct MessageRow {
  id: Uuid,
  ticket_id: Uuid,
  author_id: Uuid,
  author: String,
  visibility: String,
  body: String,
  created_at: OffsetDateTime,
}

impl MessageRow {
  pub(crate) fn into_message(self) -> Result<Message, StoreError> {
    Ok(Message::from_persistence(
      self.id,
      self.ticket_id,
      self.author_id,
      parse_author(&self.author)?,
      parse_visibility(&self.visibility)?,
      self.body,
      self.created_at,
    ))
  }
}
