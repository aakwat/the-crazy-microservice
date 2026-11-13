//! `TicketService` — open, read, list and change tickets on behalf of a principal.

use domain::{Principal, Ticket, TicketChange, TicketRepository};
use time::OffsetDateTime;
use uuid::Uuid;

use super::{AppError, TicketPage};

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

  /// Customers see their own tickets; agents see all.
  pub async fn get(&self, who: &Principal, id: Uuid) -> Result<Ticket, AppError> {
    let ticket = self.repo.find(id).await?.ok_or(AppError::NotFound)?;
    if !who.is_agent() && ticket.requester_id() != who.subject() {
      return Err(AppError::NotFound);
    }
    Ok(ticket)
  }

  /// Customers list their own tickets, agents all; newest first, one page at a time.
  pub async fn list(&self, who: &Principal, before: Option<Uuid>, limit: Option<u32>) -> Result<TicketPage, AppError> {
    let requester = if who.is_agent() { None } else { Some(who.subject()) };
    let size = TicketPage::size(limit);
    let items = self.repo.list(requester, before, size).await?;
    Ok(TicketPage::new(items, size))
  }

  /// Agents only; a ticket changed by someone else meanwhile is a conflict, never a silent overwrite.
  pub async fn change(
    &self,
    who: &Principal,
    id: Uuid,
    change: TicketChange,
    now: OffsetDateTime,
    traceparent: Option<&str>,
  ) -> Result<Ticket, AppError> {
    if !who.is_agent() {
      return Err(AppError::Forbidden);
    }
    let mut ticket = self.repo.find(id).await?.ok_or(AppError::NotFound)?;
    let read_at = ticket.updated_at();
    if ticket.apply(change, now)? && !self.repo.update(&ticket, read_at, traceparent).await? {
      return Err(AppError::Conflict);
    }
    Ok(ticket)
  }
}

#[cfg(test)]
mod tests {
  use std::collections::HashMap;
  use std::sync::Mutex;

  use domain::{StoreError, TicketError, TicketNumber, TicketStatus, TicketSummary};
  use time::macros::datetime;

  use super::*;

  const T0: OffsetDateTime = datetime!(2026-11-09 09:00 UTC);
  const T1: OffsetDateTime = datetime!(2026-11-09 10:00 UTC);

  /// An in-memory repository with the same update-if-unchanged rule as the real one.
  #[derive(Default)]
  struct MemoryRepo(Mutex<HashMap<Uuid, Ticket>>);

  // The trait is async for real databases; memory needs no await.
  #[allow(clippy::unused_async_trait_impl)]
  impl TicketRepository for MemoryRepo {
    async fn next_number(&self) -> Result<TicketNumber, StoreError> {
      let stored = u64::try_from(self.0.lock().unwrap().len()).unwrap();
      Ok(TicketNumber::new(stored + 1))
    }

    async fn create(&self, ticket: &Ticket, _traceparent: Option<&str>) -> Result<(), StoreError> {
      self.0.lock().unwrap().insert(ticket.id(), ticket.clone());
      Ok(())
    }

    async fn update(
      &self,
      ticket: &Ticket,
      expected_updated_at: OffsetDateTime,
      _traceparent: Option<&str>,
    ) -> Result<bool, StoreError> {
      let mut tickets = self.0.lock().unwrap();
      match tickets.get(&ticket.id()) {
        Some(stored) if stored.updated_at() == expected_updated_at => {
          tickets.insert(ticket.id(), ticket.clone());
          Ok(true)
        }
        _ => Ok(false),
      }
    }

    async fn find(&self, id: Uuid) -> Result<Option<Ticket>, StoreError> {
      Ok(self.0.lock().unwrap().get(&id).cloned())
    }

    async fn list(
      &self,
      requester: Option<Uuid>,
      _before: Option<Uuid>,
      limit: u32,
    ) -> Result<Vec<TicketSummary>, StoreError> {
      let tickets = self.0.lock().unwrap();
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

  fn start_work() -> TicketChange {
    TicketChange {
      status: Some(TicketStatus::InProgress),
      ..TicketChange::default()
    }
  }

  #[tokio::test]
  async fn requester_and_agents_can_read_a_ticket_but_others_cannot() {
    let service = TicketService::new(MemoryRepo::default());
    let alice = customer();
    let ticket = service
      .open(&alice, "Printer on fire", "Please help.", T0, None)
      .await
      .unwrap();

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
    service.open(&alice, "Mine", "body", T0, None).await.unwrap();
    service
      .open(&customer(), "Someone else's", "body", T0, None)
      .await
      .unwrap();

    let alices = service.list(&alice, None, Some(10)).await.unwrap();
    let all = service.list(&agent(), None, Some(10)).await.unwrap();

    assert_eq!(alices.items.len(), 1);
    assert_eq!(all.items.len(), 2);
  }

  #[tokio::test]
  async fn only_agents_change_tickets() {
    let service = TicketService::new(MemoryRepo::default());
    let alice = customer();
    let ticket = service
      .open(&alice, "Printer on fire", "Please help.", T0, None)
      .await
      .unwrap();

    let by_customer = service.change(&alice, ticket.id(), start_work(), T1, None).await;
    let by_agent = service
      .change(&agent(), ticket.id(), start_work(), T1, None)
      .await
      .unwrap();

    assert!(matches!(by_customer, Err(AppError::Forbidden)));
    assert_eq!(by_agent.status(), TicketStatus::InProgress);
  }

  #[tokio::test]
  async fn invalid_text_is_reported_as_invalid() {
    let service = TicketService::new(MemoryRepo::default());

    let result = service.open(&customer(), " ", "body", T0, None).await;

    assert!(matches!(result, Err(AppError::Invalid(TicketError::Title))));
  }

  /// Every write loses the race, as if another agent saved first.
  struct LosingRepo(MemoryRepo);

  #[allow(clippy::unused_async_trait_impl, reason = "losing a race needs no await")]
  impl TicketRepository for LosingRepo {
    async fn next_number(&self) -> Result<TicketNumber, StoreError> {
      self.0.next_number().await
    }

    async fn create(&self, ticket: &Ticket, _traceparent: Option<&str>) -> Result<(), StoreError> {
      self.0.create(ticket, None).await
    }

    async fn update(
      &self,
      _ticket: &Ticket,
      _expected_updated_at: OffsetDateTime,
      _traceparent: Option<&str>,
    ) -> Result<bool, StoreError> {
      Ok(false)
    }

    async fn find(&self, id: Uuid) -> Result<Option<Ticket>, StoreError> {
      self.0.find(id).await
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
    let service = TicketService::new(LosingRepo(MemoryRepo::default()));
    let ticket = service
      .open(&customer(), "Printer on fire", "Please help.", T0, None)
      .await
      .unwrap();

    let result = service.change(&agent(), ticket.id(), start_work(), T1, None).await;

    assert!(matches!(result, Err(AppError::Conflict)));
  }

  #[tokio::test]
  async fn a_no_op_change_writes_nothing_and_succeeds() {
    let service = TicketService::new(LosingRepo(MemoryRepo::default()));
    let ticket = service
      .open(&customer(), "Printer on fire", "Please help.", T0, None)
      .await
      .unwrap();
    let same = TicketChange {
      status: Some(TicketStatus::Open),
      ..TicketChange::default()
    };

    assert!(service.change(&agent(), ticket.id(), same, T1, None).await.is_ok());
  }
}
