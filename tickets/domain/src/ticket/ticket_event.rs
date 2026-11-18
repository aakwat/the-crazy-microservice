//! `TicketEvent` — what a ticket action changed, so the repository knows which outbox events to write.

use super::Message;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TicketEvent {
  StatusChanged,
  Assigned,
  MessageAdded(Message),
}
