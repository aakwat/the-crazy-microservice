//! `EventPublisher` — the port that puts one event on the message bus.

use super::{OutboxEvent, PublishError};

#[allow(async_fn_in_trait)]
pub trait EventPublisher: Send + Sync {
  /// Delivery is at least once: consumers dedupe on the event id.
  async fn publish(&self, event: &OutboxEvent) -> Result<(), PublishError>;
}
