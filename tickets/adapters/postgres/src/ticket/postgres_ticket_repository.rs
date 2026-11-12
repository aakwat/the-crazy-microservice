//! `PostgresTicketRepository` — tickets and their outbox events in Postgres.

use domain::{StoreError, Ticket, TicketRepository, TicketSummary};
use sqlx::{PgPool, query, query_as};
use time::OffsetDateTime;
use uuid::Uuid;

use super::ticket_row::TicketRow;
use super::ticket_summary_row::TicketSummaryRow;
use crate::{outbox, store};

pub struct PostgresTicketRepository {
  pool: PgPool,
}

impl PostgresTicketRepository {
  pub fn new(pool: PgPool) -> Self {
    Self { pool }
  }
}

impl TicketRepository for PostgresTicketRepository {
  async fn create(&self, ticket: &Ticket, traceparent: Option<&str>) -> Result<(), StoreError> {
    let mut tx = self.pool.begin().await.map_err(store)?;
    query(
      "INSERT INTO ticket (id, requester_id, assignee_id, title, body, status, created_at, updated_at)
       VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(ticket.id())
    .bind(ticket.requester_id())
    .bind(ticket.assignee_id())
    .bind(ticket.title())
    .bind(ticket.body())
    .bind(ticket.status().as_str())
    .bind(ticket.created_at())
    .bind(ticket.updated_at())
    .execute(&mut *tx)
    .await
    .map_err(store)?;
    outbox::insert(&mut tx, "ticket.created", ticket, traceparent)
      .await
      .map_err(store)?;
    tx.commit().await.map_err(store)
  }

  async fn update(
    &self,
    ticket: &Ticket,
    expected_updated_at: OffsetDateTime,
    traceparent: Option<&str>,
  ) -> Result<bool, StoreError> {
    let mut tx = self.pool.begin().await.map_err(store)?;
    let updated = query(
      "UPDATE ticket SET assignee_id = $2, status = $3, updated_at = $4
       WHERE id = $1 AND updated_at = $5",
    )
    .bind(ticket.id())
    .bind(ticket.assignee_id())
    .bind(ticket.status().as_str())
    .bind(ticket.updated_at())
    .bind(expected_updated_at)
    .execute(&mut *tx)
    .await
    .map_err(store)?
    .rows_affected();
    if updated == 0 {
      return Ok(false);
    }
    outbox::insert(&mut tx, "ticket.changed", ticket, traceparent)
      .await
      .map_err(store)?;
    tx.commit().await.map_err(store)?;
    Ok(true)
  }

  async fn find(&self, id: Uuid) -> Result<Option<Ticket>, StoreError> {
    let row: Option<TicketRow> = query_as(
      "SELECT id, requester_id, assignee_id, title, body, status, created_at, updated_at
       FROM ticket WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&self.pool)
    .await
    .map_err(store)?;
    row.map(TicketRow::into_ticket).transpose()
  }

  async fn list(
    &self,
    requester: Option<Uuid>,
    before: Option<Uuid>,
    limit: u32,
  ) -> Result<Vec<TicketSummary>, StoreError> {
    let rows: Vec<TicketSummaryRow> = query_as(
      "SELECT id, requester_id, assignee_id, title, status, updated_at
       FROM ticket
       WHERE ($1::uuid IS NULL OR requester_id = $1) AND ($2::uuid IS NULL OR id < $2)
       ORDER BY id DESC LIMIT $3",
    )
    .bind(requester)
    .bind(before)
    .bind(i64::from(limit))
    .fetch_all(&self.pool)
    .await
    .map_err(store)?;
    rows.into_iter().map(TicketSummaryRow::into_summary).collect()
  }
}
