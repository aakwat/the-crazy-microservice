//! Tickets use cases: who may do what, on top of the domain rules and its ports.

mod outbox;
mod ticket;

pub use outbox::{OutboxRelay, OutboxRetention, RelayError};
pub use ticket::{AppError, TicketPage, TicketService};
