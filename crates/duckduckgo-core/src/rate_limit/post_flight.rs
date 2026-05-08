//! Helper methods on [`RateLimiter`] that don't fit in `runner.rs`
//! without busting the per-file line cap: `wait_or_abort` (the actual
//! sleep dispatcher used by both wait branches), `emit_progress` (the
//! 1-second-threshold gate around the progress hook), and
//! `update_post_flight` (the state mutation after a request returns).

use std::cmp::max;
use std::time::Duration;

use time::OffsetDateTime;
use tokio::time::sleep;

use super::outcome::AttemptOutcome;
use super::progress::{RateLimitProgress, RateLimitWait};
use super::runner::{MAX_SLEEP_SECS, RateLimiter};
use super::wait::{WaitTracker, with_positive_jitter};
use crate::{Error, Result};

impl RateLimiter {
    /// Either error out (when `no_wait` is set) or sleep up to
    /// `MAX_SLEEP_SECS` against the current wait. The hook is fired
    /// once per call before the sleep when the *initial* total wait is
    /// ≥ 1 s.
    pub(super) async fn wait_or_abort(
        &self,
        kind: RateLimitWait,
        remaining: time::Duration,
        no_wait: bool,
        tracker: &mut WaitTracker,
        consecutive_blocks: u32,
    ) -> Result<()> {
        if no_wait {
            return Err(Error::Blocked(match kind {
                RateLimitWait::Cooldown => "Rate limit cooldown wait required".to_owned(),
                RateLimitWait::Spacing => "Rate limit spacing wait required".to_owned(),
            }));
        }
        let now = OffsetDateTime::now_utc();
        let remaining = remaining.unsigned_abs();
        let (elapsed, total) = tracker.observe(now, remaining);
        self.emit_progress(kind, elapsed, remaining, total, consecutive_blocks);
        let bounded = remaining.min(Duration::from_secs(MAX_SLEEP_SECS));
        sleep(with_positive_jitter(bounded, self.limits.jitter)).await;
        Ok(())
    }

    /// The 1-second threshold matches `docs/en/spec.md` §9.3: short
    /// spacing waits stay silent so default verbosity does not drown
    /// the user in `[INFO]` lines for routine sub-second gaps.
    fn emit_progress(
        &self,
        kind: RateLimitWait,
        elapsed: Duration,
        remaining: Duration,
        total: Duration,
        consecutive_blocks: u32,
    ) {
        if total < Duration::from_secs(1) {
            return;
        }
        let Some(hook) = self.progress_hook.as_ref() else {
            return;
        };
        hook(RateLimitProgress {
            kind,
            elapsed,
            remaining,
            total,
            consecutive_blocks,
        });
    }

    pub(super) fn update_post_flight(
        &self,
        outcome: AttemptOutcome,
        now: OffsetDateTime,
    ) -> Result<()> {
        let mut state = self.store.read_state();
        match outcome {
            AttemptOutcome::Success => {
                state.consecutive_blocks = 0;
                state.blocked_until = None;
                state.last_block_reason = None;
            }
            AttemptOutcome::Block(reason) => {
                state.consecutive_blocks = state.consecutive_blocks.saturating_add(1);
                let cooldown = self.limits.cooldown_for(state.consecutive_blocks);
                let blocked_until = now + cooldown;
                state.blocked_until = Some(blocked_until);
                state.next_allowed_at = max(state.next_allowed_at, blocked_until);
                let new_floor = now + self.limits.slowdown_duration;
                state.slowdown_until = Some(match state.slowdown_until {
                    Some(existing) => existing.max(new_floor),
                    None => new_floor,
                });
                state.last_block_reason = Some(reason.as_state_value().to_owned());
            }
            AttemptOutcome::Other => {}
        }
        self.store.write_state(&state)
    }
}
