//! `TicketRow` — a whole `ticket` row as sqlx reads it.

use domain::{StoreError, Ticket};
use time::OffsetDateTime;
use uuid::Uuid;

use super::column::{parse_close_reason, parse_count, parse_number, parse_status};

/// Every column, for a single ticket.
#[derive(sqlx::FromRow)]
pub(crate) struct TicketRow {
  id: Uuid,
  number: i64,
  requester_id: Uuid,
  assignee_id: Option<Uuid>,
  title: String,
  body: String,
  status: String,
  close_reason: Option<String>,
  reopen_count: i32,
  created_at: OffsetDateTime,
  updated_at: OffsetDateTime,
}

impl TicketRow {
  pub(crate) fn into_ticket(self) -> Result<Ticket, StoreError> {
    Ok(Ticket::from_persistence(
      self.id,
      parse_number(self.number)?,
      self.requester_id,
      self.assignee_id,
      self.title,
      self.body,
      parse_status(&self.status)?,
      self.close_reason.as_deref().map(parse_close_reason).transpose()?,
      parse_count(self.reopen_count)?,
      self.created_at,
      self.updated_at,
    ))
  }
}
