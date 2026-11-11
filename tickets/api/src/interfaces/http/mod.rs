//! The HTTP surface: every route, plus RFC 9457 problems for anything unmatched.

mod extract;
mod problem;
mod tickets;

use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::http::StatusCode;
use axum::routing::get;

pub use problem::Problem;

use crate::config::MAX_REQUEST_BYTES;
use crate::state::AppState;

pub fn router(state: AppState) -> Router {
  Router::new()
    .merge(tickets::routes())
    .route("/healthz", get(|| async { StatusCode::NO_CONTENT }))
    .fallback(|| async { Problem::new(StatusCode::NOT_FOUND, "no such resource") })
    .method_not_allowed_fallback(|| async { Problem::new(StatusCode::METHOD_NOT_ALLOWED, "method not allowed") })
    .layer(DefaultBodyLimit::max(MAX_REQUEST_BYTES))
    .with_state(state)
}
