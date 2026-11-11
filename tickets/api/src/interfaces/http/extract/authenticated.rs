//! `Authenticated` — the caller's principal, from a verified bearer access token; never from an identity header.

use axum::extract::FromRequestParts;
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;
use domain::{AccessTokenVerifier, Principal, TokenError};

use crate::interfaces::http::Problem;
use crate::state::AppState;

pub struct Authenticated(pub Principal);

impl FromRequestParts<AppState> for Authenticated {
  type Rejection = Problem;

  async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Problem> {
    let header = parts
      .headers
      .get(AUTHORIZATION)
      .and_then(|value| value.to_str().ok())
      .unwrap_or_default();
    let token = header
      .split_once(' ')
      .filter(|(scheme, token)| scheme.eq_ignore_ascii_case("Bearer") && !token.is_empty())
      .map(|(_, token)| token)
      .ok_or_else(|| TokenError::Invalid("no bearer token".to_owned()))?;
    Ok(Self(state.verifier.verify(token).await?))
  }
}
