//! `AccessClaims` — the access-token claims tickets reads; iss, aud and exp are checked by the validator.

#[derive(serde::Deserialize)]
pub(crate) struct AccessClaims {
  /// The account id; service tokens carry a client id instead and are refused here.
  pub(crate) sub: String,
  /// Present for users only.
  #[serde(default)]
  pub(crate) roles: Vec<String>,
  /// RFC 9068 §2.2 requires it; a token without it is not one of auth's access tokens.
  #[serde(rename = "client_id")]
  _client_id: String,
}
