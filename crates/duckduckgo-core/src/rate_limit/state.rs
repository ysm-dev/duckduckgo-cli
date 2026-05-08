use std::time::Duration;

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// Persistent state for the rate-limit guard. Schema 2.
///
/// Replaces the v0.1.0-pre token bucket. The new fields encode a
/// *minimum-spacing* gate plus a 10-minute slowdown ratchet that is
/// triggered on any observed block. See `docs/en/spec.md` §8 for the
/// algorithm and `docs/en/ddgr.md` for the empirical basis.
#[derive(Clone, Debug, Serialize)]
pub struct RateLimitState {
    pub schema: u8,
    #[serde(with = "time::serde::rfc3339")]
    pub next_allowed_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339::option")]
    pub blocked_until: Option<OffsetDateTime>,
    #[serde(with = "time::serde::rfc3339::option")]
    pub slowdown_until: Option<OffsetDateTime>,
    pub consecutive_blocks: u32,
    pub last_block_reason: Option<String>,
}

impl Default for RateLimitState {
    fn default() -> Self {
        Self {
            schema: 2,
            next_allowed_at: OffsetDateTime::now_utc(),
            blocked_until: None,
            slowdown_until: None,
            consecutive_blocks: 0,
            last_block_reason: None,
        }
    }
}

impl RateLimitState {
    /// If the persisted state is corrupted (timestamps absurdly far in the
    /// future), reset to defaults so the guard can never wedge a process.
    /// "Absurd" is anything more than 24 h ahead of the current clock.
    pub fn sanitize(&mut self, now: OffsetDateTime) {
        let cap = now + Duration::from_secs(24 * 60 * 60);
        if self.next_allowed_at > cap {
            *self = Self::default();
            self.next_allowed_at = now;
            return;
        }
        if self.blocked_until.is_some_and(|t| t > cap) {
            self.blocked_until = None;
        }
        if self.slowdown_until.is_some_and(|t| t > cap) {
            self.slowdown_until = None;
        }
    }
}

/// On-disk representation. Both schema 1 (token bucket) and schema 2 are
/// accepted; schema 1 fields that are no longer meaningful (`tokens`,
/// `last_refill`) are dropped. The parsed result is always a schema-2
/// `RateLimitState`; the next write will normalise the file.
#[derive(Deserialize)]
struct RawState {
    /// Discriminator we accept and discard. Schema 1 carries `tokens`
    /// and `last_refill`; schema 2 carries `next_allowed_at` and
    /// `slowdown_until`. We treat both alike, taking only the fields the
    /// new layout needs and defaulting the rest.
    #[serde(default, rename = "schema")]
    _schema: u8,
    #[serde(default, with = "time::serde::rfc3339::option")]
    next_allowed_at: Option<OffsetDateTime>,
    #[serde(default, with = "time::serde::rfc3339::option")]
    blocked_until: Option<OffsetDateTime>,
    #[serde(default, with = "time::serde::rfc3339::option")]
    slowdown_until: Option<OffsetDateTime>,
    #[serde(default)]
    consecutive_blocks: u32,
    #[serde(default)]
    last_block_reason: Option<String>,
}

impl<'de> Deserialize<'de> for RateLimitState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Schema 1 had no `next_allowed_at`; if that field is absent we
        // default to `now`, which means the next request is allowed
        // immediately (no carry-over of unused tokens).
        let raw = RawState::deserialize(deserializer)?;
        Ok(Self {
            schema: 2,
            next_allowed_at: raw.next_allowed_at.unwrap_or_else(OffsetDateTime::now_utc),
            blocked_until: raw.blocked_until,
            slowdown_until: raw.slowdown_until,
            consecutive_blocks: raw.consecutive_blocks,
            last_block_reason: raw.last_block_reason,
        })
    }
}

#[cfg(test)]
#[path = "state_tests.rs"]
mod tests;
