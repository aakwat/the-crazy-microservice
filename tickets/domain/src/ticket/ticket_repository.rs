//! `TicketRepository` — the port the persistence adapter implements.

use time::OffsetDateTime;
use uuid::Uuid;

use super::{Message, Ticket, TicketEvent, TicketNumber, TicketSummary};
use crate::StoreError;

#[allow(async_fn_in_trait)]
pub trait TicketRepository: Send + Sync {
  /// The next unused number; gaps are allowed when a ticket is never stored.
  async fn next_number(&self) -> Result<TicketNumber, StoreError>;

  /// Inserts the ticket and its `ticket.created` outbox event in one transaction.
  /// `traceparent` goes into the event, so the change can be followed across services.
  async fn create(&self, ticket: &Ticket, traceparent: Option<&str>) -> Result<(), StoreError>;

  /// `false` when someone else changed the ticket after `expected_updated_at`, so nothing is overwritten.
  async fn save(
    &self,
    ticket: &Ticket,
    expected_updated_at: OffsetDateTime,
    events: &[TicketEvent],
    traceparent: Option<&str>,
  ) -> Result<bool, StoreError>;

  async fn find(&self, id: Uuid) -> Result<Option<Ticket>, StoreError>;

  /// Filters notes in SQL, so a customer's request never even loads one.
  async fn messages(&self, ticket_id: Uuid, include_internal: bool) -> Result<Vec<Message>, StoreError>;

  /// Newest first; `requester` limits to one requester, `before` continues after that ticket id.
  async fn list(
    &self,
    requester: Option<Uuid>,
    before: Option<Uuid>,
    limit: u32,
  ) -> Result<Vec<TicketSummary>, StoreError>;
}
