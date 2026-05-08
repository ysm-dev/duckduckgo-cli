//! `RateLimiter::run`: acquire / release loop. Helper methods live in
//! `post_flight.rs` to keep this file under the per-file line cap.

use std::future::Future;
use std::path::PathBuf;
use std::time::Duration;

use fd_lock::RwLock;
use time::OffsetDateTime;
use tokio::time::sleep;

use super::config;
use super::outcome::{AttemptOutcome, RunResult, Snapshot};
use super::progress::{ProgressHook, RateLimitWait};
use super::store::StateStore;
use super::wait::{WaitTracker, snapshot_from_state};
use crate::Result;

/// Lock-retry interval. Per-cycle sleep cap (`MAX_SLEEP_SECS`) bounds
/// a *single* sleep so the loop wakes regularly enough to fire the
/// progress hook on long cooldowns; iterating still reaches the full
/// cooldown.
const LOCK_RETRY_MS: u64 = 50;
pub(super) const MAX_SLEEP_SECS: u64 = 80;

// `Debug` is intentionally not implemented: `ProgressHook` is an
// `Arc<dyn Fn>`, which has no useful representation, and no caller in
// the workspace formats the limiter directly.
#[derive(Clone)]
pub struct RateLimiter {
    pub(super) store: StateStore,
    pub(super) limits: config::Limits,
    pub(super) progress_hook: Option<ProgressHook>,
}

impl RateLimiter {
    pub fn new(state_dir: PathBuf, proxy: Option<&str>) -> Self {
        Self::with_limits(state_dir, proxy, config::limits())
    }

    pub(crate) fn with_limits(
        state_dir: PathBuf,
        proxy: Option<&str>,
        limits: config::Limits,
    ) -> Self {
        Self {
            store: StateStore::new(state_dir, proxy),
            limits,
            progress_hook: None,
        }
    }

    /// Install a callback the limiter invokes before each cooldown /
    /// spacing sleep. Replaces any previously set hook.
    #[must_use]
    pub fn with_progress_hook(mut self, hook: Option<ProgressHook>) -> Self {
        self.progress_hook = hook;
        self
    }

    /// Run a single outbound DDG request through the gate. The closure
    /// is invoked exactly once after the limiter has waited for any
    /// cooldown or spacing window. The cross-process lock is held for
    /// the full duration of the closure (preflight + HTTP + post-flight)
    /// and released when this function returns.
    ///
    /// Returns `Err(Error::Blocked)` when `no_wait` is set and a wait
    /// would otherwise be required.
    pub async fn run<F, Fut, T>(&self, no_wait: bool, f: F) -> Result<RunResult<T>>
    where
        F: FnOnce(Snapshot) -> Fut,
        Fut: Future<Output = (AttemptOutcome, T)>,
    {
        let mut f_slot = Some(f);
        let mut cooldown = WaitTracker::new();
        let mut spacing = WaitTracker::new();
        loop {
            let file = self.store.open_lock_file()?;
            let mut lock = RwLock::new(file);
            let Ok(_guard) = lock.try_write() else {
                sleep(Duration::from_millis(LOCK_RETRY_MS)).await;
                continue;
            };

            let mut state = self.store.read_state();
            let now = OffsetDateTime::now_utc();
            let consecutive = state.consecutive_blocks;

            if let Some(until) = state.blocked_until
                && until > now
            {
                self.store.write_state(&state)?;
                drop(_guard);
                drop(lock);
                let dur = until - now;
                self.wait_or_abort(
                    RateLimitWait::Cooldown,
                    dur,
                    no_wait,
                    &mut cooldown,
                    consecutive,
                )
                .await?;
                continue;
            }
            if state.blocked_until.is_some() {
                state.blocked_until = None;
                self.store.write_state(&state)?;
            }
            if state.next_allowed_at > now {
                let dur = state.next_allowed_at - now;
                self.store.write_state(&state)?;
                drop(_guard);
                drop(lock);
                self.wait_or_abort(
                    RateLimitWait::Spacing,
                    dur,
                    no_wait,
                    &mut spacing,
                    consecutive,
                )
                .await?;
                continue;
            }

            // Slot is ours. Advance the gate, persist, then call the
            // request closure with the lock still held.
            let advance = self.limits.spacing(state.slowdown_until);
            state.next_allowed_at = now + advance;
            self.store.write_state(&state)?;
            let preflight = snapshot_from_state(&state);

            let f_taken = f_slot
                .take()
                .expect("FnOnce taken exactly once on the proceed path");
            let (outcome, value) = f_taken(preflight.clone()).await;

            self.update_post_flight(outcome, OffsetDateTime::now_utc())?;
            let final_state = self.store.read_state();
            return Ok(RunResult {
                value,
                snapshot: snapshot_from_state(&final_state),
                outcome,
            });
        }
    }
}
