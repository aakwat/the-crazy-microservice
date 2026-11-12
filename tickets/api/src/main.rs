//! The tickets service binary: wiring, migrations, serving, the outbox relay and the container healthcheck.

use std::error::Error;
use std::net::SocketAddr;
use std::process::ExitCode;
use std::sync::Arc;
use std::time::Duration;

use api::config::{
  AUDIENCE, AUTH_TIMEOUT, Config, DEFAULT_LISTEN_ADDR, EVENTS_TIMEOUT, MAX_DB_CONNECTIONS, MAX_KEY_AGE,
  MIN_REFETCH_INTERVAL, OUTBOX_PURGE_INTERVAL, RELAY_DRAIN_TIMEOUT,
};
use api::interfaces::http::router;
use api::jobs::heartbeat::Heartbeat;
use api::jobs::outbox_relay::{self, TicketsOutboxRelay};
use api::jobs::outbox_retention;
use api::state::AppState;
use application::{OutboxRelay, OutboxRetention, TicketService};
use jwks::JwksVerifier;
use postgres::{PostgresOutbox, PostgresTicketRepository};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use streams::RedisEventPublisher;
use time::OffsetDateTime;
use tokio::net::TcpListener;
use tokio::signal::unix::{SignalKind, signal};
use tokio::sync::watch;
use tokio::task::{JoinError, JoinHandle};

#[tokio::main]
async fn main() -> ExitCode {
  if std::env::args().nth(1).as_deref() == Some("healthcheck") {
    return healthcheck().await;
  }
  tracing_subscriber::fmt().json().init();
  match run().await {
    Ok(()) => ExitCode::SUCCESS,
    Err(error) => {
      tracing::error!(event = "server.stopped", %error);
      ExitCode::FAILURE
    }
  }
}

async fn run() -> Result<(), Box<dyn Error>> {
  let config = Config::load(|name| std::env::var(name).ok())?;
  migrate(&config).await?;

  let pool = app_pool(&config).await?;
  let heartbeat = Arc::new(Heartbeat::new(OffsetDateTime::now_utc()));
  let state = AppState {
    tickets: Arc::new(TicketService::new(PostgresTicketRepository::new(pool.clone()))),
    verifier: Arc::new(verifier(&config)?),
    heartbeat: Arc::clone(&heartbeat),
  };

  let (stop_relay, relay_stopping) = watch::channel(false);
  let mut relay = outbox_relay::spawn(outbox_relay(&config, pool.clone())?, heartbeat, relay_stopping);
  let retention = OutboxRetention::new(PostgresOutbox::new(pool));
  let mut retention = outbox_retention::spawn(retention, OUTBOX_PURGE_INTERVAL);

  let listener = TcpListener::bind(config.listen).await?;
  tracing::info!(event = "server.started", addr = %config.listen);
  let server = axum::serve(listener, router(state)).with_graceful_shutdown(shutdown());

  // A job that ends before shutdown is a bug; exiting lets the container restart instead of serving without it.
  tokio::select! {
    result = server => result?,
    ended = &mut relay => return Err(stopped_early("outbox relay", &ended)),
    ended = &mut retention => return Err(stopped_early("outbox retention", &ended)),
  }

  drain(relay, &stop_relay).await;
  retention.abort();
  Ok(())
}

/// sqlx takes an advisory lock, so replicas starting together migrate once.
async fn migrate(config: &Config) -> Result<(), Box<dyn Error>> {
  let migrator = PgPoolOptions::new()
    .max_connections(1)
    .connect_with(config.migrator_db())
    .await?;
  postgres::migrate(&migrator).await?;
  migrator.close().await;
  Ok(())
}

async fn app_pool(config: &Config) -> Result<PgPool, sqlx::Error> {
  PgPoolOptions::new()
    .max_connections(MAX_DB_CONNECTIONS)
    .connect_with(config.app_db())
    .await
}

fn verifier(config: &Config) -> Result<JwksVerifier, reqwest::Error> {
  let http = reqwest::Client::builder()
    .timeout(AUTH_TIMEOUT)
    .redirect(reqwest::redirect::Policy::none())
    .build()?;
  Ok(JwksVerifier::new(
    http,
    config.jwks_url.clone(),
    &config.issuer,
    AUDIENCE,
    MAX_KEY_AGE,
    MIN_REFETCH_INTERVAL,
  ))
}

fn outbox_relay(config: &Config, pool: PgPool) -> Result<TicketsOutboxRelay, Box<dyn Error>> {
  let publisher = RedisEventPublisher::new(config.events_bus(), EVENTS_TIMEOUT)?;
  Ok(OutboxRelay::new(PostgresOutbox::new(pool), publisher))
}

fn stopped_early(job: &str, ended: &Result<(), JoinError>) -> Box<dyn Error> {
  format!("{job} stopped unexpectedly: {ended:?}").into()
}

/// HTTP has drained, so no request adds outbox rows any more; the relay finishes its round and stops.
async fn drain(relay: JoinHandle<()>, stop: &watch::Sender<bool>) {
  let _ = stop.send(true);
  if tokio::time::timeout(RELAY_DRAIN_TIMEOUT, relay).await.is_err() {
    tracing::warn!(
      event = "outbox.relay.drain_timeout",
      timeout_secs = RELAY_DRAIN_TIMEOUT.as_secs()
    );
  }
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
