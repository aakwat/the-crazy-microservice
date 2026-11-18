//! `POST /api/tickets/{id}/confirm` — the requester accepts the resolution; the ticket closes.

use application::AppError;
use axum::Json;
use axum::extract::rejection::PathRejection;
use axum::extract::{Path, State};
use time::OffsetDateTime;
use uuid::Uuid;

use super::ticket_response::TicketResponse;
use crate::interfaces::http::Problem;
use crate::interfaces::http::extract::{Authenticated, Traceparent};
use crate::state::AppState;

pub async fn confirm_ticket(
  State(state): State<AppState>,
  Authenticated(who): Authenticated,
  Traceparent(traceparent): Traceparent,
  id: Result<Path<Uuid>, PathRejection>,
) -> Result<Json<TicketResponse>, Problem> {
  let Path(id) = id.map_err(|_| AppError::NotFound)?;
  let now = OffsetDateTime::now_utc();
  let ticket = state.tickets.confirm(&who, id, now, traceparent.as_deref()).await?;
  Ok(Json(TicketResponse::for_viewer(&ticket, &who)))
}
