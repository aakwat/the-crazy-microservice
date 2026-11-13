//! `TicketPageResponse` — the wire shape of one page of tickets.

use application::TicketPage;
use domain::Principal;
use serde::Serialize;
use uuid::Uuid;

use super::ticket_summary_response::TicketSummaryResponse;

#[derive(Serialize)]
pub struct TicketPageResponse {
  items: Vec<TicketSummaryResponse>,
  /// Pass back as `before` for the next page; absent on the last one.
  next_before: Option<Uuid>,
}

impl TicketPageResponse {
  pub fn for_viewer(page: TicketPage, who: &Principal) -> Self {
    Self {
      items: page
        .items
        .into_iter()
        .map(|summary| TicketSummaryResponse::for_viewer(summary, who))
        .collect(),
      next_before: page.next_before,
    }
  }
}
