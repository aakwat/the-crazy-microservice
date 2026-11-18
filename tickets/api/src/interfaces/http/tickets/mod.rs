//! `/api/tickets`: one file per handler, plus the response shapes.

mod assign_ticket;
mod confirm_ticket;
mod create_ticket;
mod get_ticket;
mod list_messages;
mod list_tickets;
mod message_list_response;
mod message_response;
mod message_staff_view;
mod post_message;
mod staff_view;
mod ticket_page_response;
mod ticket_response;
mod ticket_summary_response;
mod withdraw_ticket;

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
      get(get_ticket::get_ticket).patch(assign_ticket::assign_ticket),
    )
    .route(
      "/api/tickets/{id}/messages",
      get(list_messages::list_messages).post(post_message::post_message),
    )
    .route("/api/tickets/{id}/confirm", post(confirm_ticket::confirm_ticket))
    .route("/api/tickets/{id}/withdraw", post(withdraw_ticket::withdraw_ticket))
}
