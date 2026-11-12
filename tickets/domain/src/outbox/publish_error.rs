//! `PublishError` — the bus refused or could not be reached, opaque to the layers above.

#[derive(Debug, thiserror::Error)]
#[error("publish failed: {0}")]
pub struct PublishError(#[source] pub Box<dyn std::error::Error + Send + Sync>);
