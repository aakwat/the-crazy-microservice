//! The HTTP surface: every route, plus RFC 9457 problems for anything unmatched.

mod extract;
mod health;
mod problem;
mod tickets;
mod trace_request;

use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::http::StatusCode;
use axum::middleware::from_fn;
use axum::routing::get;

pub use problem::Problem;

use crate::config::MAX_REQUEST_BYTES;
use crate::state::AppState;

pub fn router(state: AppState) -> Router {
  Router::new()
    // A route layer sees the matched route, and leaves the probe's /healthz out of the traces.
    .merge(tickets::routes().route_layer(from_fn(trace_request::trace_request)))
    .route("/healthz", get(health::health))
    .fallback(|| async { Problem::new(StatusCode::NOT_FOUND, "no such resource") })
    .method_not_allowed_fallback(|| async { Problem::new(StatusCode::METHOD_NOT_ALLOWED, "method not allowed") })
    .layer(DefaultBodyLimit::max(MAX_REQUEST_BYTES))
    .with_state(state)
}
