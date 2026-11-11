//! `POST /api/tickets` — open a ticket for the caller.

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::http::header::LOCATION;
use axum::response::IntoResponse;
use serde::Deserialize;
use time::OffsetDateTime;

use super::ticket_response::TicketResponse;
use crate::interfaces::http::Problem;
use crate::interfaces::http::extract::{Authenticated, JsonInput, Traceparent};
use crate::state::AppState;

/// Unknown fields are refused, so a client cannot try to set the requester or status.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateTicket {
  title: String,
  body: String,
}

pub async fn create_ticket(
  State(state): State<AppState>,
  Authenticated(who): Authenticated,
  Traceparent(traceparent): Traceparent,
  JsonInput(input): JsonInput<CreateTicket>,
) -> Result<impl IntoResponse, Problem> {
  let now = OffsetDateTime::now_utc();
  let ticket = state
    .tickets
    .open(&who, &input.title, &input.body, now, traceparent.as_deref())
    .await?;
  let location = format!("/api/tickets/{}", ticket.id());
  Ok((
    StatusCode::CREATED,
    [(LOCATION, location)],
    Json(TicketResponse::from(ticket)),
  ))
}
