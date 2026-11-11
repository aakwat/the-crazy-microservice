//! JWKS adapter: verifies auth's RFC 9068 access tokens against its published signing keys.

mod access_claims;
mod jwks_verifier;
mod key_cache;

pub use jwks_verifier::JwksVerifier;
