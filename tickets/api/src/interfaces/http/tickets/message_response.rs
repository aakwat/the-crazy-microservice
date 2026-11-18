//! `MessageResponse` — the wire shape of one message; staff replies are signed by the company.

use domain::{Author, Message, Principal};
use serde::Serialize;
use time::OffsetDateTime;
use uuid::Uuid;

use super::message_staff_view::MessageStaffView;

#[derive(Serialize)]
pub struct MessageResponse {
  id: Uuid,
  from: &'static str,
  body: String,
  #[serde(with = "time::serde::rfc3339")]
  created_at: OffsetDateTime,
  #[serde(flatten)]
  staff: Option<MessageStaffView>,
}

impl MessageResponse {
  pub fn for_viewer(message: &Message, who: &Principal) -> Self {
    let from = match message.author() {
      Author::Customer => "customer",
      Author::Staff => "support",
    };
    Self {
      id: message.id(),
      from,
      body: message.body().to_owned(),
      created_at: message.created_at(),
      staff: MessageStaffView::for_viewer(who, message),
    }
  }
}
