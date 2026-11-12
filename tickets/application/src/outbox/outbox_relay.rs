//! `OutboxRelay` — publishes claimed outbox events in order and decides when the next round runs.

use std::time::Duration;

use domain::{EventPublisher, Outbox, OutboxBatch, OutboxEvent, PublishError};
use uuid::Uuid;

use super::RelayError;

/// Events claimed per round; small enough that one round holds its row locks only briefly.
pub const BATCH_SIZE: u32 = 100;
/// Wait between rounds once the outbox is drained.
pub const POLL_INTERVAL: Duration = Duration::from_millis(500);
/// Longest wait after repeated failures, so a bus outage costs one attempt per half minute.
pub const MAX_BACKOFF: Duration = Duration::from_secs(30);

pub struct OutboxRelay<O, P> {
  outbox: O,
  publisher: P,
  failures: u32,
}

impl<O: Outbox, P: EventPublisher> OutboxRelay<O, P> {
  pub fn new(outbox: O, publisher: P) -> Self {
    Self {
      outbox,
      publisher,
      failures: 0,
    }
  }

  /// Ok is the wait before the next round; after an error, wait `backoff()` instead.
  pub async fn step(&mut self) -> Result<Duration, RelayError> {
    let claimed = self.relay_batch().await.inspect_err(|_| self.failures += 1)?;
    self.failures = 0;

    let full = u32::try_from(claimed).is_ok_and(|claimed| claimed == BATCH_SIZE);
    Ok(if full { Duration::ZERO } else { POLL_INTERVAL })
  }

  pub fn backoff(&self) -> Duration {
    POLL_INTERVAL.saturating_mul(1 << self.failures.min(6)).min(MAX_BACKOFF)
  }

  /// Returns how many events were claimed.
  async fn relay_batch(&self) -> Result<usize, RelayError> {
    let batch = self.outbox.claim(BATCH_SIZE).await?;
    let claimed = batch.events().len();
    let (published, failure) = publish_in_order(&self.publisher, batch.events()).await;

    batch.complete(&published).await?;
    match failure {
      Some(error) => Err(error.into()),
      None => Ok(claimed),
    }
  }
}

/// Stops at the first failure, so events never leave out of order.
async fn publish_in_order(
  publisher: &impl EventPublisher,
  events: &[OutboxEvent],
) -> (Vec<Uuid>, Option<PublishError>) {
  let mut sent = Vec::with_capacity(events.len());
  for event in events {
    if let Err(error) = publisher.publish(event).await {
      return (sent, Some(error));
    }
    sent.push(event.id);
  }
  (sent, None)
}

#[cfg(test)]
mod tests {
  use std::collections::VecDeque;
  use std::sync::{Arc, Mutex};

  use domain::StoreError;
  use time::OffsetDateTime;
  use time::macros::datetime;

  use super::*;

  type Completed = Arc<Mutex<Vec<Vec<Uuid>>>>;

  /// Hands out scripted batches and records what each one was completed with.
  #[derive(Default)]
  struct FakeOutbox {
    claims: Mutex<VecDeque<Result<Vec<OutboxEvent>, StoreError>>>,
    completed: Completed,
  }

  struct FakeBatch {
    events: Vec<OutboxEvent>,
    completed: Completed,
  }

  impl Outbox for FakeOutbox {
    type Batch = FakeBatch;

    fn claim(&self, _: u32) -> impl Future<Output = Result<FakeBatch, StoreError>> {
      let claim = self
        .claims
        .lock()
        .unwrap()
        .pop_front()
        .unwrap_or_else(|| Ok(Vec::new()));
      let completed = Arc::clone(&self.completed);
      std::future::ready(claim.map(|events| FakeBatch { events, completed }))
    }

    fn purge_published(&self, _: OffsetDateTime) -> impl Future<Output = Result<u64, StoreError>> {
      std::future::ready(Ok(0))
    }
  }

  impl OutboxBatch for FakeBatch {
    fn events(&self) -> &[OutboxEvent] {
      &self.events
    }

    fn complete(self, published: &[Uuid]) -> impl Future<Output = Result<(), StoreError>> {
      self.completed.lock().unwrap().push(published.to_vec());
      std::future::ready(Ok(()))
    }
  }

  /// Accepts every event except the one it was told to refuse.
  #[derive(Default)]
  struct FakePublisher {
    refuse: Option<Uuid>,
  }

  impl EventPublisher for FakePublisher {
    fn publish(&self, event: &OutboxEvent) -> impl Future<Output = Result<(), PublishError>> {
      let refused = self.refuse == Some(event.id);
      std::future::ready(if refused {
        Err(PublishError(boxed("bus down")))
      } else {
        Ok(())
      })
    }
  }

  fn boxed(message: &str) -> Box<dyn std::error::Error + Send + Sync> {
    Box::new(std::io::Error::other(message.to_owned()))
  }

  fn events(count: usize) -> Vec<OutboxEvent> {
    (0..count)
      .map(|_| OutboxEvent {
        id: Uuid::now_v7(),
        stream: "tickets.events".to_owned(),
        event_type: "ticket.created".to_owned(),
        version: 1,
        subject_id: Uuid::now_v7(),
        traceparent: None,
        payload: "{}".to_owned(),
        occurred_at: datetime!(2026-11-12 09:00 UTC),
      })
      .collect()
  }

  fn relay(
    claims: Vec<Result<Vec<OutboxEvent>, StoreError>>,
    publisher: FakePublisher,
  ) -> OutboxRelay<FakeOutbox, FakePublisher> {
    let outbox = FakeOutbox {
      claims: Mutex::new(claims.into()),
      ..FakeOutbox::default()
    };
    OutboxRelay::new(outbox, publisher)
  }

  #[tokio::test]
  async fn a_full_batch_runs_the_next_round_at_once_and_a_short_one_waits() {
    let mut relay = relay(vec![Ok(events(100)), Ok(events(3))], FakePublisher::default());

    assert_eq!(relay.step().await.unwrap(), Duration::ZERO);
    assert_eq!(relay.step().await.unwrap(), POLL_INTERVAL);
  }

  #[tokio::test]
  async fn a_refused_event_stops_the_batch_and_only_earlier_events_count_as_published() {
    let batch = events(3);
    let publisher = FakePublisher {
      refuse: Some(batch[1].id),
    };
    let mut relay = relay(vec![Ok(batch.clone())], publisher);

    let result = relay.step().await;

    assert!(matches!(result, Err(RelayError::Publish(_))));
    assert_eq!(*relay.outbox.completed.lock().unwrap(), vec![vec![batch[0].id]]);
  }

  #[tokio::test]
  async fn failures_back_off_up_to_the_cap_and_a_success_resets_it() {
    let claims = (0..8).map(|_| Err(StoreError(boxed("db down")))).collect();
    let mut relay = relay(claims, FakePublisher::default());

    let mut waits = Vec::new();
    for _ in 0..8 {
      assert!(relay.step().await.is_err());
      waits.push(relay.backoff());
    }
    relay.step().await.unwrap();

    assert_eq!(waits[0], Duration::from_secs(1));
    assert_eq!(waits[1], Duration::from_secs(2));
    assert_eq!(waits[7], MAX_BACKOFF);
    assert_eq!(relay.backoff(), POLL_INTERVAL);
  }
}
