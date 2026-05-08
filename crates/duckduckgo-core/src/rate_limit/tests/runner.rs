//! Smoke tests for [`RateLimiter`]'s single-process happy paths:
//! gate advancement on success, cooldown / slowdown arming on a block,
//! `--no-wait` abort behaviour, and recovery after a wait elapses.
//! Loaded via `#[path]` in `mod.rs` so it shares the crate's test
//! visibility with `rate_limit::config::Limits::test_fast`.

use std::sync::Arc;
use std::time::Duration;

use tempfile::TempDir;
use time::OffsetDateTime;

use crate::parser::BlockReason;
use crate::rate_limit::config::Limits;
use crate::rate_limit::wait::snapshot_from_state;
use crate::rate_limit::{AttemptOutcome, RateLimitState, RateLimiter};
use crate::{Error, ManualClock, SystemClock};

fn fast_limiter(dir: &TempDir) -> RateLimiter {
    RateLimiter::new(
        dir.path().to_path_buf(),
        None,
        Limits::test_fast(150, 300, 1),
        Arc::new(SystemClock),
    )
}

fn manual_fast_limiter(dir: &TempDir) -> (RateLimiter, Arc<ManualClock>) {
    let clock = Arc::new(ManualClock::new(OffsetDateTime::now_utc()));
    let limiter = RateLimiter::new(
        dir.path().to_path_buf(),
        None,
        Limits::test_fast(150, 300, 1),
        clock.clone(),
    );
    (limiter, clock)
}

#[test]
fn snapshot_from_default_state_serializes_only_next_allowed_at() {
    let state = RateLimitState::default();
    let snap = snapshot_from_state(&state);
    assert!(snap.next_allowed_at.is_some());
    assert!(snap.blocked_until.is_none());
    assert!(snap.slowdown_until.is_none());
}

#[tokio::test(flavor = "current_thread")]
async fn run_success_advances_next_allowed_at_by_spacing() {
    let dir = TempDir::new().unwrap();
    let (limiter, clock) = manual_fast_limiter(&dir);
    let started = clock.now();
    let result = limiter
        .run(false, |snap| async move { (AttemptOutcome::Success, snap) })
        .await
        .unwrap();
    assert!(matches!(result.outcome, AttemptOutcome::Success));
    let next = result.snapshot.next_allowed_at.expect("next_allowed_at");
    let parsed =
        time::OffsetDateTime::parse(&next, &time::format_description::well_known::Rfc3339).unwrap();
    let advance = (parsed - started).unsigned_abs();
    assert!(
        advance >= Duration::from_millis(140) && advance < Duration::from_secs(2),
        "expected ~150ms advance, got {advance:?}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn run_block_sets_cooldown_and_slowdown_window() {
    let dir = TempDir::new().unwrap();
    let (limiter, _clock) = manual_fast_limiter(&dir);
    let result = limiter
        .run(true, |snap| async move {
            (AttemptOutcome::Block(BlockReason::Http202), snap)
        })
        .await
        .unwrap();
    assert!(matches!(
        result.outcome,
        AttemptOutcome::Block(BlockReason::Http202)
    ));
    assert!(result.snapshot.blocked_until.is_some());
    assert!(result.snapshot.slowdown_until.is_some());
}

#[tokio::test(flavor = "current_thread")]
async fn run_block_then_no_wait_returns_blocked() {
    let dir = TempDir::new().unwrap();
    let limiter = fast_limiter(&dir);
    let _ = limiter
        .run(true, |snap| async move {
            (AttemptOutcome::Block(BlockReason::Http202), snap)
        })
        .await
        .unwrap();
    let err = limiter
        .run(true, |_| async move { (AttemptOutcome::Success, ()) })
        .await
        .err()
        .expect("should be blocked");
    assert!(matches!(err, Error::Blocked(_)));
}

#[tokio::test(flavor = "current_thread")]
async fn block_then_clean_run_resets_consecutive_blocks() {
    let dir = TempDir::new().unwrap();
    let (limiter, clock) = manual_fast_limiter(&dir);
    let _ = limiter
        .run(false, |snap| async move {
            (AttemptOutcome::Block(BlockReason::Http202), snap)
        })
        .await
        .unwrap();
    clock.advance(Duration::from_millis(1_100));
    let result = limiter
        .run(false, |snap| async move { (AttemptOutcome::Success, snap) })
        .await
        .unwrap();
    assert!(matches!(result.outcome, AttemptOutcome::Success));
    assert!(result.snapshot.blocked_until.is_none());
    // slowdown_until should still be set (10s after block).
    assert!(result.snapshot.slowdown_until.is_some());
}
