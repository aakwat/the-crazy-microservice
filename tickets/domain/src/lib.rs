//! Tickets business rules: no I/O and no framework, so every other crate can depend on it.

mod access_token_verifier;
mod outbox;
mod principal;
mod store_error;
mod ticket;
mod token_error;

pub use access_token_verifier::AccessTokenVerifier;
pub use outbox::{EventPublisher, Outbox, OutboxBatch, OutboxEvent, PublishError};
pub use principal::{AGENT_ROLE, Principal};
pub use store_error::StoreError;
pub use ticket::{
  AssigneeChange, MAX_BODY_CHARS, MAX_TITLE_CHARS, Ticket, TicketChange, TicketError, TicketNumber, TicketRepository,
  TicketStatus, TicketSummary,
};
pub use token_error::TokenError;
