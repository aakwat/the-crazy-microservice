//! `Traceparent` — the current request span as a W3C traceparent, so an outbox event continues this trace.

use std::collections::HashMap;
use std::convert::Infallible;

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use opentelemetry::global;
use tracing_opentelemetry::OpenTelemetrySpanExt;

pub struct Traceparent(pub Option<String>);

impl<S: Send + Sync> FromRequestParts<S> for Traceparent {
  type Rejection = Infallible;

  /// Built from the span `trace_request` opened, never copied from the client's header.
  fn from_request_parts(_parts: &mut Parts, _state: &S) -> impl Future<Output = Result<Self, Infallible>> + Send {
    let mut carrier = HashMap::new();
    let context = tracing::Span::current().context();
    global::get_text_map_propagator(|propagator| propagator.inject_context(&context, &mut carrier));
    std::future::ready(Ok(Self(carrier.remove("traceparent"))))
  }
}
