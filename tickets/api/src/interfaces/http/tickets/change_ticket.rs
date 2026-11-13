//! `PATCH /api/tickets/{id}` — an agent moves the status or (re)assigns the ticket.

use application::AppError;
use axum::Json;
use axum::extract::rejection::PathRejection;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use domain::{AssigneeChange, TicketChange, TicketStatus};
use serde::{Deserialize, Deserializer};
use time::OffsetDateTime;
use uuid::Uuid;

use super::ticket_response::TicketResponse;
use crate::interfaces::http::Problem;
use crate::interfaces::http::extract::{Authenticated, JsonInput, Traceparent};
use crate::state::AppState;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChangeTicket {
  #[serde(default)]
  status: Option<String>,
  /// Absent keeps the assignee, `null` unassigns, an id assigns.
  #[serde(default, deserialize_with = "assignee")]
  assignee_id: AssigneeChange,
}

/// Only runs when the field is present, so absence stays the default `Keep`.
fn assignee<'de, D: Deserializer<'de>>(deserializer: D) -> Result<AssigneeChange, D::Error> {
  Ok(Option::<Uuid>::deserialize(deserializer)?.map_or(AssigneeChange::Unassign, AssigneeChange::Assign))
}

pub async fn change_ticket(
  State(state): State<AppState>,
  Authenticated(who): Authenticated,
  Traceparent(traceparent): Traceparent,
  id: Result<Path<Uuid>, PathRejection>,
  JsonInput(input): JsonInput<ChangeTicket>,
) -> Result<Json<TicketResponse>, Problem> {
  let Path(id) = id.map_err(|_| AppError::NotFound)?;
  let status = input
    .status
    .map(|name| {
      TicketStatus::parse(&name)
        .ok_or_else(|| Problem::new(StatusCode::UNPROCESSABLE_ENTITY, format!("unknown status {name:?}")))
    })
    .transpose()?;
  let change = TicketChange {
    status,
    assignee: input.assignee_id,
  };
  let now = OffsetDateTime::now_utc();
  let ticket = state
    .tickets
    .change(&who, id, change, now, traceparent.as_deref())
    .await?;
  Ok(Json(TicketResponse::for_viewer(&ticket, &who)))
}
