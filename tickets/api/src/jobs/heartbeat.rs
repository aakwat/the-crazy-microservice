//! `Heartbeat` — when the outbox worker last finished a round, so health can tell a stuck worker.

use std::sync::atomic::{AtomicI64, Ordering};
use std::time::Duration;

use time::OffsetDateTime;

pub struct Heartbeat {
  last_unix: AtomicI64,
}

impl Heartbeat {
  pub fn new(now: OffsetDateTime) -> Self {
    Self {
      last_unix: AtomicI64::new(now.unix_timestamp()),
    }
  }

  pub fn beat(&self, now: OffsetDateTime) {
    self.last_unix.store(now.unix_timestamp(), Ordering::Relaxed);
  }

  pub fn is_silent_for(&self, max: Duration, now: OffsetDateTime) -> bool {
    let silent = now.unix_timestamp() - self.last_unix.load(Ordering::Relaxed);
    u64::try_from(silent).is_ok_and(|seconds| seconds > max.as_secs())
  }
}
