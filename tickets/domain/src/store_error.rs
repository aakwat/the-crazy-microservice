//! `StoreError` — a persistence failure, opaque to the layers above.

/// The adapter's own error, boxed so the domain never depends on a database crate.
#[derive(Debug, thiserror::Error)]
#[error("store failed: {0}")]
pub struct StoreError(#[source] pub Box<dyn std::error::Error + Send + Sync>);
