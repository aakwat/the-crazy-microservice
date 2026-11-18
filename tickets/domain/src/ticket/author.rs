//! `Author` — which side of the conversation wrote a message.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Author {
  Customer,
  Staff,
}

impl Author {
  pub const ALL: [Self; 2] = [Self::Customer, Self::Staff];

  /// The name stored in the database and sent in events.
  pub const fn as_str(self) -> &'static str {
    match self {
      Self::Customer => "customer",
      Self::Staff => "staff",
    }
  }

  pub fn parse(name: &str) -> Option<Self> {
    Self::ALL.into_iter().find(|author| author.as_str() == name)
  }
}
