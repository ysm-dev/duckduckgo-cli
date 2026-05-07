use std::time::Duration;

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RateLimitState {
    pub schema: u8,
    pub tokens: f64,
    #[serde(with = "time::serde::rfc3339")]
    pub last_refill: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339::option")]
    pub blocked_until: Option<OffsetDateTime>,
    pub consecutive_blocks: u32,
    pub last_block_reason: Option<String>,
}

impl Default for RateLimitState {
    fn default() -> Self {
        Self {
            schema: 1,
            tokens: 5.0,
            last_refill: OffsetDateTime::now_utc(),
            blocked_until: None,
            consecutive_blocks: 0,
            last_block_reason: None,
        }
    }
}

impl RateLimitState {
    pub fn refill(&mut self, now: OffsetDateTime) {
        if self.last_refill > now + Duration::from_secs(24 * 60 * 60) {
            *self = Self::default();
            return;
        }
        if self
            .blocked_until
            .is_some_and(|t| t > now + Duration::from_secs(24 * 60 * 60))
        {
            *self = Self::default();
            return;
        }
        let elapsed = (now - self.last_refill).unsigned_abs().as_secs_f64();
        if now >= self.last_refill {
            self.tokens = (self.tokens + elapsed).min(5.0);
            self.last_refill = now;
        }
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use time::OffsetDateTime;

    use super::RateLimitState;

    proptest! {
        #[test]
        fn refill_never_exceeds_bucket_cap(elapsed_secs in 0_u64..10_000) {
            let now = OffsetDateTime::now_utc();
            let mut state = RateLimitState { tokens: 0.0, last_refill: now, ..RateLimitState::default() };
            state.refill(now + std::time::Duration::from_secs(elapsed_secs));
            prop_assert!(state.tokens <= 5.0);
            prop_assert!(state.tokens >= 0.0);
        }
    }
}
