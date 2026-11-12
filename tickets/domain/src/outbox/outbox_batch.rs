//! `OutboxBatch` — claimed events held until the relay reports which ones it published.

use uuid::Uuid;

use super::OutboxEvent;
use crate::StoreError;

#[allow(async_fn_in_trait)]
pub trait OutboxBatch: Send {
  /// Oldest first.
  fn events(&self) -> &[OutboxEvent];

  /// Marks `published` and releases the rest for the next claim; dropping the batch releases all of them.
  async fn complete(self, published: &[Uuid]) -> Result<(), StoreError>;
}
