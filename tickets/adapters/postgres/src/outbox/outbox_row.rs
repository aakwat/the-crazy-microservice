//! `OutboxRow` — an unpublished `outbox` row as sqlx reads it.

use domain::OutboxEvent;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(sqlx::FromRow)]
pub(crate) struct OutboxRow {
  id: Uuid,
  stream: String,
  #[sqlx(rename = "type")]
  event_type: String,
  version: i16,
  subject_id: Uuid,
  traceparent: Option<String>,
  payload: String,
  occurred_at: OffsetDateTime,
}

impl From<OutboxRow> for OutboxEvent {
  fn from(row: OutboxRow) -> Self {
    Self {
      id: row.id,
      stream: row.stream,
      event_type: row.event_type,
      version: row.version,
      subject_id: row.subject_id,
      traceparent: row.traceparent,
      payload: row.payload,
      occurred_at: row.occurred_at,
    }
  }
}
