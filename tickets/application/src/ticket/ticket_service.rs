//! `TicketService` — every ticket use case on behalf of a principal: who may do what.

use domain::{
  AssigneeChange, Message, Principal, Ticket, TicketError, TicketEvent, TicketRepository, TicketStatus, Visibility,
};
use time::OffsetDateTime;
use uuid::Uuid;

use super::{AppError, TicketPage};

/// What a reply asks for; `internal` and `then` are for staff only.
pub struct Reply<'a> {
  pub body: &'a str,
  pub internal: bool,
  pub then: Option<TicketStatus>,
}

pub struct TicketService<R> {
  repo: R,
}

impl<R: TicketRepository> TicketService<R> {
  pub fn new(repo: R) -> Self {
    Self { repo }
  }

  pub async fn open(
    &self,
    who: &Principal,
    title: &str,
    body: &str,
    now: OffsetDateTime,
    traceparent: Option<&str>,
  ) -> Result<Ticket, AppError> {
    let number = self.repo.next_number().await?;
    let ticket = Ticket::open(number, who.subject(), title, body, now)?;
    self.repo.create(&ticket, traceparent).await?;
    Ok(ticket)
  }

  /// Someone else's ticket looks missing, so its existence never leaks.
  pub async fn get(&self, who: &Principal, id: Uuid) -> Result<Ticket, AppError> {
    self
      .repo
      .find(id)
      .await?
      .filter(|ticket| who.is_agent() || ticket.requester_id() == who.subject())
      .ok_or(AppError::NotFound)
  }

  pub async fn list(&self, who: &Principal, before: Option<Uuid>, limit: Option<u32>) -> Result<TicketPage, AppError> {
    let requester = (!who.is_agent()).then_some(who.subject());
    let size = TicketPage::size(limit);
    let items = self.repo.list(requester, before, size).await?;
    Ok(TicketPage::new(items, size))
  }

  pub async fn messages(&self, who: &Principal, id: Uuid) -> Result<Vec<Message>, AppError> {
    let ticket = self.get(who, id).await?;
    Ok(self.repo.messages(ticket.id(), who.is_agent()).await?)
  }

  pub async fn reply(
    &self,
    who: &Principal,
    id: Uuid,
    reply: Reply<'_>,
    now: OffsetDateTime,
    traceparent: Option<&str>,
  ) -> Result<Message, AppError> {
    let ticket = self.get(who, id).await?;
    let me = who.subject();

    let (_, events) = if who.is_agent() {
      // Only the assignee resolves or asks the customer for something; any agent may answer.
      allow(reply.then.is_none() || ticket.assignee_id() == Some(me))?;
      let visibility = reply.visibility();
      self
        .apply(ticket, traceparent, |t| {
          t.staff_reply(me, reply.body, visibility, reply.then, now)
        })
        .await?
    } else {
      allow(!reply.internal && reply.then.is_none())?;
      self
        .apply(ticket, traceparent, |t| t.customer_reply(me, reply.body, now))
        .await?
    };

    Ok(added_message(&events))
  }

  pub async fn confirm(
    &self,
    who: &Principal,
    id: Uuid,
    now: OffsetDateTime,
    traceparent: Option<&str>,
  ) -> Result<Ticket, AppError> {
    let ticket = self.requesters_ticket(who, id).await?;
    let (ticket, _) = self.apply(ticket, traceparent, |t| t.confirm(now)).await?;
    Ok(ticket)
  }

  pub async fn withdraw(
    &self,
    who: &Principal,
    id: Uuid,
    now: OffsetDateTime,
    traceparent: Option<&str>,
  ) -> Result<Ticket, AppError> {
    let ticket = self.requesters_ticket(who, id).await?;
    let (ticket, _) = self.apply(ticket, traceparent, |t| t.withdraw(now)).await?;
    Ok(ticket)
  }

  pub async fn assign(
    &self,
    who: &Principal,
    id: Uuid,
    change: AssigneeChange,
    now: OffsetDateTime,
    traceparent: Option<&str>,
  ) -> Result<Ticket, AppError> {
    allow(who.is_agent())?;
    let ticket = self.get(who, id).await?;
    allow(may_assign(who.subject(), ticket.assignee_id(), change))?;

    let (ticket, _) = self.apply(ticket, traceparent, |t| t.assign(change, now)).await?;
    Ok(ticket)
  }

  /// Confirm and withdraw are the customer's decisions; staff resolve instead.
  async fn requesters_ticket(&self, who: &Principal, id: Uuid) -> Result<Ticket, AppError> {
    let ticket = self.get(who, id).await?;
    allow(!who.is_agent())?;
    Ok(ticket)
  }

  /// Runs one domain action and stores it; a change made on an outdated read is refused, never overwritten.
  async fn apply(
    &self,
    mut ticket: Ticket,
    traceparent: Option<&str>,
    action: impl FnOnce(&mut Ticket) -> Result<Vec<TicketEvent>, TicketError>,
  ) -> Result<(Ticket, Vec<TicketEvent>), AppError> {
    let read_at = ticket.updated_at();
    let events = action(&mut ticket)?;

    let saved = events.is_empty() || self.repo.save(&ticket, read_at, &events, traceparent).await?;
    saved.then_some((ticket, events)).ok_or(AppError::Conflict)
  }
}

impl Reply<'_> {
  fn visibility(&self) -> Visibility {
    if self.internal {
      Visibility::Internal
    } else {
      Visibility::Public
    }
  }
}

/// Each role rule reads as one line: `allow(rule)?`.
fn allow(rule: bool) -> Result<(), AppError> {
  rule.then_some(()).ok_or(AppError::Forbidden)
}

/// An agent takes a free ticket or lets go of their own; reassigning a colleague's is for an Admin.
fn may_assign(me: Uuid, current: Option<Uuid>, change: AssigneeChange) -> bool {
  let free_or_mine = current.is_none_or(|assignee| assignee == me);
  match change {
    AssigneeChange::Assign(to) => to == me && free_or_mine,
    AssigneeChange::Unassign => free_or_mine,
  }
}

#[allow(
  clippy::expect_used,
  reason = "staff_reply and customer_reply always add exactly one message"
)]
fn added_message(events: &[TicketEvent]) -> Message {
  events
    .iter()
    .find_map(|event| match event {
      TicketEvent::MessageAdded(message) => Some(message.clone()),
      TicketEvent::StatusChanged | TicketEvent::Assigned => None,
    })
    .expect("a reply adds a message")
}

#[cfg(test)]
mod tests {
  use std::collections::HashMap;
  use std::sync::Mutex;

  use domain::{StoreError, TicketError, TicketNumber, TicketSummary};
  use time::macros::datetime;

  use super::*;

  const OPENED: OffsetDateTime = datetime!(2026-11-09 09:00 UTC);
  const LATER: OffsetDateTime = datetime!(2026-11-09 10:00 UTC);

  /// In memory, with the same update-if-unchanged rule as the real repository.
  #[derive(Default)]
  struct MemoryRepo {
    tickets: Mutex<HashMap<Uuid, Ticket>>,
    messages: Mutex<Vec<Message>>,
  }

  #[allow(clippy::unused_async_trait_impl, reason = "memory needs no await")]
  impl TicketRepository for MemoryRepo {
    async fn next_number(&self) -> Result<TicketNumber, StoreError> {
      let stored = u64::try_from(self.tickets.lock().unwrap().len()).unwrap();
      Ok(TicketNumber::new(stored + 1))
    }

    async fn create(&self, ticket: &Ticket, _traceparent: Option<&str>) -> Result<(), StoreError> {
      self.tickets.lock().unwrap().insert(ticket.id(), ticket.clone());
      Ok(())
    }

    async fn save(
      &self,
      ticket: &Ticket,
      expected_updated_at: OffsetDateTime,
      events: &[TicketEvent],
      _traceparent: Option<&str>,
    ) -> Result<bool, StoreError> {
      let mut tickets = self.tickets.lock().unwrap();
      if tickets.get(&ticket.id()).map(Ticket::updated_at) != Some(expected_updated_at) {
        return Ok(false);
      }
      tickets.insert(ticket.id(), ticket.clone());
      for event in events {
        if let TicketEvent::MessageAdded(message) = event {
          self.messages.lock().unwrap().push(message.clone());
        }
      }
      Ok(true)
    }

    async fn find(&self, id: Uuid) -> Result<Option<Ticket>, StoreError> {
      Ok(self.tickets.lock().unwrap().get(&id).cloned())
    }

    async fn messages(&self, ticket_id: Uuid, include_internal: bool) -> Result<Vec<Message>, StoreError> {
      let messages = self.messages.lock().unwrap();
      let visible = messages
        .iter()
        .filter(|m| m.ticket_id() == ticket_id && (include_internal || m.visibility() == Visibility::Public));
      Ok(visible.cloned().collect())
    }

    async fn list(
      &self,
      requester: Option<Uuid>,
      _before: Option<Uuid>,
      limit: u32,
    ) -> Result<Vec<TicketSummary>, StoreError> {
      let tickets = self.tickets.lock().unwrap();
      let visible = tickets
        .values()
        .filter(|t| requester.is_none_or(|r| t.requester_id() == r));
      Ok(visible.take(limit as usize).map(summary).collect())
    }
  }

  fn summary(t: &Ticket) -> TicketSummary {
    TicketSummary {
      id: t.id(),
      number: t.number(),
      requester_id: t.requester_id(),
      assignee_id: t.assignee_id(),
      title: t.title().to_owned(),
      status: t.status(),
      updated_at: t.updated_at(),
    }
  }

  fn customer() -> Principal {
    Principal::new(Uuid::now_v7(), vec!["customer".to_owned()])
  }

  fn agent() -> Principal {
    Principal::new(Uuid::now_v7(), vec!["agent".to_owned()])
  }

  fn say(body: &str) -> Reply<'_> {
    Reply {
      body,
      internal: false,
      then: None,
    }
  }

  fn note(body: &str) -> Reply<'_> {
    Reply {
      body,
      internal: true,
      then: None,
    }
  }

  async fn opened<R: TicketRepository>(service: &TicketService<R>, requester: &Principal) -> Ticket {
    service
      .open(requester, "Printer on fire", "Please help.", OPENED, None)
      .await
      .unwrap()
  }

  #[tokio::test]
  async fn requester_and_agents_can_read_a_ticket_but_others_cannot() {
    let service = TicketService::new(MemoryRepo::default());
    let alice = customer();
    let ticket = opened(&service, &alice).await;

    assert!(service.get(&alice, ticket.id()).await.is_ok());
    assert!(service.get(&agent(), ticket.id()).await.is_ok());
    assert!(matches!(
      service.get(&customer(), ticket.id()).await,
      Err(AppError::NotFound)
    ));
  }

  #[tokio::test]
  async fn customers_list_only_their_own_tickets() {
    let service = TicketService::new(MemoryRepo::default());
    let alice = customer();
    opened(&service, &alice).await;
    opened(&service, &customer()).await;

    let alices = service.list(&alice, None, Some(10)).await.unwrap();
    let all = service.list(&agent(), None, Some(10)).await.unwrap();

    assert_eq!(alices.items.len(), 1);
    assert_eq!(all.items.len(), 2);
  }

  #[tokio::test]
  async fn customers_never_see_internal_notes() {
    let service = TicketService::new(MemoryRepo::default());
    let (alice, bob) = (customer(), agent());
    let ticket = opened(&service, &alice).await;
    service
      .reply(&bob, ticket.id(), note("Probably the fuser."), LATER, None)
      .await
      .unwrap();
    service
      .reply(&bob, ticket.id(), say("We are on it."), LATER, None)
      .await
      .unwrap();

    let seen_by_alice = service.messages(&alice, ticket.id()).await.unwrap();
    let seen_by_bob = service.messages(&bob, ticket.id()).await.unwrap();

    assert_eq!(seen_by_alice.len(), 1);
    assert_eq!(seen_by_alice[0].body(), "We are on it.");
    assert_eq!(seen_by_bob.len(), 2);
  }

  #[tokio::test]
  async fn customers_cannot_write_notes_or_set_a_status() {
    let service = TicketService::new(MemoryRepo::default());
    let alice = customer();
    let ticket = opened(&service, &alice).await;
    let resolve = Reply {
      then: Some(TicketStatus::Resolved),
      ..say("Fixed it myself.")
    };

    let as_note = service.reply(&alice, ticket.id(), note("secret"), LATER, None).await;
    let as_resolve = service.reply(&alice, ticket.id(), resolve, LATER, None).await;

    assert!(matches!(as_note, Err(AppError::Forbidden)));
    assert!(matches!(as_resolve, Err(AppError::Forbidden)));
  }

  #[tokio::test]
  async fn nobody_replies_on_a_ticket_they_cannot_see() {
    let service = TicketService::new(MemoryRepo::default());
    let ticket = opened(&service, &customer()).await;

    let result = service.reply(&customer(), ticket.id(), say("Hi"), LATER, None).await;

    assert!(matches!(result, Err(AppError::NotFound)));
  }

  #[tokio::test]
  async fn the_requester_confirms_and_withdraws_but_staff_cannot() {
    let service = TicketService::new(MemoryRepo::default());
    let (alice, bob) = (customer(), agent());
    let ticket = opened(&service, &alice).await;

    let by_agent = service.withdraw(&bob, ticket.id(), LATER, None).await;
    let by_requester = service.withdraw(&alice, ticket.id(), LATER, None).await.unwrap();

    assert!(matches!(by_agent, Err(AppError::Forbidden)));
    assert_eq!(by_requester.status(), TicketStatus::Closed);
  }

  #[tokio::test]
  async fn a_staff_resolution_then_a_customer_confirmation_closes_the_ticket() {
    let service = TicketService::new(MemoryRepo::default());
    let (alice, bob) = (customer(), agent());
    let ticket = opened(&service, &alice).await;
    let resolve = Reply {
      then: Some(TicketStatus::Resolved),
      ..say("Replaced the fuser.")
    };
    take(&service, &bob, ticket.id()).await;
    service.reply(&bob, ticket.id(), resolve, LATER, None).await.unwrap();

    let closed = service.confirm(&alice, ticket.id(), LATER, None).await.unwrap();

    assert_eq!(closed.status(), TicketStatus::Closed);
  }

  async fn take<R: TicketRepository>(service: &TicketService<R>, who: &Principal, id: Uuid) {
    let mine = AssigneeChange::Assign(who.subject());
    service.assign(who, id, mine, LATER, None).await.unwrap();
  }

  #[tokio::test]
  async fn only_agents_assign() {
    let service = TicketService::new(MemoryRepo::default());
    let (alice, bob) = (customer(), agent());
    let ticket = opened(&service, &alice).await;
    let to_bob = AssigneeChange::Assign(bob.subject());

    let by_customer = service.assign(&alice, ticket.id(), to_bob, LATER, None).await;
    let by_bob = service.assign(&bob, ticket.id(), to_bob, LATER, None).await.unwrap();

    assert!(matches!(by_customer, Err(AppError::Forbidden)));
    assert_eq!(by_bob.assignee_id(), Some(bob.subject()));
  }

  #[tokio::test]
  async fn an_agent_takes_only_for_themselves_and_never_a_colleagues_ticket() {
    let service = TicketService::new(MemoryRepo::default());
    let (alice, bob, carol) = (customer(), agent(), agent());
    let ticket = opened(&service, &alice).await;

    let to_alice = AssigneeChange::Assign(alice.subject());
    let pushed_on_customer = service.assign(&bob, ticket.id(), to_alice, LATER, None).await;
    take(&service, &bob, ticket.id()).await;
    let carol_takes = AssigneeChange::Assign(carol.subject());
    let stolen = service.assign(&carol, ticket.id(), carol_takes, LATER, None).await;
    let dropped = service
      .assign(&carol, ticket.id(), AssigneeChange::Unassign, LATER, None)
      .await;

    assert!(matches!(pushed_on_customer, Err(AppError::Forbidden)));
    assert!(matches!(stolen, Err(AppError::Forbidden)));
    assert!(matches!(dropped, Err(AppError::Forbidden)));
  }

  #[tokio::test]
  async fn only_the_assignee_resolves_but_any_agent_answers() {
    let service = TicketService::new(MemoryRepo::default());
    let (alice, bob, carol) = (customer(), agent(), agent());
    let ticket = opened(&service, &alice).await;
    take(&service, &bob, ticket.id()).await;
    let resolve = Reply {
      then: Some(TicketStatus::Resolved),
      ..say("Fixed.")
    };

    let by_carol = service.reply(&carol, ticket.id(), resolve, LATER, None).await;
    let answer = service
      .reply(&carol, ticket.id(), say("Bob is on it."), LATER, None)
      .await;

    assert!(matches!(by_carol, Err(AppError::Forbidden)));
    assert!(answer.is_ok());
  }

  #[tokio::test]
  async fn a_rule_violation_is_reported_as_invalid() {
    let service = TicketService::new(MemoryRepo::default());
    let alice = customer();
    let ticket = opened(&service, &alice).await;

    let result = service.confirm(&alice, ticket.id(), LATER, None).await;

    assert!(matches!(result, Err(AppError::Invalid(TicketError::Transition { .. }))));
  }

  /// Every write loses the race, as if another agent saved first.
  #[derive(Default)]
  struct LosingRepo(MemoryRepo);

  #[allow(clippy::unused_async_trait_impl, reason = "losing a race needs no await")]
  impl TicketRepository for LosingRepo {
    async fn next_number(&self) -> Result<TicketNumber, StoreError> {
      self.0.next_number().await
    }

    async fn create(&self, ticket: &Ticket, traceparent: Option<&str>) -> Result<(), StoreError> {
      self.0.create(ticket, traceparent).await
    }

    async fn save(
      &self,
      _ticket: &Ticket,
      _expected_updated_at: OffsetDateTime,
      _events: &[TicketEvent],
      _traceparent: Option<&str>,
    ) -> Result<bool, StoreError> {
      Ok(false)
    }

    async fn find(&self, id: Uuid) -> Result<Option<Ticket>, StoreError> {
      self.0.find(id).await
    }

    async fn messages(&self, ticket_id: Uuid, include_internal: bool) -> Result<Vec<Message>, StoreError> {
      self.0.messages(ticket_id, include_internal).await
    }

    async fn list(
      &self,
      requester: Option<Uuid>,
      before: Option<Uuid>,
      limit: u32,
    ) -> Result<Vec<TicketSummary>, StoreError> {
      self.0.list(requester, before, limit).await
    }
  }

  #[tokio::test]
  async fn a_change_that_loses_a_concurrent_write_is_a_conflict() {
    let service = TicketService::new(LosingRepo::default());
    let ticket = opened(&service, &customer()).await;

    let result = service.reply(&agent(), ticket.id(), say("On it."), LATER, None).await;

    assert!(matches!(result, Err(AppError::Conflict)));
  }

  #[tokio::test]
  async fn a_change_with_no_effect_writes_nothing_and_succeeds() {
    let service = TicketService::new(LosingRepo::default());
    let ticket = opened(&service, &customer()).await;

    let result = service
      .assign(&agent(), ticket.id(), AssigneeChange::Unassign, LATER, None)
      .await;

    assert!(result.is_ok());
  }
}
