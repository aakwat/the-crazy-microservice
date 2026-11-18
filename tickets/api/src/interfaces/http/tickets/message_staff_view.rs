//! `MessageStaffView` — the message fields only staff see; customers never learn which agent wrote a reply.

use domain::{Message, Principal, Visibility};
use serde::Serialize;
use uuid::Uuid;

/// Flattened into message responses; absent for customers.
#[derive(Serialize)]
pub struct MessageStaffView {
  internal: bool,
  author_id: Uuid,
}

impl MessageStaffView {
  pub fn for_viewer(who: &Principal, message: &Message) -> Option<Self> {
    who.is_agent().then(|| Self {
      internal: message.visibility() == Visibility::Internal,
      author_id: message.author_id(),
    })
  }
}
