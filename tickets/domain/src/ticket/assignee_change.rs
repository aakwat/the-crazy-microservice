//! `AssigneeChange` — what an agent asks to do with a ticket's assignee.

use uuid::Uuid;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum AssigneeChange {
  #[default]
  Keep,
  Assign(Uuid),
  Unassign,
}

impl AssigneeChange {
  /// The assignee after this change, given the current one.
  pub fn applied_to(self, current: Option<Uuid>) -> Option<Uuid> {
    match self {
      Self::Keep => current,
      Self::Assign(id) => Some(id),
      Self::Unassign => None,
    }
  }
}
