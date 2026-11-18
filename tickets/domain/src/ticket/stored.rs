//! What Postgres can store, checked here so a bad value fails as a rule, never as a server error.

use time::OffsetDateTime;

/// Also refuses NUL, which Postgres rejects in text.
pub(super) fn fits(text: &str, max_chars: usize) -> bool {
  !text.is_empty() && !text.contains('\0') && text.chars().count() <= max_chars
}

/// Postgres keeps microseconds; matching it keeps `updated_at` exact for the stale-write check.
pub(super) fn stored_precision(time: OffsetDateTime) -> OffsetDateTime {
  time
    .replace_nanosecond(time.nanosecond() / 1_000 * 1_000)
    .unwrap_or(time)
}
