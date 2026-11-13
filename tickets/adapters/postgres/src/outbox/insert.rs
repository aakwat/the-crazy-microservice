//! The outbox write: one thin event per ticket change, in the change's own transaction.

use domain::Ticket;
use serde_json::{Value, json};
use sqlx::{PgConnection, query};
use uuid::Uuid;

/// Every ticket event goes to this stream (consumers: CLAUDE.md, events).
const EVENT_STREAM: &str = "tickets.events";
/// Bumped only for a breaking payload change, so consumers can tell shapes apart.
const EVENT_VERSION: i16 = 1;

pub(crate) async fn insert(
  conn: &mut PgConnection,
  event_type: &str,
  ticket: &Ticket,
  traceparent: Option<&str>,
) -> sqlx::Result<()> {
  query(
    "INSERT INTO outbox (id, stream, type, version, subject_id, traceparent, payload, occurred_at)
     VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
  )
  .bind(Uuid::now_v7())
  .bind(EVENT_STREAM)
  .bind(event_type)
  .bind(EVENT_VERSION)
  .bind(ticket.id())
  .bind(traceparent)
  .bind(payload(ticket))
  .bind(ticket.updated_at())
  .execute(conn)
  .await?;
  Ok(())
}

/// Ids, reference and status only, never ticket text: consumers fetch details from tickets if they need them.
fn payload(ticket: &Ticket) -> Value {
  json!({
    "ticket_id": ticket.id().to_string(),
    "reference": ticket.number().to_string(),
    "status": ticket.status().as_str(),
    "requester_id": ticket.requester_id().to_string(),
    "assignee_id": ticket.assignee_id().map(|id| id.to_string()),
  })
}
