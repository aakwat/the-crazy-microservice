//! The tickets service binary: wiring, migrations, serving and the container healthcheck.

use std::net::SocketAddr;
use std::process::ExitCode;
use std::sync::Arc;
use std::time::Duration;

use api::config::{
  AUDIENCE, AUTH_TIMEOUT, Config, DEFAULT_LISTEN_ADDR, MAX_DB_CONNECTIONS, MAX_KEY_AGE, MIN_REFETCH_INTERVAL,
};
use api::interfaces::http::router;
use api::state::AppState;
use application::TicketService;
use jwks::JwksVerifier;
use postgres::PostgresTicketRepository;
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;
use tokio::signal::unix::{SignalKind, signal};

#[tokio::main]
async fn main() -> ExitCode {
  if std::env::args().nth(1).as_deref() == Some("healthcheck") {
    return healthcheck().await;
  }
  tracing_subscriber::fmt().json().init();
  match run().await {
    Ok(()) => ExitCode::SUCCESS,
    Err(error) => {
      tracing::error!(%error, "tickets stopped");
      ExitCode::FAILURE
    }
  }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
  let config = Config::load(|name| std::env::var(name).ok())?;

  // sqlx takes an advisory lock, so replicas starting together migrate once.
  let migrator = PgPoolOptions::new()
    .max_connections(1)
    .connect_with(config.migrator_db())
    .await?;
  postgres::migrate(&migrator).await?;
  migrator.close().await;

  let pool = PgPoolOptions::new()
    .max_connections(MAX_DB_CONNECTIONS)
    .connect_with(config.app_db())
    .await?;
  let http = reqwest::Client::builder()
    .timeout(AUTH_TIMEOUT)
    .redirect(reqwest::redirect::Policy::none())
    .build()?;
  let verifier = JwksVerifier::new(
    http,
    config.jwks_url,
    &config.issuer,
    AUDIENCE,
    MAX_KEY_AGE,
    MIN_REFETCH_INTERVAL,
  );
  let state = AppState {
    tickets: Arc::new(TicketService::new(PostgresTicketRepository::new(pool))),
    verifier: Arc::new(verifier),
  };

  let listener = TcpListener::bind(config.listen).await?;
  tracing::info!(addr = %config.listen, "tickets listening");
  axum::serve(listener, router(state))
    .with_graceful_shutdown(shutdown())
    .await?;
  Ok(())
}

async fn shutdown() {
  let Ok(mut terminate) = signal(SignalKind::terminate()) else {
    return std::future::pending().await;
  };
  tokio::select! {
    _ = tokio::signal::ctrl_c() => {}
    _ = terminate.recv() => {}
  }
}

/// The container probe; the distroless image has no curl, so the binary checks itself.
async fn healthcheck() -> ExitCode {
  let listen = std::env::var("TICKETS_LISTEN_ADDR").unwrap_or_else(|_| DEFAULT_LISTEN_ADDR.to_owned());
  let Ok(port) = listen.parse::<SocketAddr>().map(|addr| addr.port()) else {
    return ExitCode::FAILURE;
  };
  let client = reqwest::Client::builder().timeout(Duration::from_secs(2)).build();
  let healthy = match client {
    Ok(client) => client
      .get(format!("http://127.0.0.1:{port}/healthz"))
      .send()
      .await
      .is_ok_and(|r| r.status().is_success()),
    Err(_) => false,
  };
  if healthy { ExitCode::SUCCESS } else { ExitCode::FAILURE }
}
