//! `JwksVerifier` — validates access tokens as RFC 9068 §4 asks, against auth's cached signing keys.

use std::collections::HashMap;
use std::time::Duration;

use domain::{AccessTokenVerifier, Principal, TokenError};
use jsonwebtoken::jwk::{JwkSet, PublicKeyUse};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::access_claims::AccessClaims;
use crate::key_cache::KeyCache;

/// RFC 9068 §4: the `typ` header that marks an access token, as opposed to an ID token.
const ACCESS_TOKEN_TYPES: [&str; 2] = ["at+jwt", "application/at+jwt"];

pub struct JwksVerifier {
  http: reqwest::Client,
  jwks_url: String,
  validation: Validation,
  /// Keys older than this are refreshed; while auth is down they serve for twice as long, then stop.
  max_key_age: Duration,
  /// Fetches are tried at most this often, so unknown kids or an outage cannot flood auth.
  min_refetch_interval: Duration,
  cache: RwLock<KeyCache>,
}

impl JwksVerifier {
  /// The client should carry a timeout and follow no redirects; signing is pinned to ES256.
  pub fn new(
    http: reqwest::Client,
    jwks_url: String,
    issuer: &str,
    audience: &str,
    max_key_age: Duration,
    min_refetch_interval: Duration,
  ) -> Self {
    let mut validation = Validation::new(Algorithm::ES256);
    validation.set_issuer(&[issuer]);
    validation.set_audience(&[audience]);
    validation.set_required_spec_claims(&["exp", "iss", "aud", "sub"]);
    Self {
      http,
      jwks_url,
      validation,
      max_key_age,
      min_refetch_interval,
      cache: RwLock::default(),
    }
  }

  async fn key(&self, kid: &str) -> Result<DecodingKey, TokenError> {
    if let Some(key) = self.cache.read().await.get_younger_than(kid, self.max_key_age) {
      return Ok(key);
    }
    let stale = {
      let mut cache = self.cache.write().await;
      if let Some(key) = cache.get_younger_than(kid, self.max_key_age) {
        return Ok(key);
      }
      let stale = cache.get_younger_than(kid, self.max_key_age * 2);
      if cache.attempted_within(self.min_refetch_interval) {
        return stale.ok_or_else(unknown_key);
      }
      cache.mark_attempt();
      stale
    };
    // Fetched without holding the lock, so requests whose keys are fresh never wait on auth.
    match self.fetch().await {
      Ok(keys) => {
        let mut cache = self.cache.write().await;
        cache.replace(keys);
        cache.get(kid).ok_or_else(unknown_key)
      }
      Err(error) => stale.ok_or(error),
    }
  }

  async fn fetch(&self) -> Result<HashMap<String, DecodingKey>, TokenError> {
    let unavailable = |e: reqwest::Error| TokenError::Unavailable(e.to_string());
    let set: JwkSet = self
      .http
      .get(&self.jwks_url)
      .send()
      .await
      .and_then(reqwest::Response::error_for_status)
      .map_err(unavailable)?
      .json()
      .await
      .map_err(unavailable)?;
    let signing_keys = set
      .keys
      .iter()
      .filter(|jwk| jwk.common.public_key_use != Some(PublicKeyUse::Encryption));
    let keys: HashMap<_, _> = signing_keys
      .filter_map(|jwk| Some((jwk.common.key_id.clone()?, DecodingKey::from_jwk(jwk).ok()?)))
      .collect();
    if keys.is_empty() {
      return Err(TokenError::Unavailable(
        "auth published no usable signing key".to_owned(),
      ));
    }
    Ok(keys)
  }
}

impl AccessTokenVerifier for JwksVerifier {
  async fn verify(&self, token: &str) -> Result<Principal, TokenError> {
    let invalid = |e: &dyn std::fmt::Display| TokenError::Invalid(e.to_string());
    let header = decode_header(token).map_err(|e| invalid(&e))?;
    let typ = header.typ.as_deref().unwrap_or_default();
    if !ACCESS_TOKEN_TYPES
      .iter()
      .any(|accepted| typ.eq_ignore_ascii_case(accepted))
    {
      return Err(TokenError::Invalid("typ must be at+jwt".to_owned()));
    }
    let kid = header
      .kid
      .ok_or_else(|| TokenError::Invalid("token has no kid".to_owned()))?;
    let key = self.key(&kid).await?;
    let claims = decode::<AccessClaims>(token, &key, &self.validation)
      .map_err(|e| invalid(&e))?
      .claims;
    let subject = Uuid::parse_str(&claims.sub).map_err(|e| invalid(&e))?;
    Ok(Principal::new(subject, claims.roles))
  }
}

fn unknown_key() -> TokenError {
  TokenError::Invalid("unknown signing key".to_owned())
}
