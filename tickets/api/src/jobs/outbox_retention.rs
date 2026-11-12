//! The outbox retention job: purges old published events on a fixed interval.

use std::time::Duration;

use application::OutboxRetention;
use domain::Outbox;
use postgres::PostgresOutbox;
use time::OffsetDateTime;
use tokio::task::JoinHandle;

pub type TicketsOutboxRetention = OutboxRetention<PostgresOutbox>;

/// Delete-only and idempotent, so it needs no shutdown signal: a purge cut short simply runs again.
pub fn spawn(retention: TicketsOutboxRetention, interval: Duration) -> JoinHandle<()> {
  tokio::spawn(run(retention, interval))
}

async fn run<O: Outbox>(retention: OutboxRetention<O>, interval: Duration) {
  loop {
    purge(&retention).await;
    tokio::time::sleep(interval).await;
  }
}

async fn purge<O: Outbox>(retention: &OutboxRetention<O>) {
  match retention.purge(OffsetDateTime::now_utc()).await {
    Ok(0) => {}
    Ok(removed) => tracing::info!(event = "outbox.retention.purged", removed),
    Err(error) => tracing::warn!(event = "outbox.retention.failed", %error),
  }
}
