//! `TicketResponse` — the wire shape of one ticket; the domain type never serialises directly.

use domain::{CloseReason, Principal, Ticket};
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
  close_reason: Option<&'static str>,
  /// Staff only: how often the customer rejected a resolution.
  #[serde(skip_serializing_if = "Option::is_none")]
  reopen_count: Option<u32>,
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
      close_reason: ticket.close_reason().map(CloseReason::as_str),
      reopen_count: who.is_agent().then_some(ticket.reopen_count()),
      created_at: ticket.created_at(),
      updated_at: ticket.updated_at(),
      staff: StaffView::for_viewer(who, ticket.assignee_id()),
    }
  }
}
