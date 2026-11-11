//! `TicketResponse` — the wire shape of one ticket; the domain type never serialises directly.

use domain::Ticket;
use serde::Serialize;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Serialize)]
pub struct TicketResponse {
  id: Uuid,
  requester_id: Uuid,
  assignee_id: Option<Uuid>,
  title: String,
  body: String,
  status: &'static str,
  #[serde(with = "time::serde::rfc3339")]
  created_at: OffsetDateTime,
  #[serde(with = "time::serde::rfc3339")]
  updated_at: OffsetDateTime,
}

impl From<Ticket> for TicketResponse {
  fn from(ticket: Ticket) -> Self {
    Self {
      id: ticket.id(),
      requester_id: ticket.requester_id(),
      assignee_id: ticket.assignee_id(),
      title: ticket.title().to_owned(),
      body: ticket.body().to_owned(),
      status: ticket.status().as_str(),
      created_at: ticket.created_at(),
      updated_at: ticket.updated_at(),
    }
  }
}
