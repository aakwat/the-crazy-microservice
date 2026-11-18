//! `GET /api/tickets/{id}/messages` — the conversation the caller may see.

use application::AppError;
use axum::Json;
use axum::extract::rejection::PathRejection;
use axum::extract::{Path, State};
use uuid::Uuid;

use super::message_list_response::MessageListResponse;
use super::message_response::MessageResponse;
use crate::interfaces::http::Problem;
use crate::interfaces::http::extract::Authenticated;
use crate::state::AppState;

pub async fn list_messages(
  State(state): State<AppState>,
  Authenticated(who): Authenticated,
  id: Result<Path<Uuid>, PathRejection>,
) -> Result<Json<MessageListResponse>, Problem> {
  let Path(id) = id.map_err(|_| AppError::NotFound)?;
  let messages = state.tickets.messages(&who, id).await?;
  let items = messages
    .iter()
    .map(|message| MessageResponse::for_viewer(message, &who))
    .collect();
  Ok(Json(MessageListResponse { items }))
}
