//! Inputs and outputs of `RateLimiter::run` that callers see directly.
//! `Snapshot` is the user-visible state mirror that surfaces in
//! `meta.rate_limit` of the JSON envelope. `AttemptOutcome` is the
//! caller's classification of how the request resolved, fed back into
//! the limiter so it can update persistent state under the held lock.
//! `RunResult` couples the closure's value with the post-flight state
//! mirror.

use crate::parser::BlockReason;

/// Public view of the limiter state at the moment the caller observed it.
/// Surfaces in `meta.rate_limit` of the JSON envelope.
#[derive(Clone, Debug, Default)]
pub struct Snapshot {
    pub next_allowed_at: Option<String>,
    pub blocked_until: Option<String>,
    pub slowdown_until: Option<String>,
}

/// What the caller's request resolved to. The limiter uses this to update
/// persistent state under the held lock.
#[derive(Clone, Copy, Debug)]
pub enum AttemptOutcome {
    /// HTTP 200 with usable response. Resets `consecutive_blocks` and
    /// `blocked_until`.
    Success,
    /// HTTP 202 / 403 / 429 / anomaly / challenge-redirect. Increments
    /// `consecutive_blocks` and arms the cooldown / slowdown windows.
    Block(BlockReason),
    /// Any other completion (network error, transient HTTP 5xx). Leaves
    /// block state alone but keeps the spacing advance applied so the
    /// next caller still observes the gate.
    Other,
}

/// Result of a single `run` call: the value the caller produced, the
/// limiter snapshot taken after post-flight state update, and the
/// classified outcome (kept for callers that want to distinguish a
/// successful response from a recorded block without re-inspecting the
/// snapshot timestamps).
pub struct RunResult<T> {
    pub value: T,
    pub snapshot: Snapshot,
    #[allow(dead_code)]
    pub outcome: AttemptOutcome,
}
