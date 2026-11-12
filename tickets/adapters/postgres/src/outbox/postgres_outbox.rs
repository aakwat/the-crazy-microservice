//! `PostgresOutbox` — claims unpublished outbox rows in a transaction and purges old published ones.

use domain::{Outbox, OutboxEvent, StoreError};
use sqlx::{PgPool, query, query_as};
use time::OffsetDateTime;

use super::outbox_row::OutboxRow;
use super::postgres_outbox_batch::PostgresOutboxBatch;
use crate::store;

pub struct PostgresOutbox {
  pool: PgPool,
}

impl PostgresOutbox {
  pub fn new(pool: PgPool) -> Self {
    Self { pool }
  }
}

impl Outbox for PostgresOutbox {
  type Batch = PostgresOutboxBatch;

  /// SKIP LOCKED: replicas claim side by side and never get the same row.
  async fn claim(&self, limit: u32) -> Result<PostgresOutboxBatch, StoreError> {
    let mut tx = self.pool.begin().await.map_err(store)?;
    let rows: Vec<OutboxRow> = query_as(
      "SELECT id, stream, type, version, subject_id, traceparent, payload::TEXT AS payload, occurred_at
         FROM outbox
        WHERE published_at IS NULL
        ORDER BY occurred_at, id
        LIMIT $1
          FOR UPDATE SKIP LOCKED",
    )
    .bind(i64::from(limit))
    .fetch_all(&mut *tx)
    .await
    .map_err(store)?;

    let events = rows.into_iter().map(OutboxEvent::from).collect();
    Ok(PostgresOutboxBatch::new(tx, events))
  }

  async fn purge_published(&self, before: OffsetDateTime) -> Result<u64, StoreError> {
    let done = query("DELETE FROM outbox WHERE published_at < $1")
      .bind(before)
      .execute(&self.pool)
      .await
      .map_err(store)?;
    Ok(done.rows_affected())
  }
}
