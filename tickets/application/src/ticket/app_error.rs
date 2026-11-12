//! `AppError` — why a use case failed, in terms the API maps to HTTP.

use domain::{StoreError, TicketError};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
  /// Missing, or not visible to the caller; the two look the same so existence never leaks.
  #[error("ticket not found")]
  NotFound,
  #[error("only agents may do this")]
  Forbidden,
  #[error("the ticket changed meanwhile; reload and retry")]
  Conflict,
  #[error(transparent)]
  Invalid(#[from] TicketError),
  #[error(transparent)]
  Store(#[from] StoreError),
}
