//! `JsonInput` — a JSON body whose rejections are RFC 9457 problems instead of plain text.

use axum::Json;
use axum::extract::rejection::JsonRejection;
use axum::extract::{FromRequest, Request};
use axum::http::StatusCode;
use serde::de::DeserializeOwned;

use crate::interfaces::http::Problem;

pub struct JsonInput<T>(pub T);

impl<T: DeserializeOwned, S: Send + Sync> FromRequest<S> for JsonInput<T> {
  type Rejection = Problem;

  async fn from_request(request: Request, state: &S) -> Result<Self, Problem> {
    let Json(value) = Json::<T>::from_request(request, state)
      .await
      .map_err(|rejection| problem(&rejection))?;
    Ok(Self(value))
  }
}

/// A body that does not fit the DTO is a 400, as in auth; 422 is kept for domain rules.
fn problem(rejection: &JsonRejection) -> Problem {
  let status = match rejection {
    JsonRejection::JsonDataError(_) => StatusCode::BAD_REQUEST,
    _ => rejection.status(),
  };
  Problem::new(status, rejection.body_text())
}
