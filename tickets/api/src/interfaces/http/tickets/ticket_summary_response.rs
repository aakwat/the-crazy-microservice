//! `TicketSummaryResponse` — the wire shape of one list entry: no body.

use domain::{Principal, TicketSummary};
use serde::Serialize;
use time::OffsetDateTime;
use uuid::Uuid;

use super::staff_view::StaffView;

#[derive(Serialize)]
pub struct TicketSummaryResponse {
  id: Uuid,
  reference: String,
  requester_id: Uuid,
  title: String,
  status: &'static str,
  #[serde(with = "time::serde::rfc3339")]
  updated_at: OffsetDateTime,
  #[serde(flatten)]
  staff: Option<StaffView>,
}

impl TicketSummaryResponse {
  pub fn for_viewer(summary: TicketSummary, who: &Principal) -> Self {
    Self {
      id: summary.id,
      reference: summary.number.to_string(),
      requester_id: summary.requester_id,
      title: summary.title,
      status: summary.status.as_str(),
      updated_at: summary.updated_at,
      staff: StaffView::for_viewer(who, summary.assignee_id),
    }
  }
}
