//! `PostgresOutboxBatch` — claimed rows and the transaction that keeps them locked.

use domain::{OutboxBatch, OutboxEvent, StoreError};
use sqlx::{Postgres, Transaction, query};
use uuid::Uuid;

use crate::store;

pub struct PostgresOutboxBatch {
  tx: Transaction<'static, Postgres>,
  events: Vec<OutboxEvent>,
}

impl PostgresOutboxBatch {
  pub(super) fn new(tx: Transaction<'static, Postgres>, events: Vec<OutboxEvent>) -> Self {
    Self { tx, events }
  }
}

impl OutboxBatch for PostgresOutboxBatch {
  fn events(&self) -> &[OutboxEvent] {
    &self.events
  }

  async fn complete(mut self, published: &[Uuid]) -> Result<(), StoreError> {
    if !published.is_empty() {
      query("UPDATE outbox SET published_at = now() WHERE id = ANY($1)")
        .bind(published)
        .execute(&mut *self.tx)
        .await
        .map_err(store)?;
    }
    self.tx.commit().await.map_err(store)
  }
}
