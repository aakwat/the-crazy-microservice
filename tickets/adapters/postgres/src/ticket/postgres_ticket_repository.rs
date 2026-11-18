//! `PostgresTicketRepository` — tickets, their messages and outbox events in Postgres.

use domain::{
  CloseReason, Message, StoreError, Ticket, TicketEvent, TicketNumber, TicketRepository, TicketSummary, Visibility,
};
use sqlx::{PgConnection, PgPool, query, query_as, query_scalar};
use time::OffsetDateTime;
use uuid::Uuid;

use super::column::{parse_number, stored_count, stored_number};
use super::message_row::MessageRow;
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
  async fn next_number(&self) -> Result<TicketNumber, StoreError> {
    let next: i64 = query_scalar("SELECT nextval('ticket_number_seq')")
      .fetch_one(&self.pool)
      .await
      .map_err(store)?;
    parse_number(next)
  }

  async fn create(&self, ticket: &Ticket, traceparent: Option<&str>) -> Result<(), StoreError> {
    let mut tx = self.pool.begin().await.map_err(store)?;
    query(
      "INSERT INTO ticket (id, number, requester_id, assignee_id, title, body, status, close_reason,
                           reopen_count, created_at, updated_at)
       VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
    )
    .bind(ticket.id())
    .bind(stored_number(ticket.number())?)
    .bind(ticket.requester_id())
    .bind(ticket.assignee_id())
    .bind(ticket.title())
    .bind(ticket.body())
    .bind(ticket.status().as_str())
    .bind(ticket.close_reason().map(CloseReason::as_str))
    .bind(stored_count(ticket.reopen_count())?)
    .bind(ticket.created_at())
    .bind(ticket.updated_at())
    .execute(&mut *tx)
    .await
    .map_err(store)?;
    outbox::insert(&mut tx, ticket, None, traceparent)
      .await
      .map_err(store)?;
    tx.commit().await.map_err(store)
  }

  async fn save(
    &self,
    ticket: &Ticket,
    expected_updated_at: OffsetDateTime,
    events: &[TicketEvent],
    traceparent: Option<&str>,
  ) -> Result<bool, StoreError> {
    let mut tx = self.pool.begin().await.map_err(store)?;
    let updated = query(
      "UPDATE ticket SET assignee_id = $2, status = $3, close_reason = $4, reopen_count = $5, updated_at = $6
       WHERE id = $1 AND updated_at = $7",
    )
    .bind(ticket.id())
    .bind(ticket.assignee_id())
    .bind(ticket.status().as_str())
    .bind(ticket.close_reason().map(CloseReason::as_str))
    .bind(stored_count(ticket.reopen_count())?)
    .bind(ticket.updated_at())
    .bind(expected_updated_at)
    .execute(&mut *tx)
    .await
    .map_err(store)?
    .rows_affected();
    if updated == 0 {
      return Ok(false);
    }
    for event in events {
      if let TicketEvent::MessageAdded(message) = event {
        insert_message(&mut tx, message).await.map_err(store)?;
      }
      outbox::insert(&mut tx, ticket, Some(event), traceparent)
        .await
        .map_err(store)?;
    }
    tx.commit().await.map_err(store)?;
    Ok(true)
  }

  async fn find(&self, id: Uuid) -> Result<Option<Ticket>, StoreError> {
    let row: Option<TicketRow> = query_as(
      "SELECT id, number, requester_id, assignee_id, title, body, status, close_reason, reopen_count,
              created_at, updated_at
       FROM ticket WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&self.pool)
    .await
    .map_err(store)?;
    row.map(TicketRow::into_ticket).transpose()
  }

  async fn messages(&self, ticket_id: Uuid, include_internal: bool) -> Result<Vec<Message>, StoreError> {
    let rows: Vec<MessageRow> = query_as(
      "SELECT id, ticket_id, author_id, author, visibility, body, created_at
       FROM ticket_message
       WHERE ticket_id = $1 AND ($2 OR visibility = $3)
       ORDER BY id",
    )
    .bind(ticket_id)
    .bind(include_internal)
    .bind(Visibility::Public.as_str())
    .fetch_all(&self.pool)
    .await
    .map_err(store)?;
    rows.into_iter().map(MessageRow::into_message).collect()
  }

  async fn list(
    &self,
    requester: Option<Uuid>,
    before: Option<Uuid>,
    limit: u32,
  ) -> Result<Vec<TicketSummary>, StoreError> {
    let rows: Vec<TicketSummaryRow> = query_as(
      "SELECT id, number, requester_id, assignee_id, title, status, updated_at
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

async fn insert_message(conn: &mut PgConnection, message: &Message) -> sqlx::Result<()> {
  query(
    "INSERT INTO ticket_message (id, ticket_id, author_id, author, visibility, body, created_at)
     VALUES ($1, $2, $3, $4, $5, $6, $7)",
  )
  .bind(message.id())
  .bind(message.ticket_id())
  .bind(message.author_id())
  .bind(message.author().as_str())
  .bind(message.visibility().as_str())
  .bind(message.body())
  .bind(message.created_at())
  .execute(conn)
  .await?;
  Ok(())
}
