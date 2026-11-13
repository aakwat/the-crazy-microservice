//! `TicketSummaryRow` — the list columns of a `ticket` row: no body.

use domain::{StoreError, TicketSummary};
use time::OffsetDateTime;
use uuid::Uuid;

use super::ticket_row::{parse_number, parse_status};

#[derive(sqlx::FromRow)]
pub(crate) struct TicketSummaryRow {
  id: Uuid,
  number: i64,
  requester_id: Uuid,
  assignee_id: Option<Uuid>,
  title: String,
  status: String,
  updated_at: OffsetDateTime,
}

impl TicketSummaryRow {
  pub(crate) fn into_summary(self) -> Result<TicketSummary, StoreError> {
    Ok(TicketSummary {
      id: self.id,
      number: parse_number(self.number)?,
      requester_id: self.requester_id,
      assignee_id: self.assignee_id,
      title: self.title,
      status: parse_status(&self.status)?,
      updated_at: self.updated_at,
    })
  }
}
