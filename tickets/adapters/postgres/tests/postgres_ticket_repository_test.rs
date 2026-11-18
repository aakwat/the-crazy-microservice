//! The ticket repository and its outbox writes against real Postgres.
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::TestDb;
use domain::{
  AssigneeChange, Author, CloseReason, Message, Ticket, TicketEvent, TicketRepository, TicketStatus, Visibility,
};
use time::OffsetDateTime;
use time::macros::datetime;
use uuid::Uuid;

const OPENED: OffsetDateTime = datetime!(2026-11-09 09:00 UTC);
const LATER: OffsetDateTime = datetime!(2026-11-09 10:00 UTC);
const TRACE: &str = "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01";

fn ticket_for(requester: Uuid) -> Ticket {
  Ticket::open(common::number(), requester, "Printer on fire", "Please help.", OPENED).unwrap()
}

async fn stored(db: &TestDb) -> Ticket {
  let ticket = ticket_for(Uuid::now_v7());
  db.repo().create(&ticket, None).await.unwrap();
  ticket
}

async fn save(db: &TestDb, ticket: &Ticket, read_at: OffsetDateTime, events: &[TicketEvent]) -> bool {
  db.repo().save(ticket, read_at, events, None).await.unwrap()
}

fn staff(ticket: &mut Ticket, body: &str, visibility: Visibility, then: Option<TicketStatus>) -> Vec<TicketEvent> {
  ticket
    .staff_reply(Uuid::now_v7(), body, visibility, then, LATER)
    .unwrap()
}

fn customer(ticket: &mut Ticket, body: &str) -> Vec<TicketEvent> {
  ticket.customer_reply(ticket.requester_id(), body, LATER).unwrap()
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
  assert_eq!(events[0].1["status"], "new");
  assert!(events[0].1["message"].is_null());
  assert!(events[0].1.get("title").is_none(), "events must not carry ticket text");
  let traceparent: Option<String> = sqlx::query_scalar("SELECT traceparent FROM outbox")
    .fetch_one(&db.app)
    .await
    .unwrap();
  assert_eq!(traceparent.as_deref(), Some(TRACE));
}

#[tokio::test]
async fn save_applies_once_and_refuses_a_stale_read() {
  let db = TestDb::start().await;
  let original = stored(&db).await;
  let mut changed = original.clone();
  let events = changed.assign(AssigneeChange::Assign(Uuid::now_v7()), LATER).unwrap();

  let first = save(&db, &changed, original.updated_at(), &events).await;
  let stale = save(&db, &changed, original.updated_at(), &events).await;

  assert!(first);
  assert!(!stale);
  let types: Vec<String> = db.events().await.into_iter().map(|(t, _)| t).collect();
  assert_eq!(types, ["ticket.created", "ticket.assigned"]);
}

#[tokio::test]
async fn a_staff_reply_stores_the_message_and_its_events_in_order_without_text() {
  let db = TestDb::start().await;
  let original = stored(&db).await;
  let mut ticket = original.clone();
  let events = staff(&mut ticket, "Try turning it off.", Visibility::Public, None);

  save(&db, &ticket, original.updated_at(), &events).await;

  let outbox = db.events().await;
  let types: Vec<&str> = outbox.iter().map(|(t, _)| t.as_str()).collect();
  let expected = ["ticket.created", "ticket.message_added", "ticket.status_changed"];
  assert_eq!(types, expected);
  let message = &outbox[1].1["message"];
  assert_eq!(outbox[1].1["status"], "open");
  assert_eq!(message["author"], "staff");
  assert_eq!(message["visibility"], "public");
  assert!(message.get("body").is_none(), "events must not carry message text");
  assert!(outbox[2].1["message"].is_null());
  let messages = db.repo().messages(ticket.id(), false).await.unwrap();
  assert_eq!(messages.len(), 1);
  assert_eq!(messages[0].author(), Author::Staff);
  assert_eq!(messages[0].body(), "Try turning it off.");
}

#[tokio::test]
async fn internal_notes_are_left_out_unless_asked_for() {
  let db = TestDb::start().await;
  let original = stored(&db).await;
  let mut ticket = original.clone();
  let mut events = staff(&mut ticket, "Customer seems upset.", Visibility::Internal, None);
  events.extend(customer(&mut ticket, "Any news?"));

  save(&db, &ticket, original.updated_at(), &events).await;

  let public = db.repo().messages(ticket.id(), false).await.unwrap();
  let all = db.repo().messages(ticket.id(), true).await.unwrap();
  assert_eq!(public.iter().map(Message::body).collect::<Vec<_>>(), ["Any news?"]);
  assert_eq!(all.len(), 2);
  assert_eq!(all[0].visibility(), Visibility::Internal);
}

#[tokio::test]
async fn close_reason_and_reopen_count_round_trip() {
  let db = TestDb::start().await;
  let original = stored(&db).await;
  let mut ticket = original.clone();
  let mut events = staff(&mut ticket, "Fixed.", Visibility::Public, Some(TicketStatus::Resolved));
  events.extend(customer(&mut ticket, "Still broken."));
  events.extend(ticket.withdraw(LATER).unwrap());

  save(&db, &ticket, original.updated_at(), &events).await;

  let found = db.repo().find(ticket.id()).await.unwrap().unwrap();
  assert_eq!(found.status(), TicketStatus::Closed);
  assert_eq!(found.close_reason(), Some(CloseReason::Withdrawn));
  assert_eq!(found.reopen_count(), 1);
  assert_eq!(found, ticket);
  assert_eq!(db.events().await.last().unwrap().1["close_reason"], "withdrawn");
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
async fn the_app_role_cannot_change_the_schema_or_store_a_bad_row() {
  let db = TestDb::start().await;
  let ticket = stored(&db).await;
  let run = |sql: &'static str| sqlx::query(sql).bind(ticket.id()).execute(&db.app);

  let ddl = sqlx::query("CREATE TABLE intruder (id INT)").execute(&db.app).await;
  let bad_status = run("UPDATE ticket SET status = 'reopened' WHERE id = $1").await;
  let closed_without_reason = run("UPDATE ticket SET status = 'closed' WHERE id = $1").await;
  let reason_while_open = run("UPDATE ticket SET close_reason = 'withdrawn' WHERE id = $1").await;
  let customer_note = run(
    "INSERT INTO ticket_message (ticket_id, author_id, author, visibility, body)
     VALUES ($1, uuidv7(), 'customer', 'internal', 'sneaky')",
  )
  .await;

  assert!(ddl.is_err(), "tickets_app must be DML only");
  assert!(bad_status.is_err(), "unknown statuses are refused");
  assert!(closed_without_reason.is_err(), "closed needs a reason");
  assert!(reason_while_open.is_err(), "only closed has a reason");
  assert!(customer_note.is_err(), "customers cannot write internal notes");
}

#[tokio::test]
async fn a_ticket_opened_with_nanoseconds_can_be_saved_with_its_own_timestamp() {
  let db = TestDb::start().await;
  let precise = OPENED.replace_nanosecond(123_456_789).unwrap();
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
  let events = changed.withdraw(LATER).unwrap();

  assert!(save(&db, &changed, ticket.updated_at(), &events).await);
}

#[tokio::test]
async fn numbers_grow_and_a_stored_ticket_keeps_its_number() {
  let db = TestDb::start().await;
  let first = db.repo().next_number().await.unwrap();
  let second = db.repo().next_number().await.unwrap();
  let ticket = Ticket::open(second, Uuid::now_v7(), "Printer on fire", "Please help.", OPENED).unwrap();

  db.repo().create(&ticket, None).await.unwrap();

  assert!(second.value() > first.value());
  assert_eq!(db.repo().find(ticket.id()).await.unwrap().unwrap().number(), second);
  assert_eq!(db.events().await[0].1["reference"], second.to_string());
}
