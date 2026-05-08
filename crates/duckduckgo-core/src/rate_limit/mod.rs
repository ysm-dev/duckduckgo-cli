//! Per-egress rate-limit guard. Acquires a cross-process exclusive
//! lock on `rate-limit.lock`, observes the persisted state in
//! `rate-limit.json`, and either grants the slot, sleeps, or aborts
//! (when `--no-wait` is set). See `docs/en/spec.md` §8 for the
//! algorithm.
//!
//! Module layout:
//! - `config` — tunable limits with env overrides
//! - `outcome` — `Snapshot`, `AttemptOutcome`, `RunResult`
//! - `progress` — `RateLimitWait`, `RateLimitProgress`, `ProgressHook`
//! - `runner` — `RateLimiter` struct + the acquire / release loop
//! - `post_flight` — sibling impl block with helper methods
//! - `state` — persisted schema-2 state with schema-1 migration
//! - `store` — atomic state file + lock file pathing
//! - `wait` — wait tracker, jitter, snapshot projection

mod config;
mod outcome;
mod post_flight;
mod progress;
mod runner;
mod state;
mod store;
mod wait;

pub use config::Limits;
pub use outcome::{AttemptOutcome, RunResult, Snapshot};
pub use progress::{ProgressHook, RateLimitProgress, RateLimitWait};
pub use runner::RateLimiter;
pub use state::RateLimitState;

#[cfg(test)]
#[path = "tests/runner.rs"]
mod runner_tests;

#[cfg(test)]
#[path = "post_flight_tests.rs"]
mod post_flight_tests;

#[cfg(test)]
#[path = "store_tests.rs"]
mod store_tests;

#[cfg(test)]
#[path = "tests/concurrency.rs"]
mod concurrency_tests;

#[cfg(test)]
#[path = "tests/progress.rs"]
mod progress_tests;
