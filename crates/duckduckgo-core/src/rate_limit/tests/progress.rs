//! Coverage for the progress-hook contract: fires on cooldown,
//! suppresses sub-1 s spacing waits, and stays silent when `--no-wait`
//! aborts before any sleep happens.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use tempfile::TempDir;

use crate::parser::BlockReason;
use crate::rate_limit::config::Limits;
use crate::rate_limit::{
    AttemptOutcome, ProgressHook, RateLimitProgress, RateLimitWait, RateLimiter,
};
use crate::{Error, ManualClock};

#[tokio::test(flavor = "current_thread")]
async fn progress_hook_fires_during_cooldown_with_carrying_consecutive_blocks() {
    let dir = TempDir::new().unwrap();
    let calls: Arc<Mutex<Vec<RateLimitProgress>>> = Arc::new(Mutex::new(Vec::new()));
    let calls_for_hook = calls.clone();
    let hook: ProgressHook = Arc::new(move |p| calls_for_hook.lock().unwrap().push(p));
    let limiter = RateLimiter::new(
        dir.path().to_path_buf(),
        None,
        // 2 s base cooldown — the limiter's internal threshold is 1 s
        // total wait, so this guarantees emission on the second run.
        Limits::test_fast(50, 100, 2),
        Arc::new(ManualClock::new(time::OffsetDateTime::now_utc())),
    )
    .with_progress_hook(Some(hook));

    // First call: closure fires immediately (no prior state, no
    // wait). It returns Block, which arms `blocked_until = now + 2s`.
    let _ = limiter
        .run(false, |snap| async move {
            (AttemptOutcome::Block(BlockReason::Http202), snap)
        })
        .await
        .unwrap();
    assert!(
        calls.lock().unwrap().is_empty(),
        "no waiting happened, hook must not have fired",
    );

    // Second call: cooldown is in effect; the limiter must call the
    // hook before sleeping.
    let _ = limiter
        .run(false, |snap| async move { (AttemptOutcome::Success, snap) })
        .await
        .unwrap();

    let observed = calls.lock().unwrap();
    let cooldown = observed
        .iter()
        .find(|p| p.kind == RateLimitWait::Cooldown)
        .expect("at least one cooldown progress event");
    assert!(
        cooldown.total >= Duration::from_secs(1),
        "total wait below threshold: {:?}",
        cooldown.total,
    );
    assert!(
        cooldown.remaining > Duration::ZERO,
        "remaining must be positive on the first emit",
    );
    assert_eq!(cooldown.consecutive_blocks, 1, "first block in the streak");
}

#[tokio::test(flavor = "current_thread")]
async fn progress_hook_silent_below_one_second_threshold() {
    let dir = TempDir::new().unwrap();
    let calls: Arc<Mutex<Vec<RateLimitProgress>>> = Arc::new(Mutex::new(Vec::new()));
    let calls_for_hook = calls.clone();
    let hook: ProgressHook = Arc::new(move |p| calls_for_hook.lock().unwrap().push(p));
    let limiter = RateLimiter::new(
        dir.path().to_path_buf(),
        None,
        // 200 ms spacing → second run sees a sub-second wait.
        Limits::test_fast(200, 200, 1),
        Arc::new(ManualClock::new(time::OffsetDateTime::now_utc())),
    )
    .with_progress_hook(Some(hook));

    let _ = limiter
        .run(false, |snap| async move { (AttemptOutcome::Success, snap) })
        .await
        .unwrap();
    let _ = limiter
        .run(false, |snap| async move { (AttemptOutcome::Success, snap) })
        .await
        .unwrap();

    assert!(
        calls
            .lock()
            .unwrap()
            .iter()
            .all(|p| p.total >= Duration::from_secs(1)),
        "spacing waits below 1 s must not be reported",
    );
}

#[tokio::test(flavor = "current_thread")]
async fn progress_hook_not_called_when_no_wait_aborts() {
    let dir = TempDir::new().unwrap();
    let calls: Arc<Mutex<Vec<RateLimitProgress>>> = Arc::new(Mutex::new(Vec::new()));
    let calls_for_hook = calls.clone();
    let hook: ProgressHook = Arc::new(move |p| calls_for_hook.lock().unwrap().push(p));
    let limiter = RateLimiter::new(
        dir.path().to_path_buf(),
        None,
        Limits::test_fast(50, 100, 2),
        Arc::new(ManualClock::new(time::OffsetDateTime::now_utc())),
    )
    .with_progress_hook(Some(hook));

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
        .expect("blocked");
    assert!(matches!(err, Error::Blocked(_)));
    assert!(
        calls.lock().unwrap().is_empty(),
        "hook must not fire on the no_wait abort path",
    );
}
