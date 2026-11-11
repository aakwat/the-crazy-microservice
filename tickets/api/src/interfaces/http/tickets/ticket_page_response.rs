//! `TicketPageResponse` — the wire shape of one page of tickets.

use application::TicketPage;
use serde::Serialize;
use uuid::Uuid;

use super::ticket_summary_response::TicketSummaryResponse;

#[derive(Serialize)]
pub struct TicketPageResponse {
  items: Vec<TicketSummaryResponse>,
  /// Pass back as `before` for the next page; absent on the last one.
  next_before: Option<Uuid>,
}

impl From<TicketPage> for TicketPageResponse {
  fn from(page: TicketPage) -> Self {
    Self {
      items: page.items.into_iter().map(TicketSummaryResponse::from).collect(),
      next_before: page.next_before,
    }
  }
}
