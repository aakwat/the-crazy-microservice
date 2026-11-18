//! `MessageListResponse` — a ticket's conversation, oldest first.

use serde::Serialize;

use super::message_response::MessageResponse;

#[derive(Serialize)]
pub struct MessageListResponse {
  pub items: Vec<MessageResponse>,
}
