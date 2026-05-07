mod state;
mod store;

use std::path::PathBuf;
use std::time::Duration;

use time::OffsetDateTime;
use tokio::time::sleep;

pub use state::RateLimitState;
use store::StateStore;

use crate::parser::BlockReason;
use crate::{Error, Result};

#[derive(Clone, Debug)]
pub struct Snapshot {
    pub tokens_remaining: Option<f64>,
    pub blocked_until: Option<String>,
}

#[derive(Clone, Debug)]
pub struct RateLimiter {
    store: StateStore,
}

impl RateLimiter {
    pub fn new(state_dir: PathBuf, proxy: Option<&str>) -> Self {
        Self {
            store: StateStore::new(state_dir, proxy),
        }
    }

    pub async fn preflight(&self, no_wait: bool) -> Result<Snapshot> {
        loop {
            let decision = self.store.with_locked(|store| {
                let mut state = store.read_state();
                state.refill(OffsetDateTime::now_utc());
                if let Some(until) = state.blocked_until
                    && until > OffsetDateTime::now_utc()
                {
                    store.write_state(&state)?;
                    let wait = (until - OffsetDateTime::now_utc()).unsigned_abs();
                    return Ok(Decision::Wait(wait, "Rate limit wait required"));
                }
                if state.tokens >= 1.0 {
                    state.tokens -= 1.0;
                    let snapshot = Snapshot::from_state(&state);
                    store.write_state(&state)?;
                    return Ok(Decision::Proceed(snapshot));
                }
                let wait = Duration::from_secs_f64(1.0 - state.tokens.fract());
                store.write_state(&state)?;
                Ok(Decision::Wait(wait, "Rate limit token wait required"))
            })?;
            match decision {
                Decision::Proceed(snapshot) => return Ok(snapshot),
                Decision::Wait(wait, message) => {
                    if no_wait {
                        return Err(Error::Blocked(message.to_owned()));
                    }
                    sleep(wait.min(Duration::from_secs(80))).await;
                }
            }
        }
    }

    pub fn success(&self) -> Result<Snapshot> {
        self.store.with_locked(|store| {
            let mut state = store.read_state();
            state.consecutive_blocks = 0;
            state.blocked_until = None;
            state.last_block_reason = None;
            let snapshot = Snapshot::from_state(&state);
            store.write_state(&state)?;
            Ok(snapshot)
        })
    }

    pub fn block(&self, reason: BlockReason) -> Result<Snapshot> {
        self.store.with_locked(|store| {
            let mut state = store.read_state();
            state.consecutive_blocks = state.consecutive_blocks.saturating_add(1);
            let exp = state.consecutive_blocks.saturating_sub(1);
            let secs = 20_u64.saturating_mul(2_u64.saturating_pow(exp)).min(80);
            state.blocked_until = Some(OffsetDateTime::now_utc() + Duration::from_secs(secs));
            state.last_block_reason = Some(reason.as_state_value().to_owned());
            let snapshot = Snapshot::from_state(&state);
            store.write_state(&state)?;
            Ok(snapshot)
        })
    }
}

enum Decision {
    Proceed(Snapshot),
    Wait(Duration, &'static str),
}

impl Snapshot {
    fn from_state(state: &RateLimitState) -> Self {
        Self {
            tokens_remaining: Some(state.tokens),
            blocked_until: state.blocked_until.and_then(|t| {
                t.format(&time::format_description::well_known::Rfc3339)
                    .ok()
            }),
        }
    }
}
