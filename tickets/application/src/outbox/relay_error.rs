//! `RelayError` — why a relay round stopped: the store or the bus.

use domain::{PublishError, StoreError};

#[derive(Debug, thiserror::Error)]
pub enum RelayError {
  #[error(transparent)]
  Store(#[from] StoreError),
  #[error(transparent)]
  Publish(#[from] PublishError),
}
