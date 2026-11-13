//! `GET /api/tickets/{id}` — one ticket, if the caller may see it.

use application::AppError;
use axum::Json;
use axum::extract::rejection::PathRejection;
use axum::extract::{Path, State};
use uuid::Uuid;

use super::ticket_response::TicketResponse;
use crate::interfaces::http::Problem;
use crate::interfaces::http::extract::Authenticated;
use crate::state::AppState;

pub async fn get_ticket(
  State(state): State<AppState>,
  Authenticated(who): Authenticated,
  id: Result<Path<Uuid>, PathRejection>,
) -> Result<Json<TicketResponse>, Problem> {
  // A malformed id names no ticket.
  let Path(id) = id.map_err(|_| AppError::NotFound)?;
  let ticket = state.tickets.get(&who, id).await?;
  Ok(Json(TicketResponse::for_viewer(&ticket, &who)))
}
