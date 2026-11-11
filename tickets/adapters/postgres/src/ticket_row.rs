//! `TicketRow` — a whole `ticket` row as sqlx reads it.

use domain::{StoreError, Ticket, TicketStatus};
use time::OffsetDateTime;
use uuid::Uuid;

/// Every column, for a single ticket.
#[derive(sqlx::FromRow)]
pub(crate) struct TicketRow {
  id: Uuid,
  requester_id: Uuid,
  assignee_id: Option<Uuid>,
  title: String,
  body: String,
  status: String,
  created_at: OffsetDateTime,
  updated_at: OffsetDateTime,
}

impl TicketRow {
  pub(crate) fn into_ticket(self) -> Result<Ticket, StoreError> {
    Ok(Ticket::from_persistence(
      self.id,
      self.requester_id,
      self.assignee_id,
      self.title,
      self.body,
      parse_status(&self.status)?,
      self.created_at,
      self.updated_at,
    ))
  }
}

pub(crate) fn parse_status(name: &str) -> Result<TicketStatus, StoreError> {
  TicketStatus::parse(name).ok_or_else(|| StoreError(format!("unknown ticket status {name:?}").into()))
}
