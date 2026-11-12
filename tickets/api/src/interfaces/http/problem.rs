//! `Problem` — RFC 9457 error responses, and how use-case and token errors map onto them.

use application::AppError;
use axum::Json;
use axum::http::header::{CONTENT_TYPE, WWW_AUTHENTICATE};
use axum::http::{HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use domain::TokenError;
use serde_json::json;

#[derive(Debug)]
pub struct Problem {
  status: StatusCode,
  detail: String,
}

impl Problem {
  /// `detail` is shown to the client, so it must never carry internal errors.
  pub fn new(status: StatusCode, detail: impl Into<String>) -> Self {
    Self {
      status,
      detail: detail.into(),
    }
  }
}

impl IntoResponse for Problem {
  fn into_response(self) -> Response {
    let title = self.status.canonical_reason().unwrap_or("Error");
    let body = json!({"type": "about:blank", "title": title, "status": self.status.as_u16(), "detail": self.detail});
    let mut response = (self.status, Json(body)).into_response();
    let headers = response.headers_mut();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/problem+json"));
    if self.status == StatusCode::UNAUTHORIZED {
      headers.insert(WWW_AUTHENTICATE, HeaderValue::from_static("Bearer"));
    }
    response
  }
}

impl From<AppError> for Problem {
  fn from(error: AppError) -> Self {
    match error {
      AppError::NotFound => Self::new(StatusCode::NOT_FOUND, error.to_string()),
      AppError::Forbidden => Self::new(StatusCode::FORBIDDEN, error.to_string()),
      AppError::Conflict => Self::new(StatusCode::CONFLICT, error.to_string()),
      AppError::Invalid(invalid) => Self::new(StatusCode::UNPROCESSABLE_ENTITY, invalid.to_string()),
      AppError::Store(store) => {
        tracing::error!(event = "store.failed", error = %store);
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, "unexpected error")
      }
    }
  }
}

impl From<TokenError> for Problem {
  /// The reason is logged, never sent: it would tell an attacker which check their token failed.
  fn from(error: TokenError) -> Self {
    match error {
      TokenError::Invalid(reason) => {
        tracing::debug!(event = "auth.token_refused", reason);
        Self::new(StatusCode::UNAUTHORIZED, "log in or send a valid access token")
      }
      TokenError::Unavailable(reason) => {
        tracing::warn!(event = "auth.keys_unavailable", reason);
        Self::new(StatusCode::SERVICE_UNAVAILABLE, "sign-in is unavailable; try again")
      }
    }
  }
}
