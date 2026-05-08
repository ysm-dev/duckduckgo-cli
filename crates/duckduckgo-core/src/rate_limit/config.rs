use std::env;
use std::sync::OnceLock;
use std::time::Duration;

/// Tunable constants for the spacing/in-flight guard. Defaults match the
/// 2026-05-08 first-party field report (`docs/en/ddgr.md`); they may be
/// overridden via environment variables for advanced users that have
/// measured a different egress profile.
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub base_spacing: Duration,
    pub slow_spacing: Duration,
    pub slowdown_duration: Duration,
    pub base_cooldown: Duration,
    pub max_cooldown: Duration,
    pub jitter: Duration,
}

impl Limits {
    /// Test-only fast-spacing preset for in-process unit tests.
    #[cfg(test)]
    pub fn test_fast(spacing_ms: u64, slow_spacing_ms: u64, cooldown_secs: u64) -> Self {
        Self {
            base_spacing: Duration::from_millis(spacing_ms),
            slow_spacing: Duration::from_millis(slow_spacing_ms.max(spacing_ms)),
            slowdown_duration: Duration::from_secs(cooldown_secs.saturating_mul(10)),
            base_cooldown: Duration::from_secs(cooldown_secs.max(1)),
            max_cooldown: Duration::from_secs(cooldown_secs.max(1).saturating_mul(5)),
            jitter: Duration::from_millis(0),
        }
    }
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            base_spacing: Duration::from_millis(2_000),
            slow_spacing: Duration::from_millis(4_000),
            slowdown_duration: Duration::from_secs(600),
            base_cooldown: Duration::from_secs(60),
            max_cooldown: Duration::from_secs(300),
            jitter: Duration::from_millis(300),
        }
    }
}

impl Limits {
    /// Compute the cooldown after the *n*-th consecutive block.
    /// `consecutive_blocks` is 1-indexed (1 = first block in a streak).
    pub fn cooldown_for(&self, consecutive_blocks: u32) -> Duration {
        let exp = consecutive_blocks.saturating_sub(1).min(8);
        let scaled = self.base_cooldown.saturating_mul(2_u32.saturating_pow(exp));
        scaled.min(self.max_cooldown)
    }

    /// Spacing in effect at `now`: `slow_spacing` while the slowdown window
    /// covers the moment, otherwise `base_spacing`.
    pub fn spacing(&self, slowdown_until: Option<time::OffsetDateTime>) -> Duration {
        if slowdown_until.is_some_and(|t| t > time::OffsetDateTime::now_utc()) {
            self.slow_spacing
        } else {
            self.base_spacing
        }
    }
}

/// Environment overrides:
/// - `DUCKDUCKGO_BASE_SPACING_MS` — integer ms, clamped to ≤ 60 000.
/// - `DUCKDUCKGO_SLOW_SPACING_MS` — integer ms, clamped to ≤ 120 000 and to ≥ base.
/// - `DUCKDUCKGO_BASE_COOLDOWN_S` — integer seconds, clamped to 1..=600.
///
/// Read once on first call and cached. Invalid values are ignored silently
/// to keep the limiter robust under broken shell environments.
pub fn limits() -> Limits {
    static LIMITS: OnceLock<Limits> = OnceLock::new();
    *LIMITS.get_or_init(load_from_env)
}

fn load_from_env() -> Limits {
    let mut limits = Limits::default();
    if let Some(ms) = env_u64("DUCKDUCKGO_BASE_SPACING_MS").filter(|v| *v <= 60_000) {
        limits.base_spacing = Duration::from_millis(ms);
    }
    if let Some(ms) = env_u64("DUCKDUCKGO_SLOW_SPACING_MS").filter(|v| *v <= 120_000) {
        let candidate = Duration::from_millis(ms);
        limits.slow_spacing = candidate.max(limits.base_spacing);
    }
    if let Some(secs) = env_u64("DUCKDUCKGO_BASE_COOLDOWN_S").filter(|v| (1..=600).contains(v)) {
        limits.base_cooldown = Duration::from_secs(secs);
        limits.max_cooldown = limits.max_cooldown.max(limits.base_cooldown);
    }
    limits
}

fn env_u64(key: &str) -> Option<u64> {
    env::var(key).ok().and_then(|v| v.parse::<u64>().ok())
}

#[cfg(test)]
mod tests {
    use super::Limits;
    use std::time::Duration;

    #[test]
    fn cooldown_doubles_then_caps() {
        let limits = Limits::default();
        assert_eq!(limits.cooldown_for(0), Duration::from_secs(60));
        assert_eq!(limits.cooldown_for(1), Duration::from_secs(60));
        assert_eq!(limits.cooldown_for(2), Duration::from_secs(120));
        assert_eq!(limits.cooldown_for(3), Duration::from_secs(240));
        assert_eq!(limits.cooldown_for(4), Duration::from_secs(300));
        assert_eq!(limits.cooldown_for(8), Duration::from_secs(300));
        assert_eq!(limits.cooldown_for(99), Duration::from_secs(300));
    }

    #[test]
    fn spacing_picks_slow_inside_window() {
        let limits = Limits::default();
        let in_future = time::OffsetDateTime::now_utc() + Duration::from_secs(60);
        let in_past = time::OffsetDateTime::now_utc() - Duration::from_secs(60);
        assert_eq!(
            limits.spacing(Some(in_future)),
            Duration::from_millis(4_000)
        );
        assert_eq!(limits.spacing(Some(in_past)), Duration::from_millis(2_000));
        assert_eq!(limits.spacing(None), Duration::from_millis(2_000));
    }
}
