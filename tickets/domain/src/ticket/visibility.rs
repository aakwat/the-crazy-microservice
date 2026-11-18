//! `Visibility` — who may read a message.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
  Public,
  /// Staff only: it must never reach a customer, not even as a count or an email.
  Internal,
}

impl Visibility {
  pub const ALL: [Self; 2] = [Self::Public, Self::Internal];

  /// The name stored in the database and sent in events.
  pub const fn as_str(self) -> &'static str {
    match self {
      Self::Public => "public",
      Self::Internal => "internal",
    }
  }

  pub fn parse(name: &str) -> Option<Self> {
    Self::ALL.into_iter().find(|visibility| visibility.as_str() == name)
  }
}
