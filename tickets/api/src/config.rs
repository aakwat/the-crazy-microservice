//! `Config` — settings from env and secrets from files, never from env; every problem is reported at once.

use std::fmt::Debug;
use std::net::SocketAddr;
use std::path::Path;
use std::str::FromStr;
use std::time::Duration;

use sqlx::postgres::PgConnectOptions;

/// The `aud` every service checks in access tokens (CLAUDE.md, identity).
pub const AUDIENCE: &str = "crazy-ms";
/// Listen address when `TICKETS_LISTEN_ADDR` is unset.
pub const DEFAULT_LISTEN_ADDR: &str = "0.0.0.0:8080";
/// Postgres port when `TICKETS_DB_PORT` is unset.
pub const DEFAULT_DB_PORT: &str = "5432";
/// Where secret files are mounted when `SECRETS_DIR` is unset.
pub const DEFAULT_SECRETS_DIR: &str = "/run/secrets";
/// Request bodies above this are refused before parsing; the largest valid ticket is far smaller.
pub const MAX_REQUEST_BYTES: usize = 64 * 1024;
/// Postgres connections each replica keeps open.
pub const MAX_DB_CONNECTIONS: u32 = 10;
/// How long one call to auth may take.
pub const AUTH_TIMEOUT: Duration = Duration::from_secs(5);
/// Signing keys older than this are refreshed.
pub const MAX_KEY_AGE: Duration = Duration::from_secs(600);
/// Signing keys are fetched at most this often.
pub const MIN_REFETCH_INTERVAL: Duration = Duration::from_secs(30);

pub struct Config {
  pub listen: SocketAddr,
  pub issuer: String,
  pub jwks_url: String,
  db_host: String,
  db_port: u16,
  db_name: String,
  app_user: String,
  app_password: String,
  migrator_user: String,
  migrator_password: String,
}

impl Config {
  pub fn load(getenv: impl Fn(&str) -> Option<String>) -> Result<Self, String> {
    let mut loader = Loader {
      getenv,
      problems: Vec::new(),
    };
    let secrets_dir = loader.optional("SECRETS_DIR", DEFAULT_SECRETS_DIR);
    let config = Self {
      listen: loader.parsed("TICKETS_LISTEN_ADDR", DEFAULT_LISTEN_ADDR),
      issuer: loader.required("AUTH_ISSUER"),
      jwks_url: loader.required("AUTH_JWKS_URL"),
      db_host: loader.required("TICKETS_DB_HOST"),
      db_port: loader.parsed("TICKETS_DB_PORT", DEFAULT_DB_PORT),
      db_name: loader.required("TICKETS_DB_NAME"),
      app_user: loader.required("TICKETS_DB_APP_USER"),
      app_password: loader.secret(&secrets_dir, "TICKETS_DB_APP_PASSWORD"),
      migrator_user: loader.required("TICKETS_DB_MIGRATOR_USER"),
      migrator_password: loader.secret(&secrets_dir, "TICKETS_DB_MIGRATOR_PASSWORD"),
    };
    if loader.problems.is_empty() {
      Ok(config)
    } else {
      Err(format!("invalid configuration: {}", loader.problems.join("; ")))
    }
  }

  /// The schema owner's connection: used once at startup to migrate.
  pub fn migrator_db(&self) -> PgConnectOptions {
    self.db(&self.migrator_user, &self.migrator_password)
  }

  /// The DML-only connection every request uses.
  pub fn app_db(&self) -> PgConnectOptions {
    self.db(&self.app_user, &self.app_password)
  }

  fn db(&self, user: &str, password: &str) -> PgConnectOptions {
    PgConnectOptions::new()
      .host(&self.db_host)
      .port(self.db_port)
      .database(&self.db_name)
      .username(user)
      .password(password)
  }
}

/// Collects every configuration problem instead of stopping at the first.
struct Loader<F> {
  getenv: F,
  problems: Vec<String>,
}

impl<F: Fn(&str) -> Option<String>> Loader<F> {
  fn optional(&self, name: &str, fallback: &str) -> String {
    (self.getenv)(name)
      .filter(|v| !v.is_empty())
      .unwrap_or_else(|| fallback.to_owned())
  }

  fn required(&mut self, name: &str) -> String {
    let value = (self.getenv)(name).unwrap_or_default();
    if value.is_empty() {
      self.problems.push(format!("{name} is not set"));
    }
    value
  }

  /// Falls back to the default, which is always valid, so loading can go on and report every problem.
  fn parsed<T: FromStr>(&mut self, name: &str, fallback: &str) -> T
  where
    T::Err: Debug,
  {
    let raw = self.optional(name, fallback);
    raw.parse().unwrap_or_else(|_| {
      self.problems.push(format!("{name} is not valid: {raw:?}"));
      fallback.parse().expect("defaults are valid")
    })
  }

  fn secret(&mut self, dir: &str, name: &str) -> String {
    let path = Path::new(dir).join(name);
    let value = std::fs::read_to_string(&path)
      .map(|v| v.trim().to_owned())
      .unwrap_or_default();
    if value.is_empty() {
      self.problems.push(format!("{} is missing or empty", path.display()));
    }
    value
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn every_missing_setting_is_reported_at_once() {
    let error = Config::load(|name| (name == "SECRETS_DIR").then(|| "/nonexistent".to_owned()))
      .err()
      .unwrap();

    for name in [
      "AUTH_ISSUER",
      "AUTH_JWKS_URL",
      "TICKETS_DB_HOST",
      "TICKETS_DB_APP_PASSWORD",
    ] {
      assert!(error.contains(name), "{name} missing from: {error}");
    }
  }
}
