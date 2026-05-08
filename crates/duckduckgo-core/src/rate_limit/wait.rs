//! Stateless helpers used by `runner.rs`: per-wait progress tracking,
//! one-sided sleep jitter, and the `Snapshot` projection of
//! `RateLimitState`. Keeping these out of `runner.rs` keeps that file
//! at file-cap size without having to inline 50+ lines of utility math
//! into the main loop.

use std::time::Duration;

use time::OffsetDateTime;

use super::outcome::Snapshot;
use super::state::RateLimitState;

/// Tracks the start time and initial total of a single wait window so
/// successive iterations of the run loop can report consistent
/// `elapsed` / `remaining` figures while the gate stays closed.
pub(super) struct WaitTracker {
    started_at: Option<OffsetDateTime>,
    total: Option<Duration>,
}

impl WaitTracker {
    pub(super) fn new() -> Self {
        Self {
            started_at: None,
            total: None,
        }
    }

    /// Initialize on first observation, then report `(elapsed, total)`
    /// against the *initial* total so the user sees a stable
    /// denominator across re-entries.
    pub(super) fn observe(
        &mut self,
        now: OffsetDateTime,
        remaining: Duration,
    ) -> (Duration, Duration) {
        let started = *self.started_at.get_or_insert(now);
        let total = *self.total.get_or_insert(remaining);
        let elapsed = (now - started).unsigned_abs();
        (elapsed, total)
    }
}

pub(super) fn snapshot_from_state(state: &RateLimitState) -> Snapshot {
    let format = time::format_description::well_known::Rfc3339;
    Snapshot {
        next_allowed_at: state.next_allowed_at.format(&format).ok(),
        blocked_until: state.blocked_until.and_then(|t| t.format(&format).ok()),
        slowdown_until: state.slowdown_until.and_then(|t| t.format(&format).ok()),
    }
}

/// Add a small positive offset (≤ `jitter`) to `base` so concurrent
/// processes wake at slightly different moments. We never *shorten* the
/// wait — that would defeat the gate — so the jitter is one-sided.
pub(super) fn with_positive_jitter(base: Duration, jitter: Duration) -> Duration {
    if jitter.is_zero() {
        return base;
    }
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| u128::from(d.subsec_nanos()))
        .unwrap_or(0);
    let pid = u128::from(std::process::id());
    let span = jitter.as_nanos().max(1);
    let offset = ((nanos.wrapping_mul(2_654_435_761) ^ pid) % span) as u64;
    base.saturating_add(Duration::from_nanos(offset))
}

#[cfg(test)]
mod tests {
    use super::{WaitTracker, with_positive_jitter};
    use std::time::Duration;
    use time::macros::datetime;

    #[test]
    fn wait_tracker_keeps_initial_total_stable() {
        let mut tracker = WaitTracker::new();
        let started = datetime!(2026-05-09 00:00 UTC);
        assert_eq!(
            tracker.observe(started, Duration::from_secs(10)),
            (Duration::ZERO, Duration::from_secs(10))
        );
        assert_eq!(
            tracker.observe(started + time::Duration::seconds(3), Duration::from_secs(7)),
            (Duration::from_secs(3), Duration::from_secs(10))
        );
    }

    #[test]
    fn positive_jitter_never_shortens_wait() {
        let base = Duration::from_millis(100);
        assert_eq!(with_positive_jitter(base, Duration::ZERO), base);
        let jittered = with_positive_jitter(base, Duration::from_millis(50));
        assert!(jittered >= base);
        assert!(jittered < base + Duration::from_millis(50));
    }
}
