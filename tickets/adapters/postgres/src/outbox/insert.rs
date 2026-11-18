//! The outbox write: one thin event per ticket change, in the change's own transaction.

use domain::{CloseReason, Message, Ticket, TicketEvent};
use serde_json::{Value, json};
use sqlx::{PgConnection, query};
use time::OffsetDateTime;
use uuid::Uuid;

/// Every ticket event goes to this stream (consumers: CLAUDE.md, events).
const EVENT_STREAM: &str = "tickets.events";
/// Bumped only for a breaking payload change, so consumers can tell shapes apart.
const EVENT_VERSION: i16 = 2;

/// `None` is the `ticket.created` event.
pub(crate) async fn insert(
  conn: &mut PgConnection,
  ticket: &Ticket,
  event: Option<&TicketEvent>,
  traceparent: Option<&str>,
) -> sqlx::Result<()> {
  query(
    "INSERT INTO outbox (id, stream, type, version, subject_id, traceparent, payload, occurred_at)
     VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
  )
  .bind(Uuid::now_v7())
  .bind(EVENT_STREAM)
  .bind(event_type(event))
  .bind(EVENT_VERSION)
  .bind(ticket.id())
  .bind(traceparent)
  .bind(payload(ticket, event))
  .bind(occurred_at(ticket, event))
  .execute(conn)
  .await?;
  Ok(())
}

/// A note leaves `updated_at` as it was, so a message carries its own time.
fn occurred_at(ticket: &Ticket, event: Option<&TicketEvent>) -> OffsetDateTime {
  match event {
    Some(TicketEvent::MessageAdded(message)) => message.created_at(),
    _ => ticket.updated_at(),
  }
}

fn event_type(event: Option<&TicketEvent>) -> &'static str {
  match event {
    None => "ticket.created",
    Some(TicketEvent::StatusChanged) => "ticket.status_changed",
    Some(TicketEvent::Assigned) => "ticket.assigned",
    Some(TicketEvent::MessageAdded(_)) => "ticket.message_added",
  }
}

/// Ids, reference and status only, never ticket or message text: consumers fetch details from tickets.
fn payload(ticket: &Ticket, event: Option<&TicketEvent>) -> Value {
  let message = match event {
    Some(TicketEvent::MessageAdded(message)) => message_kind(message),
    _ => Value::Null,
  };
  json!({
    "ticket_id": ticket.id().to_string(),
    "reference": ticket.number().to_string(),
    "status": ticket.status().as_str(),
    "close_reason": ticket.close_reason().map(CloseReason::as_str),
    "requester_id": ticket.requester_id().to_string(),
    "assignee_id": ticket.assignee_id().map(|id| id.to_string()),
    "message": message,
  })
}

fn message_kind(message: &Message) -> Value {
  json!({
    "id": message.id().to_string(),
    "author": message.author().as_str(),
    "visibility": message.visibility().as_str(),
  })
}
