//! `StaffView` — the fields only staff see; customers deal with the company, never with a named agent.

use domain::Principal;
use serde::Serialize;
use uuid::Uuid;

/// Flattened into ticket responses; absent for customers (docs/business-rules.md, P1).
#[derive(Serialize)]
pub struct StaffView {
  assignee_id: Option<Uuid>,
}

impl StaffView {
  pub fn for_viewer(who: &Principal, assignee_id: Option<Uuid>) -> Option<Self> {
    who.is_agent().then_some(Self { assignee_id })
  }
}
