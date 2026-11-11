//! `TicketSummaryResponse` — the wire shape of one list entry: no body.

use domain::TicketSummary;
use serde::Serialize;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Serialize)]
pub struct TicketSummaryResponse {
  id: Uuid,
  requester_id: Uuid,
  assignee_id: Option<Uuid>,
  title: String,
  status: &'static str,
  #[serde(with = "time::serde::rfc3339")]
  updated_at: OffsetDateTime,
}

impl From<TicketSummary> for TicketSummaryResponse {
  fn from(summary: TicketSummary) -> Self {
    Self {
      id: summary.id,
      requester_id: summary.requester_id,
      assignee_id: summary.assignee_id,
      title: summary.title,
      status: summary.status.as_str(),
      updated_at: summary.updated_at,
    }
  }
}
