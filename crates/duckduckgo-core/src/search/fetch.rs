use std::time::Duration;

use url::Url;

use super::http::send_once;
use super::options::ClientOptions;
use super::state::FetchState;
use crate::parser::{classify_block, parse_html};
use crate::rate_limit::RateLimiter;
use crate::{Error, Result};

pub(crate) async fn fetch_page(
    client: &wreq::Client,
    options: &ClientOptions,
    endpoint: &Url,
    fields: Vec<(String, String)>,
    limiter: Option<&RateLimiter>,
    state: &mut FetchState,
) -> Result<crate::parser::ParsedPage> {
    let mut attempt = 0;
    loop {
        if let Some(limiter) = limiter {
            state.snapshot = limiter.preflight(options.no_wait).await?;
        }
        match send_once(client, options, fields.clone()).await {
            Ok((status, final_url, body)) => {
                if let Some(reason) = classify_block(status, &body, &final_url, endpoint) {
                    if let Some(limiter) = limiter {
                        state.snapshot = limiter.block(reason)?;
                    }
                    if attempt < options.retry && !options.no_wait && !options.no_rate_limit {
                        state.retried_count += 1;
                        attempt += 1;
                        tokio::time::sleep(Duration::from_secs(20)).await;
                        continue;
                    }
                    return Err(Error::Blocked(format!(
                        "blocked: {}",
                        reason.as_state_value()
                    )));
                }
                if status == 200 {
                    let parsed = parse_html(&body)?;
                    if let Some(limiter) = limiter {
                        state.snapshot = limiter.success()?;
                    }
                    return Ok(parsed);
                }
                if status >= 500 && attempt < options.retry {
                    state.retried_count += 1;
                    attempt += 1;
                    sleep_retry(attempt).await;
                    continue;
                }
                return Err(Error::Remote(format!("HTTP {status}")));
            }
            Err(error) if attempt < options.retry => {
                state.retried_count += 1;
                attempt += 1;
                sleep_retry(attempt).await;
                if error.is_empty() {
                    continue;
                }
            }
            Err(error) => return Err(Error::Network(error)),
        }
    }
}

async fn sleep_retry(attempt: u8) {
    let millis = 250_u64
        .saturating_mul(2_u64.saturating_pow(attempt.saturating_sub(1).into()))
        .min(4_000);
    tokio::time::sleep(Duration::from_millis(millis)).await;
}
