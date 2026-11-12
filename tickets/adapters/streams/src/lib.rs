//! Redis Streams adapter: publishes outbox events in the contract envelope.

mod bus_address;
mod envelope;
mod redis_event_publisher;

use domain::PublishError;

pub use bus_address::BusAddress;
pub use envelope::ENVELOPE_FIELD;
pub use redis_event_publisher::{MAX_STREAM_LENGTH, RedisEventPublisher};

fn publish_error(error: impl std::error::Error + Send + Sync + 'static) -> PublishError {
  PublishError(Box::new(error))
}
