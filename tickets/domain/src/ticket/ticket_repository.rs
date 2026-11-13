//! `TicketRepository` — the port the persistence adapter implements.

use time::OffsetDateTime;
use uuid::Uuid;

use super::{Ticket, TicketNumber, TicketSummary};
use crate::StoreError;

#[allow(async_fn_in_trait)]
pub trait TicketRepository: Send + Sync {
  /// The next unused number; gaps are allowed when a ticket is never stored.
  async fn next_number(&self) -> Result<TicketNumber, StoreError>;

  /// Inserts the ticket and its `ticket.created` outbox event in one transaction.
  /// `traceparent` goes into the event, so the change can be followed across services.
  async fn create(&self, ticket: &Ticket, traceparent: Option<&str>) -> Result<(), StoreError>;

  /// Writes the change and its `ticket.changed` outbox event in one transaction;
  /// `false` when the stored ticket no longer has `expected_updated_at` (someone else changed it).
  async fn update(
    &self,
    ticket: &Ticket,
    expected_updated_at: OffsetDateTime,
    traceparent: Option<&str>,
  ) -> Result<bool, StoreError>;

  async fn find(&self, id: Uuid) -> Result<Option<Ticket>, StoreError>;

  /// Newest first; `requester` limits to one requester, `before` continues after that ticket id.
  async fn list(
    &self,
    requester: Option<Uuid>,
    before: Option<Uuid>,
    limit: u32,
  ) -> Result<Vec<TicketSummary>, StoreError>;
}
