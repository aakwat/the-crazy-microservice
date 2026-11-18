//! `AssigneeChange` — what an agent asks to do with a ticket's assignee.

use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssigneeChange {
  Assign(Uuid),
  Unassign,
}

impl AssigneeChange {
  /// The assignee after this change.
  pub fn assignee(self) -> Option<Uuid> {
    match self {
      Self::Assign(id) => Some(id),
      Self::Unassign => None,
    }
  }
}
