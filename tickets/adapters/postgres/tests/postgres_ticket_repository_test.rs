//! The ticket repository and its outbox writes against real Postgres.
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::TestDb;
use domain::{Ticket, TicketChange, TicketRepository, TicketStatus};
use time::OffsetDateTime;
use time::macros::datetime;
use uuid::Uuid;

const T0: OffsetDateTime = datetime!(2026-11-09 09:00 UTC);
const T1: OffsetDateTime = datetime!(2026-11-09 10:00 UTC);
const TRACE: &str = "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01";

fn ticket_for(requester: Uuid) -> Ticket {
  Ticket::open(common::number(), requester, "Printer on fire", "Please help.", T0).unwrap()
}

#[tokio::test]
async fn create_stores_the_ticket_and_a_thin_created_event() {
  let db = TestDb::start().await;
  let ticket = ticket_for(Uuid::now_v7());

  db.repo().create(&ticket, Some(TRACE)).await.unwrap();

  assert_eq!(db.repo().find(ticket.id()).await.unwrap(), Some(ticket.clone()));
  let events = db.events().await;
  assert_eq!(events.len(), 1);
  assert_eq!(events[0].0, "ticket.created");
  assert_eq!(events[0].1["ticket_id"], ticket.id().to_string());
  assert!(events[0].1.get("title").is_none(), "events must not carry ticket text");
  let traceparent: Option<String> = sqlx::query_scalar("SELECT traceparent FROM outbox")
    .fetch_one(&db.app)
    .await
    .unwrap();
  assert_eq!(traceparent.as_deref(), Some(TRACE));
}

#[tokio::test]
async fn update_applies_once_and_refuses_a_stale_read() {
  let db = TestDb::start().await;
  let original = ticket_for(Uuid::now_v7());
  db.repo().create(&original, None).await.unwrap();
  let start_work = TicketChange {
    status: Some(TicketStatus::InProgress),
    ..TicketChange::default()
  };
  let mut changed = original.clone();
  changed.apply(start_work, T1).unwrap();

  let first = db.repo().update(&changed, original.updated_at(), None).await.unwrap();
  let stale = db.repo().update(&changed, original.updated_at(), None).await.unwrap();

  assert!(first);
  assert!(!stale);
  let types: Vec<String> = db.events().await.into_iter().map(|(t, _)| t).collect();
  assert_eq!(types, ["ticket.created", "ticket.changed"]);
}

#[tokio::test]
async fn list_filters_by_requester_and_pages_newest_first() {
  let db = TestDb::start().await;
  let alice = Uuid::now_v7();
  let tickets: Vec<Ticket> = (0..3).map(|_| ticket_for(alice)).collect();
  for ticket in &tickets {
    db.repo().create(ticket, None).await.unwrap();
  }
  db.repo().create(&ticket_for(Uuid::now_v7()), None).await.unwrap();

  let first_page = db.repo().list(Some(alice), None, 2).await.unwrap();
  let second_page = db.repo().list(Some(alice), Some(first_page[1].id), 2).await.unwrap();
  let everyone = db.repo().list(None, None, 10).await.unwrap();

  assert_eq!(
    first_page.iter().map(|s| s.id).collect::<Vec<_>>(),
    [tickets[2].id(), tickets[1].id()]
  );
  assert_eq!(second_page.iter().map(|s| s.id).collect::<Vec<_>>(), [tickets[0].id()]);
  assert_eq!(everyone.len(), 4);
}

#[tokio::test]
async fn the_app_role_cannot_change_the_schema_or_store_a_bad_status() {
  let db = TestDb::start().await;
  let ticket = ticket_for(Uuid::now_v7());
  db.repo().create(&ticket, None).await.unwrap();

  let ddl = sqlx::query("CREATE TABLE intruder (id INT)").execute(&db.app).await;
  let bad_status = sqlx::query("UPDATE ticket SET status = 'reopened' WHERE id = $1")
    .bind(ticket.id())
    .execute(&db.app)
    .await;

  assert!(ddl.is_err(), "tickets_app must be DML only");
  assert!(bad_status.is_err(), "the CHECK constraint must refuse unknown statuses");
}

#[tokio::test]
async fn a_ticket_opened_with_nanoseconds_can_be_updated_with_its_own_timestamp() {
  let db = TestDb::start().await;
  let precise = T0.replace_nanosecond(123_456_789).unwrap();
  let ticket = Ticket::open(
    common::number(),
    Uuid::now_v7(),
    "Printer on fire",
    "Please help.",
    precise,
  )
  .unwrap();
  db.repo().create(&ticket, None).await.unwrap();
  let mut changed = ticket.clone();
  changed
    .apply(
      TicketChange {
        status: Some(TicketStatus::InProgress),
        ..TicketChange::default()
      },
      T1,
    )
    .unwrap();

  assert!(db.repo().update(&changed, ticket.updated_at(), None).await.unwrap());
}

#[tokio::test]
async fn numbers_grow_and_a_stored_ticket_keeps_its_number() {
  let db = TestDb::start().await;
  let first = db.repo().next_number().await.unwrap();
  let second = db.repo().next_number().await.unwrap();
  let ticket = Ticket::open(second, Uuid::now_v7(), "Printer on fire", "Please help.", T0).unwrap();

  db.repo().create(&ticket, None).await.unwrap();

  assert!(second.value() > first.value());
  assert_eq!(db.repo().find(ticket.id()).await.unwrap().unwrap().number(), second);
  assert_eq!(db.events().await[0].1["reference"], second.to_string());
}
