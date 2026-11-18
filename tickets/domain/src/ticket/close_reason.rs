//! `CloseReason` — why a ticket is closed.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloseReason {
  Confirmed,
  Withdrawn,
}

impl CloseReason {
  pub const ALL: [Self; 2] = [Self::Confirmed, Self::Withdrawn];

  /// The name stored in the database and sent over the wire.
  pub const fn as_str(self) -> &'static str {
    match self {
      Self::Confirmed => "confirmed",
      Self::Withdrawn => "withdrawn",
    }
  }

  pub fn parse(name: &str) -> Option<Self> {
    Self::ALL.into_iter().find(|reason| reason.as_str() == name)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn names_round_trip() {
    for reason in CloseReason::ALL {
      assert_eq!(CloseReason::parse(reason.as_str()), Some(reason));
    }
  }
}
