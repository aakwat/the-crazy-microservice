//! `GET /healthz` — the process serves HTTP and its outbox relay still finishes rounds.

use axum::extract::State;
use axum::http::StatusCode;
use time::OffsetDateTime;

use crate::config::MAX_RELAY_SILENCE;
use crate::state::AppState;

/// Never checks Postgres or Redis directly: a restart cannot fix them, so health would only flap.
pub(super) async fn health(State(state): State<AppState>) -> StatusCode {
  if state
    .heartbeat
    .is_silent_for(MAX_RELAY_SILENCE, OffsetDateTime::now_utc())
  {
    StatusCode::SERVICE_UNAVAILABLE
  } else {
    StatusCode::NO_CONTENT
  }
}
