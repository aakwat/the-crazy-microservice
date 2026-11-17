//! `trace_request` — one span per request, continuing the gateway's trace and named after the route.

use axum::extract::{MatchedPath, Request};
use axum::middleware::Next;
use axum::response::Response;
use opentelemetry::global;
use opentelemetry_http::HeaderExtractor;
use tracing::Instrument;
use tracing::field::Empty;
use tracing_opentelemetry::OpenTelemetrySpanExt;

/// Records method, route and status only: no ids from the path, no headers, no client address.
pub async fn trace_request(request: Request, next: Next) -> Response {
  let parent = global::get_text_map_propagator(|propagator| propagator.extract(&HeaderExtractor(request.headers())));
  let method = request.method().clone();
  let route = request
    .extensions()
    .get::<MatchedPath>()
    .map_or("", MatchedPath::as_str)
    .to_owned();
  let span = tracing::info_span!(
    "request",
    otel.name = %format!("{method} {route}"),
    otel.kind = "server",
    otel.status_code = Empty,
    http.request.method = %method,
    http.route = %route,
    http.response.status_code = Empty,
  );
  // Fails only when no OpenTelemetry layer is installed; the request then runs untraced.
  let _ = span.set_parent(parent);

  let response = next.run(request).instrument(span.clone()).await;

  span.record("http.response.status_code", response.status().as_u16());
  if response.status().is_server_error() {
    span.record("otel.status_code", "error");
  }
  response
}
