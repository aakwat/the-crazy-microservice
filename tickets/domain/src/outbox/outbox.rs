//! `Outbox` — the port over stored events: claim a batch to publish, forget old published ones.

use time::OffsetDateTime;

use super::OutboxBatch;
use crate::StoreError;

#[allow(async_fn_in_trait)]
pub trait Outbox: Send + Sync {
  type Batch: OutboxBatch;

  /// Oldest unpublished events no other relay holds; they stay locked until the batch completes or drops.
  async fn claim(&self, limit: u32) -> Result<Self::Batch, StoreError>;

  async fn purge_published(&self, before: OffsetDateTime) -> Result<u64, StoreError>;
}
