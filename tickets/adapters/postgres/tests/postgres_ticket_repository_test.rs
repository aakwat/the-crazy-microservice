//! Runs against real Postgres 18 with the real roles script: migrates as the owner, works as the app.

use domain::{Ticket, TicketChange, TicketRepository, TicketStatus};
use postgres::PostgresTicketRepository;
use serde_json::Value;
use sqlx::PgPool;
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, ImageExt};
use testcontainers_modules::postgres::Postgres;
use time::OffsetDateTime;
use time::macros::datetime;
use uuid::Uuid;

const T0: OffsetDateTime = datetime!(2026-11-09 09:00 UTC);
const T1: OffsetDateTime = datetime!(2026-11-09 10:00 UTC);
const TRACE: &str = "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01";

struct TestDb {
  _container: ContainerAsync<Postgres>,
  app: PgPool,
}

impl TestDb {
  async fn start() -> Self {
    let container = Postgres::default()
      .with_db_name("tickets")
      .with_tag("18-alpine")
      .with_copy_to(
        "/docker-entrypoint-initdb.d/01-roles.sql",
        include_bytes!("../../../db/init/01-roles.sql").to_vec(),
      )
      .with_copy_to(
        "/run/secrets/TICKETS_DB_MIGRATOR_PASSWORD",
        b"migrator-test-password".to_vec(),
      )
      .with_copy_to("/run/secrets/TICKETS_DB_APP_PASSWORD", b"app-test-password".to_vec())
      .start()
      .await
      .expect("start postgres");
    let port = container.get_host_port_ipv4(5432).await.expect("postgres port");
    let url = |user: &str, password: &str| format!("postgres://{user}:{password}@127.0.0.1:{port}/tickets");

    let migrator = PgPool::connect(&url("tickets_migrator", "migrator-test-password"))
      .await
      .expect("migrator");
    postgres::migrate(&migrator).await.expect("migrate");
    let app = PgPool::connect(&url("tickets_app", "app-test-password"))
      .await
      .expect("app");
    Self {
      _container: container,
      app,
    }
  }

  fn repo(&self) -> PostgresTicketRepository {
    PostgresTicketRepository::new(self.app.clone())
  }

  async fn events(&self) -> Vec<(String, Value)> {
    sqlx::query_as("SELECT type, payload FROM outbox ORDER BY id")
      .fetch_all(&self.app)
      .await
      .expect("outbox")
  }
}

fn ticket_for(requester: Uuid) -> Ticket {
  Ticket::open(requester, "Printer on fire", "Please help.", T0).unwrap()
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
  let ticket = Ticket::open(Uuid::now_v7(), "Printer on fire", "Please help.", precise).unwrap();
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
