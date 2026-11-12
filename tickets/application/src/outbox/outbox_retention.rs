//! `OutboxRetention` — forgets published events once they are too old to help debugging.

use domain::{Outbox, StoreError};
use time::OffsetDateTime;

/// Published events are kept this long, then purged.
pub const RETENTION: time::Duration = time::Duration::days(7);

pub struct OutboxRetention<O> {
  outbox: O,
}

impl<O: Outbox> OutboxRetention<O> {
  pub fn new(outbox: O) -> Self {
    Self { outbox }
  }

  pub async fn purge(&self, now: OffsetDateTime) -> Result<u64, StoreError> {
    self.outbox.purge_published(now - RETENTION).await
  }
}
