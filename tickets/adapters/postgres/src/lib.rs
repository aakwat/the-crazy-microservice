//! Postgres adapter: the ticket repository, its outbox and the schema migrations.

mod outbox;
mod ticket;

use domain::StoreError;

pub use outbox::{PostgresOutbox, PostgresOutboxBatch};
pub use ticket::PostgresTicketRepository;

/// Applies pending migrations; call it with the schema owner's pool, never the app's.
pub async fn migrate(migrator: &sqlx::PgPool) -> Result<(), sqlx::migrate::MigrateError> {
  sqlx::migrate!("./migrations").run(migrator).await
}

fn store(error: sqlx::Error) -> StoreError {
  StoreError(Box::new(error))
}
