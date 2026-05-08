//! Public types backing the `RateLimiter` progress hook. The CLI
//! installs a closure of type [`ProgressHook`] which the limiter calls
//! before each cooldown / spacing sleep so the user can see a
//! transparent "[INFO] rate-limit ..." line on stderr while the
//! request is gated. Library callers that don't need progress
//! reporting can ignore this module entirely; the hook is opt-in.

use std::sync::Arc;
use std::time::Duration;

/// Which kind of wait the limiter is currently performing. Surfaces in
/// `RateLimitProgress` events so callers can render distinct user-facing
/// messages for the post-block cooldown vs. the inter-request spacing
/// gap.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RateLimitWait {
    /// Post-block cooldown after a 202/403/429/anomaly response.
    Cooldown,
    /// Inter-request spacing (`base_spacing` or `slow_spacing`).
    Spacing,
}

impl RateLimitWait {
    /// Stable lowercase token suitable for short user-facing messages.
    pub fn as_token(&self) -> &'static str {
        match self {
            Self::Cooldown => "cooldown",
            Self::Spacing => "spacing",
        }
    }
}

/// One progress observation for an in-flight rate-limit wait. Emitted
/// once per acquire cycle (the inner sleep is bounded so a long
/// cooldown still yields periodic updates without a separate timer
/// task).
///
/// `total ≈ elapsed + remaining` at the moment of emission, modulo
/// jitter and small clock drift.
#[derive(Clone, Copy, Debug)]
pub struct RateLimitProgress {
    pub kind: RateLimitWait,
    pub elapsed: Duration,
    pub remaining: Duration,
    pub total: Duration,
    pub consecutive_blocks: u32,
}

/// Optional callback the limiter invokes before each wait sleep so a
/// host (typically the CLI) can render a transparent progress line. The
/// closure must be cheap; it is called while holding no locks but on
/// the hot path of the limiter loop.
pub type ProgressHook = Arc<dyn Fn(RateLimitProgress) + Send + Sync>;
