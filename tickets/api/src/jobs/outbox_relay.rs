//! The outbox relay job: rounds back to back until shutdown; a failed round backs off, it never ends the loop.

use std::sync::Arc;
use std::time::Duration;

use application::OutboxRelay;
use domain::{EventPublisher, Outbox};
use postgres::PostgresOutbox;
use streams::RedisEventPublisher;
use time::OffsetDateTime;
use tokio::sync::watch;
use tokio::task::JoinHandle;

use super::heartbeat::Heartbeat;

pub type TicketsOutboxRelay = OutboxRelay<PostgresOutbox, RedisEventPublisher>;

// Concrete types: tokio::spawn needs Send futures, which async trait methods only prove per implementation.
pub fn spawn(relay: TicketsOutboxRelay, heartbeat: Arc<Heartbeat>, shutdown: watch::Receiver<bool>) -> JoinHandle<()> {
  tokio::spawn(run(relay, heartbeat, shutdown))
}

async fn run<O: Outbox, P: EventPublisher>(
  mut relay: OutboxRelay<O, P>,
  heartbeat: Arc<Heartbeat>,
  mut shutdown: watch::Receiver<bool>,
) {
  loop {
    let wait = round(&mut relay).await;
    heartbeat.beat(OffsetDateTime::now_utc());

    // Shutdown is heard only between rounds, so a round in flight always finishes.
    tokio::select! {
      () = tokio::time::sleep(wait) => {}
      _ = shutdown.changed() => {
        tracing::info!(event = "outbox.relay.shutdown");
        return;
      }
    }
  }
}

async fn round<O: Outbox, P: EventPublisher>(relay: &mut OutboxRelay<O, P>) -> Duration {
  match relay.step().await {
    Ok(wait) => wait,
    Err(error) => {
      let wait = relay.backoff();
      tracing::warn!(event = "outbox.relay.round_failed", %error, retry_in_ms = wait.as_millis());
      wait
    }
  }
}
