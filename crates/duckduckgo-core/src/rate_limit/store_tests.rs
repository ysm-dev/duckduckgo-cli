use tempfile::TempDir;
use time::macros::datetime;

use crate::rate_limit::RateLimitState;
use crate::rate_limit::store::{StateStore, normalize_proxy, proxy_hash};

#[test]
fn normalize_proxy_canonicalizes_default_ports_and_case() {
    assert_eq!(
        normalize_proxy("HTTP://EXAMPLE.COM:80/path/"),
        Some("http://example.com/path".to_owned())
    );
    assert_eq!(
        normalize_proxy("socks5h://Proxy.Example:1080"),
        Some("socks5h://proxy.example".to_owned())
    );
    assert_eq!(
        normalize_proxy("https://example.com:8443/a?x=1"),
        Some("https://example.com:8443/a?x=1".to_owned())
    );
}

#[test]
fn proxy_hash_is_stable_for_equivalent_proxy_urls() {
    assert_eq!(
        proxy_hash("http://example.com"),
        proxy_hash("HTTP://EXAMPLE.COM:80/")
    );
    assert_eq!(proxy_hash("not a url"), proxy_hash("not a url"));
}

#[test]
fn write_then_read_state_round_trips() {
    let dir = TempDir::new().unwrap();
    let now = datetime!(2026-05-09 00:00 UTC);
    let store = StateStore::new(dir.path().to_path_buf(), None);
    let state = RateLimitState {
        schema: 2,
        next_allowed_at: now + time::Duration::seconds(2),
        blocked_until: Some(now + time::Duration::seconds(3)),
        slowdown_until: Some(now + time::Duration::seconds(4)),
        consecutive_blocks: 2,
        last_block_reason: Some("http_202".to_owned()),
    };
    store.write_state(&state).unwrap();
    let read = store.read_state(now);
    assert_eq!(read.consecutive_blocks, 2);
    assert_eq!(read.last_block_reason.as_deref(), Some("http_202"));
    assert_eq!(read.blocked_until, state.blocked_until);
}
