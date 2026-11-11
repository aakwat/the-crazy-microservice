//! `Principal` — who is calling, reduced to what the ticket rules need.

use uuid::Uuid;

/// The role name auth puts in the access token's `roles` claim for support staff.
pub const AGENT_ROLE: &str = "agent";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Principal {
  subject: Uuid,
  roles: Vec<String>,
}

impl Principal {
  pub fn new(subject: Uuid, roles: Vec<String>) -> Self {
    Self { subject, roles }
  }

  pub fn subject(&self) -> Uuid {
    self.subject
  }

  pub fn is_agent(&self) -> bool {
    self.roles.iter().any(|role| role == AGENT_ROLE)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn agent_role_makes_an_agent() {
    let agent = Principal::new(Uuid::now_v7(), vec!["agent".to_owned()]);
    let customer = Principal::new(Uuid::now_v7(), vec!["customer".to_owned()]);

    assert!(agent.is_agent());
    assert!(!customer.is_agent());
  }
}
