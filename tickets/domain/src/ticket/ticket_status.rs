//! `TicketStatus` — where a ticket is in its life (docs/business-rules.md 13.1).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TicketStatus {
  New,
  Open,
  Pending,
  Resolved,
  Closed,
}

impl TicketStatus {
  pub const ALL: [Self; 5] = [Self::New, Self::Open, Self::Pending, Self::Resolved, Self::Closed];

  /// The name stored in the database and sent over the wire.
  pub const fn as_str(self) -> &'static str {
    match self {
      Self::New => "new",
      Self::Open => "open",
      Self::Pending => "pending",
      Self::Resolved => "resolved",
      Self::Closed => "closed",
    }
  }

  pub fn parse(name: &str) -> Option<Self> {
    Self::ALL.into_iter().find(|status| status.as_str() == name)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn names_round_trip() {
    for status in TicketStatus::ALL {
      assert_eq!(TicketStatus::parse(status.as_str()), Some(status));
    }
  }

  #[test]
  fn reopened_is_not_a_status() {
    assert_eq!(TicketStatus::parse("reopened"), None);
  }
}
