//! `TicketRow` — a whole `ticket` row as sqlx reads it.

use domain::{StoreError, Ticket, TicketNumber, TicketStatus};
use time::OffsetDateTime;
use uuid::Uuid;

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
      self.created_at,
      self.updated_at,
    ))
  }
}

pub(crate) fn parse_status(name: &str) -> Result<TicketStatus, StoreError> {
  TicketStatus::parse(name).ok_or_else(|| StoreError(format!("unknown ticket status {name:?}").into()))
}

pub(crate) fn parse_number(stored: i64) -> Result<TicketNumber, StoreError> {
  u64::try_from(stored)
    .map(TicketNumber::new)
    .map_err(|_| StoreError(format!("bad ticket number {stored}").into()))
}

pub(crate) fn stored_number(number: TicketNumber) -> Result<i64, StoreError> {
  i64::try_from(number.value()).map_err(|_| StoreError(format!("ticket number {number} too large").into()))
}
