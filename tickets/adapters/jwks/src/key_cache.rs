//! `KeyCache` — auth's signing keys by kid, with when they were fetched and when a fetch was last tried.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use jsonwebtoken::DecodingKey;

#[derive(Default)]
pub(crate) struct KeyCache {
  keys: HashMap<String, DecodingKey>,
  fetched_at: Option<Instant>,
  attempted_at: Option<Instant>,
}

impl KeyCache {
  pub(crate) fn get(&self, kid: &str) -> Option<DecodingKey> {
    self.keys.get(kid).cloned()
  }

  /// The key, if the set it came from was fetched less than `max_age` ago.
  pub(crate) fn get_younger_than(&self, kid: &str, max_age: Duration) -> Option<DecodingKey> {
    if within(self.fetched_at, max_age) {
      self.get(kid)
    } else {
      None
    }
  }

  pub(crate) fn attempted_within(&self, period: Duration) -> bool {
    within(self.attempted_at, period)
  }

  pub(crate) fn mark_attempt(&mut self) {
    self.attempted_at = Some(Instant::now());
  }

  pub(crate) fn replace(&mut self, keys: HashMap<String, DecodingKey>) {
    self.keys = keys;
    self.fetched_at = Some(Instant::now());
  }
}

fn within(at: Option<Instant>, period: Duration) -> bool {
  at.is_some_and(|at| at.elapsed() < period)
}
