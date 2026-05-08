use std::sync::Arc;
use std::time::Duration;

use tempfile::TempDir;
use time::macros::datetime;

use crate::ManualClock;
use crate::parser::BlockReason;
use crate::rate_limit::config::Limits;
use crate::rate_limit::wait::WaitTracker;
use crate::rate_limit::{AttemptOutcome, RateLimitWait, RateLimiter};

fn limiter(dir: &TempDir) -> RateLimiter {
    RateLimiter::new(
        dir.path().to_path_buf(),
        None,
        Limits::test_fast(100, 200, 2),
        Arc::new(ManualClock::new(datetime!(2026-05-09 00:00 UTC))),
    )
}

#[test]
fn block_ratchets_slowdown_until_forward() {
    let dir = TempDir::new().unwrap();
    let limiter = limiter(&dir);
    let first = datetime!(2026-05-09 00:00 UTC);
    let second = first + time::Duration::seconds(5);
    limiter
        .update_post_flight(AttemptOutcome::Block(BlockReason::Http202), first)
        .unwrap();
    let original = limiter.store.read_state(first).slowdown_until.unwrap();
    limiter
        .update_post_flight(AttemptOutcome::Block(BlockReason::Http429), second)
        .unwrap();
    let updated = limiter.store.read_state(second).slowdown_until.unwrap();
    assert!(updated > original);
}

#[test]
fn success_clears_block_metadata() {
    let dir = TempDir::new().unwrap();
    let limiter = limiter(&dir);
    let now = datetime!(2026-05-09 00:00 UTC);
    limiter
        .update_post_flight(AttemptOutcome::Block(BlockReason::Http202), now)
        .unwrap();
    limiter
        .update_post_flight(AttemptOutcome::Success, now + time::Duration::seconds(3))
        .unwrap();
    let state = limiter.store.read_state(now + time::Duration::seconds(3));
    assert_eq!(state.consecutive_blocks, 0);
    assert!(state.blocked_until.is_none());
    assert!(state.last_block_reason.is_none());
}

#[tokio::test(flavor = "current_thread")]
async fn wait_or_abort_no_wait_returns_blocked_without_sleep() {
    let dir = TempDir::new().unwrap();
    let limiter = limiter(&dir);
    let err = limiter
        .wait_or_abort(
            RateLimitWait::Spacing,
            time::Duration::seconds(1),
            true,
            &mut WaitTracker::new(),
            0,
        )
        .await
        .unwrap_err();
    assert!(err.to_string().contains("spacing wait required"));
}

#[test]
fn other_outcome_does_not_change_block_state() {
    let dir = TempDir::new().unwrap();
    let limiter = limiter(&dir);
    let now = datetime!(2026-05-09 00:00 UTC);
    limiter
        .update_post_flight(AttemptOutcome::Other, now)
        .unwrap();
    assert_eq!(limiter.store.read_state(now).consecutive_blocks, 0);
}

#[test]
fn cooldown_uses_configured_limits() {
    assert_eq!(
        Limits::test_fast(100, 200, 2).cooldown_for(2),
        Duration::from_secs(4)
    );
}
