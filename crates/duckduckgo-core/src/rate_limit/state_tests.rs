//! Schema-1 → schema-2 migration coverage and `sanitize` invariants for
//! [`RateLimitState`]. Lives in a sibling file so the production
//! `state.rs` stays inside the workspace's per-file line cap; load via
//! the `#[path]` attribute in `state.rs`.

use proptest::prelude::*;
use time::OffsetDateTime;

use super::RateLimitState;

#[test]
fn schema1_state_migrates_to_schema2_defaults() {
    let json = r#"{
        "schema": 1,
        "tokens": 5.0,
        "last_refill": "2026-05-07T12:34:56.789Z",
        "blocked_until": null,
        "consecutive_blocks": 0,
        "last_block_reason": null
    }"#;
    let state: RateLimitState = serde_json::from_str(json).expect("valid");
    assert_eq!(state.schema, 2);
    assert!(state.blocked_until.is_none());
    assert_eq!(state.consecutive_blocks, 0);
    assert!(state.slowdown_until.is_none());
}

#[test]
fn schema1_block_state_preserves_block_metadata() {
    let json = r#"{
        "schema": 1,
        "tokens": 0.0,
        "last_refill": "2026-05-07T12:34:56.789Z",
        "blocked_until": "2030-01-01T00:00:00Z",
        "consecutive_blocks": 2,
        "last_block_reason": "http_202"
    }"#;
    let state: RateLimitState = serde_json::from_str(json).expect("valid");
    assert_eq!(state.consecutive_blocks, 2);
    assert_eq!(state.last_block_reason.as_deref(), Some("http_202"));
    assert!(state.blocked_until.is_some());
}

proptest! {
    #[test]
    fn sanitize_clears_far_future_timestamps(future_secs in 0_i64..100_000) {
        let now = OffsetDateTime::now_utc();
        let far = now + std::time::Duration::from_secs(future_secs as u64 * 24 * 60 * 60 + 1);
        let mut state = RateLimitState {
            schema: 2,
            next_allowed_at: far,
            blocked_until: Some(far),
            slowdown_until: Some(far),
            consecutive_blocks: 5,
            last_block_reason: Some("http_202".to_owned()),
        };
        state.sanitize(now);
        prop_assert!(state.next_allowed_at <= now + std::time::Duration::from_secs(24 * 60 * 60));
    }
}
