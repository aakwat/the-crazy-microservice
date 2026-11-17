//! Logs and traces: JSON logs to stdout, spans to OTLP when `OTEL_EXPORTER_OTLP_ENDPOINT` is set.

use opentelemetry::global;
use opentelemetry::trace::TracerProvider;
use opentelemetry_otlp::{ExporterBuildError, SpanExporter};
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::trace::SdkTracerProvider;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

/// Spans are always recorded so every write gets a trace id for its outbox event, even with no exporter.
pub fn init() -> Result<SdkTracerProvider, ExporterBuildError> {
  let provider = tracer_provider()?;
  global::set_text_map_propagator(TraceContextPropagator::new());
  tracing_subscriber::registry()
    .with(tracing_subscriber::fmt::layer().json())
    .with(tracing_opentelemetry::layer().with_tracer(provider.tracer("tickets")))
    .init();
  Ok(provider)
}

fn tracer_provider() -> Result<SdkTracerProvider, ExporterBuildError> {
  let builder = SdkTracerProvider::builder();
  if std::env::var_os("OTEL_EXPORTER_OTLP_ENDPOINT").is_none() {
    return Ok(builder.build());
  }
  let exporter = SpanExporter::builder().with_http().build()?;
  Ok(builder.with_batch_exporter(exporter).build())
}
