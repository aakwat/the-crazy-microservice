//! `TicketStatus` — where a ticket is in its life.

/// Where a ticket is in its life; it only moves one step forward.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TicketStatus {
  Open,
  InProgress,
  Resolved,
  Closed,
}

impl TicketStatus {
  pub const ALL: [Self; 4] = [Self::Open, Self::InProgress, Self::Resolved, Self::Closed];

  /// The name stored in the database and sent over the wire.
  pub const fn as_str(self) -> &'static str {
    match self {
      Self::Open => "open",
      Self::InProgress => "in_progress",
      Self::Resolved => "resolved",
      Self::Closed => "closed",
    }
  }

  pub fn parse(name: &str) -> Option<Self> {
    Self::ALL.into_iter().find(|status| status.as_str() == name)
  }

  /// The only status this one may move to.
  pub const fn next(self) -> Option<Self> {
    match self {
      Self::Open => Some(Self::InProgress),
      Self::InProgress => Some(Self::Resolved),
      Self::Resolved => Some(Self::Closed),
      Self::Closed => None,
    }
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
  fn unknown_name_is_none() {
    assert_eq!(TicketStatus::parse("reopened"), None);
  }

  #[test]
  fn closed_is_final() {
    assert_eq!(TicketStatus::Closed.next(), None);
  }
}
