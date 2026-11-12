//! `RedisEventPublisher` — adds each outbox event to its Redis stream.

use std::time::Duration;

use domain::{EventPublisher, OutboxEvent, PublishError};
use redis::aio::{ConnectionManager, ConnectionManagerConfig};
use redis::streams::StreamMaxlen;
use redis::{AsyncCommands, Client};

use crate::envelope::{ENVELOPE_FIELD, envelope};
use crate::{BusAddress, publish_error};

/// Streams are trimmed near this length; a consumer further behind than that misses events.
pub const MAX_STREAM_LENGTH: usize = 100_000;

pub struct RedisEventPublisher {
  connection: ConnectionManager,
}

impl RedisEventPublisher {
  /// Connects on first use, so tickets starts and keeps taking writes while the bus is down.
  pub fn new(address: BusAddress, timeout: Duration) -> Result<Self, PublishError> {
    let client = Client::open(address.connection_info()?).map_err(publish_error)?;
    // One retry per publish: the relay has its own backoff.
    let config = ConnectionManagerConfig::new()
      .set_connection_timeout(Some(timeout))
      .set_response_timeout(Some(timeout))
      .set_number_of_retries(1);
    let connection = ConnectionManager::new_lazy_with_config(client, config).map_err(publish_error)?;
    Ok(Self { connection })
  }
}

impl EventPublisher for RedisEventPublisher {
  async fn publish(&self, event: &OutboxEvent) -> Result<(), PublishError> {
    let entry = [(ENVELOPE_FIELD, envelope(event)?)];
    let mut connection = self.connection.clone();

    let _: String = connection
      .xadd_maxlen(&event.stream, StreamMaxlen::Approx(MAX_STREAM_LENGTH), "*", &entry)
      .await
      .map_err(publish_error)?;
    Ok(())
  }
}
