//! `GET /api/tickets` — the caller's visible tickets, newest first, one page at a time.

use axum::Json;
use axum::extract::rejection::QueryRejection;
use axum::extract::{Query, State};
use serde::Deserialize;
use uuid::Uuid;

use super::ticket_page_response::TicketPageResponse;
use crate::interfaces::http::Problem;
use crate::interfaces::http::extract::Authenticated;
use crate::state::AppState;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ListQuery {
  before: Option<Uuid>,
  limit: Option<u32>,
}

pub async fn list_tickets(
  State(state): State<AppState>,
  Authenticated(who): Authenticated,
  query: Result<Query<ListQuery>, QueryRejection>,
) -> Result<Json<TicketPageResponse>, Problem> {
  let Query(query) = query.map_err(|rejection| Problem::new(rejection.status(), rejection.body_text()))?;
  let page = state.tickets.list(&who, query.before, query.limit).await?;
  Ok(Json(page.into()))
}
