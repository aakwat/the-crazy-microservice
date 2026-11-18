//! Column conversions shared by the ticket rows: a stored value the domain cannot accept is a store error.

use domain::{Author, CloseReason, StoreError, TicketNumber, TicketStatus, Visibility};

pub(crate) fn parse_status(name: &str) -> Result<TicketStatus, StoreError> {
  TicketStatus::parse(name).ok_or_else(|| corrupt(format!("unknown ticket status {name:?}")))
}

pub(crate) fn parse_close_reason(name: &str) -> Result<CloseReason, StoreError> {
  CloseReason::parse(name).ok_or_else(|| corrupt(format!("unknown close reason {name:?}")))
}

pub(crate) fn parse_author(name: &str) -> Result<Author, StoreError> {
  Author::parse(name).ok_or_else(|| corrupt(format!("unknown author {name:?}")))
}

pub(crate) fn parse_visibility(name: &str) -> Result<Visibility, StoreError> {
  Visibility::parse(name).ok_or_else(|| corrupt(format!("unknown visibility {name:?}")))
}

pub(crate) fn parse_count(stored: i32) -> Result<u32, StoreError> {
  u32::try_from(stored).map_err(|_| corrupt(format!("bad reopen count {stored}")))
}

pub(crate) fn stored_count(count: u32) -> Result<i32, StoreError> {
  i32::try_from(count).map_err(|_| corrupt(format!("reopen count {count} too large")))
}

pub(crate) fn parse_number(stored: i64) -> Result<TicketNumber, StoreError> {
  u64::try_from(stored)
    .map(TicketNumber::new)
    .map_err(|_| corrupt(format!("bad ticket number {stored}")))
}

pub(crate) fn stored_number(number: TicketNumber) -> Result<i64, StoreError> {
  i64::try_from(number.value()).map_err(|_| corrupt(format!("ticket number {number} too large")))
}

fn corrupt(reason: String) -> StoreError {
  StoreError(reason.into())
}
