//! `TicketResponse` — the wire shape of one ticket; the domain type never serialises directly.

use domain::{Principal, Ticket};
use serde::Serialize;
use time::OffsetDateTime;
use uuid::Uuid;

use super::staff_view::StaffView;

#[derive(Serialize)]
pub struct TicketResponse {
  id: Uuid,
  reference: String,
  requester_id: Uuid,
  title: String,
  body: String,
  status: &'static str,
  #[serde(with = "time::serde::rfc3339")]
  created_at: OffsetDateTime,
  #[serde(with = "time::serde::rfc3339")]
  updated_at: OffsetDateTime,
  #[serde(flatten)]
  staff: Option<StaffView>,
}

impl TicketResponse {
  pub fn for_viewer(ticket: &Ticket, who: &Principal) -> Self {
    Self {
      id: ticket.id(),
      reference: ticket.number().to_string(),
      requester_id: ticket.requester_id(),
      title: ticket.title().to_owned(),
      body: ticket.body().to_owned(),
      status: ticket.status().as_str(),
      created_at: ticket.created_at(),
      updated_at: ticket.updated_at(),
      staff: StaffView::for_viewer(who, ticket.assignee_id()),
    }
  }
}
