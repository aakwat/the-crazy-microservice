//! `OutboxEvent` — one stored event waiting to be published, already in its wire shape.

use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutboxEvent {
  pub id: Uuid,
  pub stream: String,
  pub event_type: String,
  pub version: i16,
  pub subject_id: Uuid,
  pub traceparent: Option<String>,
  /// Serialized JSON, kept as text so the domain needs no JSON library.
  pub payload: String,
  pub occurred_at: OffsetDateTime,
}
