//! `TicketChange` — what an agent asks to change on a ticket.

use super::{AssigneeChange, TicketStatus};

/// What an agent asks to change; `status: None` keeps the status.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TicketChange {
  pub status: Option<TicketStatus>,
  pub assignee: AssigneeChange,
}
