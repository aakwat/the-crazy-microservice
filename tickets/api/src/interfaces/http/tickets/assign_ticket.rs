//! `PATCH /api/tickets/{id}` — an agent assigns or unassigns the ticket; status moves only through messages.

use application::AppError;
use axum::Json;
use axum::extract::rejection::PathRejection;
use axum::extract::{Path, State};
use domain::AssigneeChange;
use serde::{Deserialize, Deserializer};
use time::OffsetDateTime;
use uuid::Uuid;

use super::ticket_response::TicketResponse;
use crate::interfaces::http::Problem;
use crate::interfaces::http::extract::{Authenticated, JsonInput, Traceparent};
use crate::state::AppState;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssignTicket {
  /// Required: `null` unassigns, an id assigns.
  #[serde(deserialize_with = "assignee")]
  assignee_id: AssigneeChange,
}

fn assignee<'de, D: Deserializer<'de>>(deserializer: D) -> Result<AssigneeChange, D::Error> {
  Ok(Option::<Uuid>::deserialize(deserializer)?.map_or(AssigneeChange::Unassign, AssigneeChange::Assign))
}

pub async fn assign_ticket(
  State(state): State<AppState>,
  Authenticated(who): Authenticated,
  Traceparent(traceparent): Traceparent,
  id: Result<Path<Uuid>, PathRejection>,
  JsonInput(input): JsonInput<AssignTicket>,
) -> Result<Json<TicketResponse>, Problem> {
  let Path(id) = id.map_err(|_| AppError::NotFound)?;
  let now = OffsetDateTime::now_utc();
  let ticket = state
    .tickets
    .assign(&who, id, input.assignee_id, now, traceparent.as_deref())
    .await?;
  Ok(Json(TicketResponse::for_viewer(&ticket, &who)))
}
