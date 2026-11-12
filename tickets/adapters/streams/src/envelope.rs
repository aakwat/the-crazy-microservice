//! The contract envelope (contracts/events): the JSON every stream entry carries.

use domain::{OutboxEvent, PublishError};
use serde_json::{Value, json};
use time::format_description::well_known::Rfc3339;

use crate::publish_error;

/// The field holding the envelope in every stream entry.
pub const ENVELOPE_FIELD: &str = "envelope";

pub(crate) fn envelope(event: &OutboxEvent) -> Result<String, PublishError> {
  let payload: Value = serde_json::from_str(&event.payload).map_err(publish_error)?;
  let occurred_at = event.occurred_at.format(&Rfc3339).map_err(publish_error)?;
  let envelope = json!({
    "id": event.id.to_string(),
    "type": event.event_type,
    "version": event.version,
    "occurred_at": occurred_at,
    "subject_id": event.subject_id.to_string(),
    "traceparent": event.traceparent,
    "payload": payload,
  });
  Ok(envelope.to_string())
}
