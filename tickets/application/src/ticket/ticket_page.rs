//! `TicketPage` — one page of a ticket list, and the paging rules every caller gets.

use domain::TicketSummary;
use uuid::Uuid;

/// Page size when the caller asks for none.
pub const DEFAULT_PAGE_SIZE: u32 = 20;
/// Largest page a caller may ask for.
pub const MAX_PAGE_SIZE: u32 = 100;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TicketPage {
  pub items: Vec<TicketSummary>,
  /// Pass back as `before` for the next page; `None` on the last one.
  pub next_before: Option<Uuid>,
}

impl TicketPage {
  /// The page size actually used for a requested one.
  pub(crate) fn size(requested: Option<u32>) -> u32 {
    requested.unwrap_or(DEFAULT_PAGE_SIZE).clamp(1, MAX_PAGE_SIZE)
  }

  /// A full page may have more after it; a short one is the last.
  pub(crate) fn new(items: Vec<TicketSummary>, size: u32) -> Self {
    let next_before = items
      .last()
      .filter(|_| items.len() == size as usize)
      .map(|last| last.id);
    Self { items, next_before }
  }
}
