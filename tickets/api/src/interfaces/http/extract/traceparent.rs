//! `Traceparent` — the W3C trace context the gateway started, kept only when well-formed.

use std::convert::Infallible;

use axum::extract::FromRequestParts;
use axum::http::request::Parts;

pub struct Traceparent(pub Option<String>);

impl<S: Send + Sync> FromRequestParts<S> for Traceparent {
  type Rejection = Infallible;

  fn from_request_parts(parts: &mut Parts, _state: &S) -> impl Future<Output = Result<Self, Infallible>> + Send {
    let value = parts.headers.get("traceparent").and_then(|value| value.to_str().ok());
    std::future::ready(Ok(Self(value.filter(|value| is_valid(value)).map(str::to_owned))))
  }
}

/// W3C Trace Context §3.2, version 00: an all-zero trace or span id is invalid.
fn is_valid(value: &str) -> bool {
  let parts: Vec<&str> = value.split('-').collect();
  matches!(parts.as_slice(), [version, trace, span, flags]
    if *version == "00" && hex(trace, 32) && hex(span, 16) && hex(flags, 2) && non_zero(trace) && non_zero(span))
}

fn hex(part: &str, len: usize) -> bool {
  part.len() == len && part.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn non_zero(part: &str) -> bool {
  part.bytes().any(|b| b != b'0')
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn only_well_formed_version_00_values_pass() {
    assert!(is_valid("00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01"));
    for bad in [
      "",
      "01-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01",
      "00-0AF7651916CD43DD8448EB211C80319C-b7ad6b7169203331-01",
      "00-00000000000000000000000000000000-b7ad6b7169203331-01",
      "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331",
      "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01-extra",
    ] {
      assert!(!is_valid(bad), "{bad:?} must be refused");
    }
  }
}
