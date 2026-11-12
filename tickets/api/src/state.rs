//! `AppState` — what every handler shares: the use cases, the token verifier and the relay heartbeat.

use std::sync::Arc;

use application::TicketService;
use jwks::JwksVerifier;
use postgres::PostgresTicketRepository;

use crate::jobs::heartbeat::Heartbeat;

#[derive(Clone)]
pub struct AppState {
  pub tickets: Arc<TicketService<PostgresTicketRepository>>,
  pub verifier: Arc<JwksVerifier>,
  pub heartbeat: Arc<Heartbeat>,
}
