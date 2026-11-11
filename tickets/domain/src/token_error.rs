//! `TokenError` — why a bearer token gave no principal.

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TokenError {
  /// The caller's fault: answer 401.
  #[error("access token is invalid: {0}")]
  Invalid(String),
  /// Ours: the signing keys could not be fetched, so answer 503.
  #[error("signing keys are unavailable: {0}")]
  Unavailable(String),
}
