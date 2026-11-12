//! Real Postgres 18 with the real roles script: migrates as the owner, works as the app.
#![allow(dead_code)]

use postgres::{PostgresOutbox, PostgresTicketRepository};
use serde_json::Value;
use sqlx::PgPool;
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, ImageExt};
use testcontainers_modules::postgres::Postgres;

pub struct TestDb {
  _container: ContainerAsync<Postgres>,
  pub app: PgPool,
}

impl TestDb {
  pub async fn start() -> Self {
    let container = Postgres::default()
      .with_db_name("tickets")
      .with_tag("18-alpine")
      .with_copy_to(
        "/docker-entrypoint-initdb.d/01-roles.sql",
        include_bytes!("../../../../db/init/01-roles.sql").to_vec(),
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

  pub fn repo(&self) -> PostgresTicketRepository {
    PostgresTicketRepository::new(self.app.clone())
  }

  pub fn outbox(&self) -> PostgresOutbox {
    PostgresOutbox::new(self.app.clone())
  }

  pub async fn events(&self) -> Vec<(String, Value)> {
    sqlx::query_as("SELECT type, payload FROM outbox ORDER BY id")
      .fetch_all(&self.app)
      .await
      .expect("outbox")
  }
}
