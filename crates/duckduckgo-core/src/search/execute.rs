use std::time::Instant;

use url::Url;

use super::client::SearchBuilder;
use super::fetch::fetch_page;
use super::form::{effective_query, first_form};
use super::http::build_client;
use super::state::FetchState;
use super::types::{RateLimitJson, SearchMeta, SearchResponse};
use crate::rate_limit::RateLimiter;
use crate::{Error, Result};

pub(crate) async fn execute(builder: SearchBuilder) -> Result<SearchResponse> {
    let start = Instant::now();
    let endpoint_url =
        Url::parse(&builder.options.endpoint).map_err(|e| Error::Usage(e.to_string()))?;
    let limiter = (!builder.options.no_rate_limit).then(|| {
        RateLimiter::new(
            builder.options.state_dir.clone(),
            builder.options.proxy.as_deref(),
        )
        .with_progress_hook(builder.options.progress_hook.clone())
    });
    let http = build_client(&builder.options)?;
    let mut state = FetchState::new(builder.page.saturating_mul(builder.options.num));
    while state.should_fetch() {
        let fields = state.next.take().unwrap_or_else(|| first_form(&builder));
        let page = fetch_page(
            &http,
            &builder.options,
            &endpoint_url,
            fields,
            limiter.as_ref(),
            &mut state,
        )
        .await?;
        state.accept_page(page, &builder)?;
    }
    let results = state.slice_results(builder.page, builder.options.num);
    Ok(SearchResponse {
        schema: 1,
        query: effective_query(&builder),
        kind: "web",
        region: builder.options.region.code().to_owned(),
        page: builder.page,
        count: results.len(),
        instant_answer: state.instant_answer,
        results,
        meta: SearchMeta {
            rate_limit: RateLimitJson {
                next_allowed_at: state.snapshot.next_allowed_at.clone(),
                blocked_until: state.snapshot.blocked_until.clone(),
                slowdown_until: state.snapshot.slowdown_until.clone(),
                retried_count: state.retried_count,
            },
            fetched_pages: state.fetched_pages,
            elapsed_ms: start.elapsed().as_millis(),
        },
    })
}
