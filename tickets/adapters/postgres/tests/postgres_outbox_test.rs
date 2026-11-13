//! Outbox claim, completion and purge against real Postgres.
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use std::collections::HashSet;

use common::TestDb;
use domain::{Outbox, OutboxBatch, Ticket, TicketRepository};
use time::OffsetDateTime;
use time::macros::datetime;
use uuid::Uuid;

const T0: OffsetDateTime = datetime!(2026-11-12 09:00 UTC);

async fn create_tickets(db: &TestDb, count: usize) {
  for _ in 0..count {
    let ticket = Ticket::open(common::number(), Uuid::now_v7(), "Printer on fire", "Please help.", T0).unwrap();
    db.repo().create(&ticket, None).await.unwrap();
  }
}

async fn outbox_ids(db: &TestDb) -> Vec<Uuid> {
  sqlx::query_scalar("SELECT id FROM outbox ORDER BY occurred_at, id")
    .fetch_all(&db.app)
    .await
    .unwrap()
}

fn ids(batch: &impl OutboxBatch) -> Vec<Uuid> {
  batch.events().iter().map(|event| event.id).collect()
}

#[tokio::test]
async fn a_claim_returns_unpublished_events_oldest_first() {
  let db = TestDb::start().await;
  create_tickets(&db, 3).await;

  let batch = db.outbox().claim(10).await.unwrap();

  assert_eq!(ids(&batch), outbox_ids(&db).await);
}

#[tokio::test]
async fn complete_marks_only_the_published_events() {
  let db = TestDb::start().await;
  create_tickets(&db, 3).await;
  let all = outbox_ids(&db).await;

  let batch = db.outbox().claim(10).await.unwrap();
  batch.complete(&all[..1]).await.unwrap();
  let next = db.outbox().claim(10).await.unwrap();

  assert_eq!(ids(&next), all[1..]);
}

#[tokio::test]
async fn a_dropped_batch_releases_its_events() {
  let db = TestDb::start().await;
  create_tickets(&db, 2).await;

  drop(db.outbox().claim(10).await.unwrap());
  let again = db.outbox().claim(10).await.unwrap();

  assert_eq!(ids(&again), outbox_ids(&db).await);
}

#[tokio::test]
async fn two_claims_at_once_never_share_an_event() {
  let db = TestDb::start().await;
  create_tickets(&db, 10).await;

  let first = db.outbox().claim(4).await.unwrap();
  let second = db.outbox().claim(100).await.unwrap();

  let first_ids: HashSet<Uuid> = ids(&first).into_iter().collect();
  let second_ids: HashSet<Uuid> = ids(&second).into_iter().collect();
  assert_eq!((first_ids.len(), second_ids.len()), (4, 6));
  assert!(first_ids.is_disjoint(&second_ids));
}

#[tokio::test]
async fn purge_removes_published_events_only() {
  let db = TestDb::start().await;
  create_tickets(&db, 2).await;
  let published = outbox_ids(&db).await;
  db.outbox().claim(10).await.unwrap().complete(&published).await.unwrap();
  create_tickets(&db, 1).await;

  let purged = db
    .outbox()
    .purge_published(OffsetDateTime::now_utc() + time::Duration::minutes(1))
    .await
    .unwrap();

  assert_eq!(purged, 2);
  assert_eq!(outbox_ids(&db).await.len(), 1);
}
