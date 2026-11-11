//! Postgres adapter: the ticket repository, its outbox and the schema migrations.

mod outbox;
mod postgres_ticket_repository;
mod ticket_row;
mod ticket_summary_row;

pub use postgres_ticket_repository::PostgresTicketRepository;

/// Applies pending migrations; call it with the schema owner's pool, never the app's.
pub async fn migrate(migrator: &sqlx::PgPool) -> Result<(), sqlx::migrate::MigrateError> {
  sqlx::migrate!("./migrations").run(migrator).await
}
