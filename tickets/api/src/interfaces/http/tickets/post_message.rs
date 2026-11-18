//! `POST /api/tickets/{id}/messages` — reply; staff may also write a note, ask for something or resolve.

use application::{AppError, Reply};
use axum::Json;
use axum::extract::rejection::PathRejection;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use domain::TicketStatus;
use serde::Deserialize;
use time::OffsetDateTime;
use uuid::Uuid;

use super::message_response::MessageResponse;
use crate::interfaces::http::Problem;
use crate::interfaces::http::extract::{Authenticated, JsonInput, Traceparent};
use crate::state::AppState;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostMessage {
  body: String,
  #[serde(default)]
  internal: bool,
  #[serde(default)]
  then: Option<Then>,
}

/// The only statuses a reply may ask for; anything else is a 400 before the domain runs.
#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum Then {
  Pending,
  Resolved,
}

impl From<Then> for TicketStatus {
  fn from(then: Then) -> Self {
    match then {
      Then::Pending => Self::Pending,
      Then::Resolved => Self::Resolved,
    }
  }
}

pub async fn post_message(
  State(state): State<AppState>,
  Authenticated(who): Authenticated,
  Traceparent(traceparent): Traceparent,
  id: Result<Path<Uuid>, PathRejection>,
  JsonInput(input): JsonInput<PostMessage>,
) -> Result<(StatusCode, Json<MessageResponse>), Problem> {
  let Path(id) = id.map_err(|_| AppError::NotFound)?;
  let reply = Reply {
    body: &input.body,
    internal: input.internal,
    then: input.then.map(TicketStatus::from),
  };
  let now = OffsetDateTime::now_utc();
  let message = state
    .tickets
    .reply(&who, id, reply, now, traceparent.as_deref())
    .await?;
  Ok((StatusCode::CREATED, Json(MessageResponse::for_viewer(&message, &who))))
}
