//! `Ticket` — a support ticket and its lifecycle (docs/business-rules.md 13.2).

use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use super::stored::{fits, stored_precision};
use super::{
  AssigneeChange, Author, CloseReason, Message, TicketError, TicketEvent, TicketNumber, TicketStatus, Visibility,
};

/// Longest title in characters; the database enforces the same limit.
pub const MAX_TITLE_CHARS: usize = 200;
/// Longest body in characters; the database enforces the same limit.
pub const MAX_BODY_CHARS: usize = 10_000;

/// Changed only through its actions, so every change arrives with the events the outbox needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ticket {
  id: Uuid,
  number: TicketNumber,
  requester_id: Uuid,
  assignee_id: Option<Uuid>,
  title: String,
  body: String,
  status: TicketStatus,
  close_reason: Option<CloseReason>,
  reopen_count: u32,
  created_at: OffsetDateTime,
  updated_at: OffsetDateTime,
}

impl Ticket {
  /// The limits count the trimmed text, so padding never pushes a ticket over them.
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
      status: TicketStatus::New,
      close_reason: None,
      reopen_count: 0,
      created_at: now,
      updated_at: now,
    })
  }

  pub fn customer_reply(
    &mut self,
    author_id: Uuid,
    body: &str,
    now: OffsetDateTime,
  ) -> Result<Vec<TicketEvent>, TicketError> {
    self.ensure_not_closed()?;
    let message = Message::write(self.id, author_id, Author::Customer, Visibility::Public, body, now)?;
    let mut events = vec![TicketEvent::MessageAdded(message)];
    match self.status {
      TicketStatus::Pending => self.move_to(TicketStatus::Open, &mut events),
      TicketStatus::Resolved => {
        self.reopen_count += 1;
        self.move_to(TicketStatus::Open, &mut events);
      }
      TicketStatus::New | TicketStatus::Open | TicketStatus::Closed => {}
    }
    self.touch(now);
    Ok(events)
  }

  pub fn staff_reply(
    &mut self,
    author_id: Uuid,
    body: &str,
    visibility: Visibility,
    then: Option<TicketStatus>,
    now: OffsetDateTime,
  ) -> Result<Vec<TicketEvent>, TicketError> {
    self.ensure_not_closed()?;
    match visibility {
      Visibility::Public => self.public_reply(author_id, body, then, now),
      Visibility::Internal => self.note(author_id, body, then, now),
    }
  }

  pub fn confirm(&mut self, now: OffsetDateTime) -> Result<Vec<TicketEvent>, TicketError> {
    self.close(CloseReason::Confirmed, now)
  }

  pub fn withdraw(&mut self, now: OffsetDateTime) -> Result<Vec<TicketEvent>, TicketError> {
    self.close(CloseReason::Withdrawn, now)
  }

  pub fn assign(&mut self, change: AssigneeChange, now: OffsetDateTime) -> Result<Vec<TicketEvent>, TicketError> {
    self.ensure_not_closed()?;
    let assignee_id = change.assignee();
    if assignee_id == self.assignee_id {
      return Ok(Vec::new());
    }
    self.assignee_id = assignee_id;
    self.touch(now);
    Ok(vec![TicketEvent::Assigned])
  }

  fn public_reply(
    &mut self,
    author_id: Uuid,
    body: &str,
    then: Option<TicketStatus>,
    now: OffsetDateTime,
  ) -> Result<Vec<TicketEvent>, TicketError> {
    let target = self.after_staff_reply(then)?;
    let message = Message::write(self.id, author_id, Author::Staff, Visibility::Public, body, now)?;

    let mut events = vec![TicketEvent::MessageAdded(message)];
    self.move_to(target, &mut events);
    self.touch(now);
    Ok(events)
  }

  /// A note leaves the ticket untouched, so customers cannot see one was written.
  fn note(
    &self,
    author_id: Uuid,
    body: &str,
    then: Option<TicketStatus>,
    now: OffsetDateTime,
  ) -> Result<Vec<TicketEvent>, TicketError> {
    if then.is_some() {
      return Err(TicketError::NoteWithStatus);
    }
    let message = Message::write(self.id, author_id, Author::Staff, Visibility::Internal, body, now)?;
    Ok(vec![TicketEvent::MessageAdded(message)])
  }

  /// The status a public staff reply leads to; the current one when it stays.
  fn after_staff_reply(&self, then: Option<TicketStatus>) -> Result<TicketStatus, TicketError> {
    use TicketStatus::{New, Open, Pending, Resolved};
    match (self.status, then) {
      (New, None) => Ok(Open),
      (current, None) => Ok(current),
      (New | Open | Pending, Some(to @ Pending)) | (New | Open | Pending | Resolved, Some(to @ Resolved)) => Ok(to),
      (_, Some(to)) => Err(self.cannot_move_to(to)),
    }
  }

  fn close(&mut self, reason: CloseReason, now: OffsetDateTime) -> Result<Vec<TicketEvent>, TicketError> {
    self.ensure_can_close(reason)?;

    let mut events = Vec::new();
    self.close_reason = Some(reason);
    self.move_to(TicketStatus::Closed, &mut events);
    self.touch(now);
    Ok(events)
  }

  /// Confirming needs a resolution; withdrawing is only for a ticket not yet resolved.
  fn ensure_can_close(&self, reason: CloseReason) -> Result<(), TicketError> {
    use TicketStatus::{Closed, New, Open, Pending, Resolved};
    match (self.status, reason) {
      (Closed, _) => Err(TicketError::Closed),
      (Resolved, CloseReason::Confirmed) | (New | Open | Pending, CloseReason::Withdrawn) => Ok(()),
      _ => Err(self.cannot_move_to(Closed)),
    }
  }

  fn move_to(&mut self, status: TicketStatus, events: &mut Vec<TicketEvent>) {
    if status != self.status {
      self.status = status;
      events.push(TicketEvent::StatusChanged);
    }
  }

  fn ensure_not_closed(&self) -> Result<(), TicketError> {
    if self.status == TicketStatus::Closed {
      return Err(TicketError::Closed);
    }
    Ok(())
  }

  fn cannot_move_to(&self, to: TicketStatus) -> TicketError {
    TicketError::Transition { from: self.status, to }
  }

  /// Always moves forward, even on a replica whose clock runs behind: it is the stale-write check's version.
  fn touch(&mut self, now: OffsetDateTime) {
    self.updated_at = stored_precision(now).max(self.updated_at + Duration::microseconds(1));
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
    close_reason: Option<CloseReason>,
    reopen_count: u32,
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
      close_reason,
      reopen_count,
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

  pub fn close_reason(&self) -> Option<CloseReason> {
    self.close_reason
  }

  pub fn reopen_count(&self) -> u32 {
    self.reopen_count
  }

  pub fn created_at(&self) -> OffsetDateTime {
    self.created_at
  }

  pub fn updated_at(&self) -> OffsetDateTime {
    self.updated_at
  }
}

#[cfg(test)]
mod tests {
  use time::macros::datetime;

  use super::*;
  use TicketStatus::{Closed, New, Open, Pending, Resolved};

  const OPENED: OffsetDateTime = datetime!(2026-11-09 09:00 UTC);
  const LATER: OffsetDateTime = datetime!(2026-11-09 10:00 UTC);

  fn open(requester: Uuid, title: &str, body: &str, now: OffsetDateTime) -> Result<Ticket, TicketError> {
    Ticket::open(TicketNumber::new(1), requester, title, body, now)
  }

  fn new_ticket() -> Ticket {
    open(Uuid::now_v7(), "  Printer on fire  ", "Please help.", OPENED).unwrap()
  }

  /// A ticket already in `status`, as if read back from the database.
  fn ticket_in(status: TicketStatus) -> Ticket {
    let t = new_ticket();
    let reason = (status == Closed).then_some(CloseReason::Withdrawn);
    Ticket::from_persistence(
      t.id(),
      t.number(),
      t.requester_id(),
      None,
      t.title().to_owned(),
      t.body().to_owned(),
      status,
      reason,
      0,
      OPENED,
      OPENED,
    )
  }

  fn staff(
    ticket: &mut Ticket,
    visibility: Visibility,
    then: Option<TicketStatus>,
  ) -> Result<Vec<TicketEvent>, TicketError> {
    ticket.staff_reply(Uuid::now_v7(), "On it.", visibility, then, LATER)
  }

  fn status_changed(events: &[TicketEvent]) -> bool {
    events.contains(&TicketEvent::StatusChanged)
  }

  #[test]
  fn open_trims_the_title_and_starts_new() {
    let ticket = new_ticket();

    assert_eq!(ticket.title(), "Printer on fire");
    assert_eq!(ticket.status(), New);
    assert_eq!(ticket.id().get_version_num(), 7);
  }

  #[test]
  fn open_counts_characters_not_bytes() {
    let title = "မ".repeat(MAX_TITLE_CHARS);

    assert!(open(Uuid::now_v7(), &title, "body", OPENED).is_ok());
  }

  #[test]
  fn open_rejects_texts_outside_the_limits() {
    let requester = Uuid::now_v7();
    let too_long_title = "x".repeat(MAX_TITLE_CHARS + 1);
    let too_long_body = "x".repeat(MAX_BODY_CHARS + 1);

    assert_eq!(open(requester, "   ", "body", OPENED), Err(TicketError::Title));
    assert_eq!(
      open(requester, &too_long_title, "body", OPENED),
      Err(TicketError::Title)
    );
    assert_eq!(open(requester, "title", " \n", OPENED), Err(TicketError::Body));
    assert_eq!(open(requester, "title", &too_long_body, OPENED), Err(TicketError::Body));
    assert_eq!(open(requester, "a\0b", "body", OPENED), Err(TicketError::Title));
  }

  #[test]
  fn first_public_staff_reply_starts_work() {
    let mut ticket = new_ticket();

    let events = staff(&mut ticket, Visibility::Public, None).unwrap();

    assert_eq!(ticket.status(), Open);
    assert!(status_changed(&events));
    assert_eq!(ticket.updated_at(), LATER);
  }

  #[test]
  fn an_internal_note_never_changes_the_status() {
    let mut ticket = new_ticket();

    let events = staff(&mut ticket, Visibility::Internal, None).unwrap();

    assert_eq!(ticket.status(), New);
    assert!(!status_changed(&events));
    assert_eq!(ticket.updated_at(), OPENED, "a note must not show on the ticket");
    assert_eq!(
      staff(&mut ticket, Visibility::Internal, Some(Pending)),
      Err(TicketError::NoteWithStatus)
    );
  }

  #[test]
  fn staff_ask_or_resolve_only_where_the_table_allows() {
    let allowed = [
      (New, Pending),
      (Open, Pending),
      (New, Resolved),
      (Open, Resolved),
      (Pending, Resolved),
    ];
    for (from, to) in allowed {
      let mut ticket = ticket_in(from);
      staff(&mut ticket, Visibility::Public, Some(to)).unwrap();
      assert_eq!(ticket.status(), to, "{from:?} -> {to:?}");
    }

    let forbidden = [(Resolved, Pending), (Open, New), (Open, Open), (Open, Closed)];
    for (from, to) in forbidden {
      let mut ticket = ticket_in(from);
      assert_eq!(
        staff(&mut ticket, Visibility::Public, Some(to)),
        Err(TicketError::Transition { from, to }),
        "{from:?} -> {to:?}"
      );
    }
  }

  #[test]
  fn customer_reply_moves_pending_to_open() {
    let mut ticket = ticket_in(Pending);

    let events = ticket.customer_reply(Uuid::now_v7(), "Here it is.", LATER).unwrap();

    assert_eq!(ticket.status(), Open);
    assert!(status_changed(&events));
    assert_eq!(ticket.reopen_count(), 0);
  }

  #[test]
  fn customer_reply_reopens_a_resolved_ticket_and_counts_it() {
    let mut ticket = ticket_in(Resolved);

    ticket.customer_reply(Uuid::now_v7(), "Still broken.", LATER).unwrap();

    assert_eq!(ticket.status(), Open);
    assert_eq!(ticket.reopen_count(), 1);
  }

  #[test]
  fn customer_reply_on_new_or_open_keeps_the_status() {
    for status in [New, Open] {
      let mut ticket = ticket_in(status);

      let events = ticket.customer_reply(Uuid::now_v7(), "More details.", LATER).unwrap();

      assert_eq!(ticket.status(), status);
      assert!(!status_changed(&events));
    }
  }

  #[test]
  fn confirm_closes_only_a_resolved_ticket() {
    let mut resolved = ticket_in(Resolved);
    resolved.confirm(LATER).unwrap();
    assert_eq!(resolved.status(), Closed);
    assert_eq!(resolved.close_reason(), Some(CloseReason::Confirmed));

    let mut open = ticket_in(Open);
    assert_eq!(
      open.confirm(LATER),
      Err(TicketError::Transition { from: Open, to: Closed })
    );
  }

  #[test]
  fn withdraw_closes_anything_not_yet_resolved() {
    for status in [New, Open, Pending] {
      let mut ticket = ticket_in(status);
      ticket.withdraw(LATER).unwrap();
      assert_eq!(ticket.close_reason(), Some(CloseReason::Withdrawn));
    }

    let mut resolved = ticket_in(Resolved);
    assert_eq!(
      resolved.withdraw(LATER),
      Err(TicketError::Transition {
        from: Resolved,
        to: Closed
      })
    );
  }

  #[test]
  fn closed_is_final() {
    let mut ticket = ticket_in(Closed);

    assert_eq!(
      ticket.customer_reply(Uuid::now_v7(), "Hello?", LATER),
      Err(TicketError::Closed)
    );
    assert_eq!(staff(&mut ticket, Visibility::Internal, None), Err(TicketError::Closed));
    assert_eq!(ticket.confirm(LATER), Err(TicketError::Closed));
    assert_eq!(ticket.withdraw(LATER), Err(TicketError::Closed));
    assert_eq!(
      ticket.assign(AssigneeChange::Assign(Uuid::now_v7()), LATER),
      Err(TicketError::Closed)
    );
  }

  #[test]
  fn assignee_can_be_set_and_cleared_and_same_is_no_event() {
    let mut ticket = new_ticket();
    let agent = Uuid::now_v7();

    assert_eq!(
      ticket.assign(AssigneeChange::Assign(agent), LATER),
      Ok(vec![TicketEvent::Assigned])
    );
    assert_eq!(ticket.assign(AssigneeChange::Assign(agent), LATER), Ok(Vec::new()));
    assert_eq!(
      ticket.assign(AssigneeChange::Unassign, LATER),
      Ok(vec![TicketEvent::Assigned])
    );
    assert_eq!(ticket.assignee_id(), None);
  }

  #[test]
  fn a_message_must_fit_its_limit() {
    let mut ticket = new_ticket();

    assert_eq!(
      ticket.customer_reply(Uuid::now_v7(), "  ", LATER),
      Err(TicketError::Message)
    );
    assert_eq!(ticket.status(), New);
  }

  #[test]
  fn times_are_kept_to_microseconds_and_always_move_forward() {
    let precise = OPENED.replace_nanosecond(123_456_789).unwrap();
    let mut ticket = open(Uuid::now_v7(), "title", "body", precise).unwrap();
    assert_eq!(ticket.created_at().nanosecond(), 123_456_000);

    ticket
      .assign(AssigneeChange::Assign(Uuid::now_v7()), OPENED - Duration::minutes(5))
      .unwrap();

    assert_eq!(ticket.updated_at(), ticket.created_at() + Duration::microseconds(1));
  }
}
