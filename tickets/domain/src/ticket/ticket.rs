//! `Ticket` — a support ticket and the rules for changing it.

use time::OffsetDateTime;
use uuid::Uuid;

use super::{TicketChange, TicketError, TicketNumber, TicketStatus};

/// Longest title in characters; the database enforces the same limit.
pub const MAX_TITLE_CHARS: usize = 200;
/// Longest body in characters; the database enforces the same limit.
pub const MAX_BODY_CHARS: usize = 10_000;

/// A support ticket; create it with [`Ticket::open`] and change it only through [`Ticket::apply`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ticket {
  id: Uuid,
  number: TicketNumber,
  requester_id: Uuid,
  assignee_id: Option<Uuid>,
  title: String,
  body: String,
  status: TicketStatus,
  created_at: OffsetDateTime,
  updated_at: OffsetDateTime,
}

impl Ticket {
  /// Opens a ticket; both texts are trimmed and must fit their limits.
  pub fn open(
    number: TicketNumber,
    requester_id: Uuid,
    title: &str,
    body: &str,
    now: OffsetDateTime,
  ) -> Result<Self, TicketError> {
    let (title, body, now) = (title.trim(), body.trim(), stored_precision(now));
    if !fits(title, MAX_TITLE_CHARS) {
      return Err(TicketError::Title);
    }
    if !fits(body, MAX_BODY_CHARS) {
      return Err(TicketError::Body);
    }
    Ok(Self {
      id: Uuid::now_v7(),
      number,
      requester_id,
      assignee_id: None,
      title: title.to_owned(),
      body: body.to_owned(),
      status: TicketStatus::Open,
      created_at: now,
      updated_at: now,
    })
  }

  /// Applies an agent's change and reports whether anything changed.
  pub fn apply(&mut self, change: TicketChange, now: OffsetDateTime) -> Result<bool, TicketError> {
    if self.status == TicketStatus::Closed {
      return Err(TicketError::Closed);
    }
    let status = change.status.unwrap_or(self.status);
    if status != self.status && self.status.next() != Some(status) {
      return Err(TicketError::Transition {
        from: self.status,
        to: status,
      });
    }
    let assignee_id = change.assignee.applied_to(self.assignee_id);
    if status == self.status && assignee_id == self.assignee_id {
      return Ok(false);
    }
    self.status = status;
    self.assignee_id = assignee_id;
    // Another replica's clock may run behind; time never moves backwards for one ticket.
    self.updated_at = stored_precision(now).max(self.updated_at);
    Ok(true)
  }
}

impl Ticket {
  /// Rebuilds a stored ticket; only the persistence adapter calls this.
  #[allow(
    clippy::too_many_arguments,
    reason = "one argument per column keeps the adapter mapping obvious"
  )]
  pub fn from_persistence(
    id: Uuid,
    number: TicketNumber,
    requester_id: Uuid,
    assignee_id: Option<Uuid>,
    title: String,
    body: String,
    status: TicketStatus,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
  ) -> Self {
    Self {
      id,
      number,
      requester_id,
      assignee_id,
      title,
      body,
      status,
      created_at,
      updated_at,
    }
  }

  pub fn id(&self) -> Uuid {
    self.id
  }

  pub fn number(&self) -> TicketNumber {
    self.number
  }

  pub fn requester_id(&self) -> Uuid {
    self.requester_id
  }

  pub fn assignee_id(&self) -> Option<Uuid> {
    self.assignee_id
  }

  pub fn title(&self) -> &str {
    &self.title
  }

  pub fn body(&self) -> &str {
    &self.body
  }

  pub fn status(&self) -> TicketStatus {
    self.status
  }

  pub fn created_at(&self) -> OffsetDateTime {
    self.created_at
  }

  pub fn updated_at(&self) -> OffsetDateTime {
    self.updated_at
  }
}

/// Postgres rejects NUL in text, so it is refused here rather than failing as a server error.
fn fits(text: &str, max_chars: usize) -> bool {
  !text.is_empty() && !text.contains('\0') && text.chars().count() <= max_chars
}

/// Postgres keeps microseconds; matching it keeps `updated_at` exact for the stale-write check.
fn stored_precision(time: OffsetDateTime) -> OffsetDateTime {
  time
    .replace_nanosecond(time.nanosecond() / 1_000 * 1_000)
    .unwrap_or(time)
}

#[cfg(test)]
mod tests {
  use time::macros::datetime;

  use crate::AssigneeChange;

  use super::*;

  const T0: OffsetDateTime = datetime!(2026-11-09 09:00 UTC);
  const T1: OffsetDateTime = datetime!(2026-11-09 10:00 UTC);

  fn open(requester: Uuid, title: &str, body: &str, now: OffsetDateTime) -> Result<Ticket, TicketError> {
    Ticket::open(TicketNumber::new(1), requester, title, body, now)
  }

  fn open_ticket() -> Ticket {
    open(Uuid::now_v7(), "  Printer on fire  ", "Please help.", T0).unwrap()
  }

  fn move_to(status: TicketStatus) -> TicketChange {
    TicketChange {
      status: Some(status),
      ..TicketChange::default()
    }
  }

  #[test]
  fn open_trims_the_title_and_starts_open() {
    let ticket = open_ticket();

    assert_eq!(ticket.title(), "Printer on fire");
    assert_eq!(ticket.status(), TicketStatus::Open);
    assert_eq!(ticket.id().get_version_num(), 7);
  }

  #[test]
  fn open_counts_characters_not_bytes() {
    let title = "မ".repeat(MAX_TITLE_CHARS);

    assert!(open(Uuid::now_v7(), &title, "body", T0).is_ok());
  }

  #[test]
  fn open_rejects_texts_outside_the_limits() {
    let requester = Uuid::now_v7();
    let too_long_title = "x".repeat(MAX_TITLE_CHARS + 1);
    let too_long_body = "x".repeat(MAX_BODY_CHARS + 1);

    assert_eq!(open(requester, "   ", "body", T0), Err(TicketError::Title));
    assert_eq!(open(requester, &too_long_title, "body", T0), Err(TicketError::Title));
    assert_eq!(open(requester, "title", " \n", T0), Err(TicketError::Body));
    assert_eq!(open(requester, "title", &too_long_body, T0), Err(TicketError::Body));
  }

  #[test]
  fn status_moves_one_step_forward() {
    let mut ticket = open_ticket();

    for status in [TicketStatus::InProgress, TicketStatus::Resolved, TicketStatus::Closed] {
      assert_eq!(ticket.apply(move_to(status), T1), Ok(true));
    }
    assert_eq!(ticket.updated_at(), T1);
  }

  #[test]
  fn status_cannot_skip_a_step() {
    let mut ticket = open_ticket();

    let result = ticket.apply(move_to(TicketStatus::Resolved), T1);

    assert_eq!(
      result,
      Err(TicketError::Transition {
        from: TicketStatus::Open,
        to: TicketStatus::Resolved
      })
    );
  }

  #[test]
  fn closed_ticket_cannot_change() {
    let open = open_ticket();
    let mut ticket = Ticket::from_persistence(
      open.id(),
      open.number(),
      open.requester_id(),
      None,
      open.title().to_owned(),
      open.body().to_owned(),
      TicketStatus::Closed,
      T0,
      T0,
    );

    assert_eq!(ticket.apply(move_to(TicketStatus::Open), T1), Err(TicketError::Closed));
  }

  #[test]
  fn assignee_can_be_set_and_cleared() {
    let mut ticket = open_ticket();
    let agent = Uuid::now_v7();

    ticket
      .apply(
        TicketChange {
          assignee: AssigneeChange::Assign(agent),
          ..TicketChange::default()
        },
        T1,
      )
      .unwrap();
    assert_eq!(ticket.assignee_id(), Some(agent));

    ticket
      .apply(
        TicketChange {
          assignee: AssigneeChange::Unassign,
          ..TicketChange::default()
        },
        T1,
      )
      .unwrap();
    assert_eq!(ticket.assignee_id(), None);
  }

  #[test]
  fn same_values_are_not_a_change() {
    let mut ticket = open_ticket();

    let changed = ticket.apply(move_to(TicketStatus::Open), T1);

    assert_eq!(changed, Ok(false));
    assert_eq!(ticket.updated_at(), T0);
  }

  #[test]
  fn open_stores_the_trimmed_body_and_refuses_nul() {
    let requester = Uuid::now_v7();
    let padded = format!("{}   ", "x".repeat(MAX_BODY_CHARS));

    assert_eq!(open(requester, "t", &padded, T0).unwrap().body().len(), MAX_BODY_CHARS);
    assert_eq!(open(requester, "a\0b", "body", T0), Err(TicketError::Title));
    assert_eq!(open(requester, "title", "a\0b", T0), Err(TicketError::Body));
  }

  #[test]
  fn times_are_kept_to_microseconds_like_postgres() {
    let precise = T0.replace_nanosecond(123_456_789).unwrap();

    let ticket = open(Uuid::now_v7(), "title", "body", precise).unwrap();

    assert_eq!(ticket.created_at().nanosecond(), 123_456_000);
  }

  #[test]
  fn updated_at_never_moves_backwards() {
    let mut ticket = open_ticket();

    ticket
      .apply(move_to(TicketStatus::InProgress), T0 - time::Duration::minutes(5))
      .unwrap();

    assert_eq!(ticket.updated_at(), T0);
  }
}
