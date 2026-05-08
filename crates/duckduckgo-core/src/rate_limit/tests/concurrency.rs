//! Demonstrates that the OS-level lock plus persisted spacing
//! serialise in-process concurrent calls: at most one closure runs at
//! a time, and consecutive starts respect `BASE_SPACING`. Lives in
//! a sibling test file so `runner.rs` and `runner_tests.rs` stay
//! within the per-file line cap.

use std::sync::Arc;
use std::time::{Duration, Instant};

use tempfile::TempDir;
use tokio::sync::Mutex;
use tokio::task::LocalSet;

use crate::rate_limit::config::Limits;
use crate::rate_limit::{AttemptOutcome, RateLimiter};

#[tokio::test(flavor = "current_thread")]
async fn concurrent_in_process_calls_serialise_via_lock_and_spacing() {
    let dir = TempDir::new().unwrap();
    let limiter = Arc::new(RateLimiter::with_limits(
        dir.path().to_path_buf(),
        None,
        Limits::test_fast(120, 240, 1),
    ));
    let starts: Arc<Mutex<Vec<Instant>>> = Arc::new(Mutex::new(Vec::new()));
    let in_flight: Arc<Mutex<u32>> = Arc::new(Mutex::new(0));
    let max_in_flight: Arc<Mutex<u32>> = Arc::new(Mutex::new(0));
    let count = 5_u32;

    let local = LocalSet::new();
    local
        .run_until(async {
            let mut handles = Vec::with_capacity(count as usize);
            for _ in 0..count {
                let limiter = limiter.clone();
                let starts = starts.clone();
                let in_flight = in_flight.clone();
                let max_in_flight = max_in_flight.clone();
                handles.push(tokio::task::spawn_local(async move {
                    limiter
                        .run(false, |snap| {
                            let starts = starts.clone();
                            let in_flight = in_flight.clone();
                            let max_in_flight = max_in_flight.clone();
                            async move {
                                starts.lock().await.push(Instant::now());
                                let mut current = in_flight.lock().await;
                                *current += 1;
                                let cur = *current;
                                drop(current);
                                let mut high = max_in_flight.lock().await;
                                if cur > *high {
                                    *high = cur;
                                }
                                drop(high);
                                tokio::time::sleep(Duration::from_millis(40)).await;
                                *in_flight.lock().await -= 1;
                                (AttemptOutcome::Success, snap)
                            }
                        })
                        .await
                        .expect("run succeeded")
                }));
            }
            for h in handles {
                let _ = h.await;
            }
        })
        .await;

    let starts = starts.lock().await.clone();
    assert_eq!(starts.len() as u32, count, "all closures must run");
    let mut sorted = starts.clone();
    sorted.sort();
    for window in sorted.windows(2) {
        let gap = window[1].duration_since(window[0]);
        assert!(
            gap >= Duration::from_millis(110),
            "consecutive starts must be spaced by ≥ 110ms (~ base spacing - 10ms tolerance), got {gap:?}",
        );
    }
    let max = *max_in_flight.lock().await;
    assert_eq!(
        max, 1,
        "at most one closure must run at a time; observed {max}"
    );
}
