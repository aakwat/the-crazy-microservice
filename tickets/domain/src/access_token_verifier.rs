//! `AccessTokenVerifier` — the port that turns a bearer token into a principal.

use crate::{Principal, TokenError};

#[allow(async_fn_in_trait)]
pub trait AccessTokenVerifier: Send + Sync {
  async fn verify(&self, token: &str) -> Result<Principal, TokenError>;
}
