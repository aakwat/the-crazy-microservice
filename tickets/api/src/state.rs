//! `AppState` — what every handler shares: the use cases and the token verifier.

use std::sync::Arc;

use application::TicketService;
use jwks::JwksVerifier;
use postgres::PostgresTicketRepository;

#[derive(Clone)]
pub struct AppState {
  pub tickets: Arc<TicketService<PostgresTicketRepository>>,
  pub verifier: Arc<JwksVerifier>,
}
