use std::time::Duration;

use url::Url;

use super::http::send_once;
use super::options::ClientOptions;
use super::state::FetchState;
use crate::parser::{BlockReason, ParsedPage, classify_block, parse_html};
use crate::rate_limit::{AttemptOutcome, RateLimiter, RunResult, Snapshot};
use crate::{Error, Result};

/// Outcome of one HTTP attempt as observed by the limiter-aware caller.
/// Distinct from `AttemptOutcome` because we keep the parsed body / error
/// alongside so the post-limit decision can route between
/// success / retry / fail without re-running the HTTP request.
pub(super) enum CallResult {
    Parsed(ParsedPage),
    Block(BlockReason),
    Transient5xx(u16),
    PermanentHttp(u16),
    Network(String),
    Parse(Error),
}

pub(crate) async fn fetch_page(
    client: &wreq::Client,
    options: &ClientOptions,
    endpoint: &Url,
    fields: Vec<(String, String)>,
    limiter: Option<&RateLimiter>,
    state: &mut FetchState,
) -> Result<ParsedPage> {
    let mut attempt: u8 = 0;
    loop {
        let result = run_attempt(client, options, endpoint, &fields, limiter).await?;
        state.snapshot = result.snapshot;
        match result.value {
            CallResult::Parsed(parsed) => return Ok(parsed),
            CallResult::Block(reason) => {
                if attempt < options.retry && !options.no_wait && !options.no_rate_limit {
                    state.retried_count += 1;
                    attempt += 1;
                    continue;
                }
                return Err(Error::Blocked(format!(
                    "blocked: {}",
                    reason.as_state_value()
                )));
            }
            CallResult::Transient5xx(status) => {
                if attempt < options.retry {
                    state.retried_count += 1;
                    attempt += 1;
                    sleep_retry(attempt).await;
                    continue;
                }
                return Err(Error::Remote(format!("HTTP {status}")));
            }
            CallResult::PermanentHttp(status) => {
                return Err(Error::Remote(format!("HTTP {status}")));
            }
            CallResult::Network(message) => {
                if attempt < options.retry {
                    state.retried_count += 1;
                    attempt += 1;
                    sleep_retry(attempt).await;
                    continue;
                }
                return Err(Error::Network(message));
            }
            CallResult::Parse(error) => return Err(error),
        }
    }
}

async fn run_attempt(
    client: &wreq::Client,
    options: &ClientOptions,
    endpoint: &Url,
    fields: &[(String, String)],
    limiter: Option<&RateLimiter>,
) -> Result<RunResult<CallResult>> {
    let body = |_snapshot: Snapshot| async move {
        let response = send_once(client, options, fields.to_vec()).await;
        classify_response(response, endpoint)
    };

    if let Some(limiter) = limiter {
        limiter.run(options.no_wait, body).await
    } else {
        let snapshot = Snapshot::default();
        let (outcome, value) = body(snapshot.clone()).await;
        Ok(RunResult {
            value,
            snapshot,
            outcome,
        })
    }
}

pub(super) fn classify_response(
    response: std::result::Result<(u16, Url, String), String>,
    endpoint: &Url,
) -> (AttemptOutcome, CallResult) {
    match response {
        Ok((status, final_url, body)) => {
            if let Some(reason) = classify_block(status, &body, &final_url, endpoint) {
                (AttemptOutcome::Block(reason), CallResult::Block(reason))
            } else if status == 200 {
                match parse_html(&body) {
                    Ok(parsed) => (AttemptOutcome::Success, CallResult::Parsed(parsed)),
                    Err(error) => (AttemptOutcome::Success, CallResult::Parse(error)),
                }
            } else if status >= 500 {
                (AttemptOutcome::Other, CallResult::Transient5xx(status))
            } else {
                (AttemptOutcome::Other, CallResult::PermanentHttp(status))
            }
        }
        Err(message) => (AttemptOutcome::Other, CallResult::Network(message)),
    }
}

async fn sleep_retry(attempt: u8) {
    let millis = 250_u64
        .saturating_mul(2_u64.saturating_pow(attempt.saturating_sub(1).into()))
        .min(4_000);
    tokio::time::sleep(Duration::from_millis(millis)).await;
}
