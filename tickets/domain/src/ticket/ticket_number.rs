//! `TicketNumber` — the short number customers quote, shown as `T-1042` (docs/business-rules.md, TICKET-002).

use std::fmt;

/// One prefix for every ticket, so a reference reads the same in emails, the portal and on the phone.
const PREFIX: &str = "T-";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TicketNumber(u64);

impl TicketNumber {
  pub fn new(value: u64) -> Self {
    Self(value)
  }

  pub fn value(self) -> u64 {
    self.0
  }
}

impl fmt::Display for TicketNumber {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{PREFIX}{}", self.0)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn a_number_reads_as_a_short_reference() {
    assert_eq!(TicketNumber::new(1042).to_string(), "T-1042");
  }
}
