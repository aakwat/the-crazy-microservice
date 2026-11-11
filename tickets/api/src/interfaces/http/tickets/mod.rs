//! `/api/tickets`: one file per handler, plus the response shapes.

mod change_ticket;
mod create_ticket;
mod get_ticket;
mod list_tickets;
mod ticket_page_response;
mod ticket_response;
mod ticket_summary_response;

use axum::Router;
use axum::routing::{get, post};

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
  Router::new()
    .route(
      "/api/tickets",
      post(create_ticket::create_ticket).get(list_tickets::list_tickets),
    )
    .route(
      "/api/tickets/{id}",
      get(get_ticket::get_ticket).patch(change_ticket::change_ticket),
    )
}
