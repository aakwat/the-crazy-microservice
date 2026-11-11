//! `TicketSummary` — the list view of a ticket.

use time::OffsetDateTime;
use uuid::Uuid;

use super::TicketStatus;

/// A read-only projection for lists: no body, so a page stays small.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TicketSummary {
  pub id: Uuid,
  pub requester_id: Uuid,
  pub assignee_id: Option<Uuid>,
  pub title: String,
  pub status: TicketStatus,
  pub updated_at: OffsetDateTime,
}
