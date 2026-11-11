//! `TicketError` — why a ticket cannot be opened or changed as asked.

use super::{MAX_BODY_CHARS, MAX_TITLE_CHARS, TicketStatus};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TicketError {
  #[error("title must be 1 to {MAX_TITLE_CHARS} characters")]
  Title,
  #[error("body must be 1 to {MAX_BODY_CHARS} characters")]
  Body,
  #[error("a closed ticket cannot change")]
  Closed,
  #[error("status cannot move from {} to {}", from.as_str(), to.as_str())]
  Transition { from: TicketStatus, to: TicketStatus },
}
