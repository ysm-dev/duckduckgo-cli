#![forbid(unsafe_code)]

mod args;
mod config;
mod meta;
mod output;
mod stdin_query;
mod update_check;

#[cfg(test)]
mod args_tests;
#[cfg(test)]
mod update_check_http_tests;
#[cfg(test)]
mod update_check_tests;

use std::io::Write;
use std::process::ExitCode;
use std::sync::Arc;

use clap::Parser;
use duckduckgo_core::{Client, Limits, ProgressHook, RateLimitProgress};

use crate::args::Cli;
use crate::config::Settings;
use crate::output::OutputCtx;

pub async fn main_entry() -> ExitCode {
    match run().await {
        Ok(code) => ExitCode::from(code as u8),
        Err(error) => {
            eprintln!("[ERROR] {error}");
            ExitCode::from(error.exit_code() as u8)
        }
    }
}

async fn run() -> duckduckgo_core::Result<i32> {
    let cli = Cli::try_parse().map_err(meta::clap_usage)?;
    if meta::dispatch(&cli)? {
        return Ok(0);
    }

    let settings = config::load(&cli)?;
    output::emit_warnings(&settings);
    if cli.print_config {
        println!("{}", config::print_json(&settings));
        return Ok(0);
    }
    if cli.check_updates {
        update_check::check_now(&settings).await?;
        return Ok(0);
    }

    run_search(&cli, &settings).await
}

async fn run_search(cli: &Cli, settings: &Settings) -> duckduckgo_core::Result<i32> {
    let query = stdin_query::query(&cli.query)?;
    let client = build_client(settings)?;
    let mut search = client.search(query).page(settings.page).time(settings.time);
    for site in &settings.sites {
        search = search.site(site.clone());
    }
    let response = search.send().await?;
    let code = if response.results.is_empty() { 1 } else { 0 };
    output::render(settings, &response)?;
    if code == 1 && !settings.json && !settings.quiet {
        output::warn(settings, "No results.");
    }
    if code <= 1 && output::should_auto_update(settings) {
        let _ = std::io::stdout().flush();
        let _ = update_check::check_auto(settings).await;
    }
    Ok(code)
}

fn build_client(settings: &Settings) -> duckduckgo_core::Result<Client> {
    Client::builder()
        .region(settings.region.clone())
        .num(settings.num)
        .safe(settings.safe)
        .timeout(settings.timeout)
        .proxy(settings.proxy.clone())
        .user_agent(settings.user_agent.clone())
        .retry(settings.retry)
        .no_wait(settings.no_wait)
        .no_rate_limit(!settings.rate_limit)
        .state_dir(settings.paths.state_dir.clone())
        .endpoint(settings.ddg_url.clone())
        .limits(Limits::from_env())
        .on_rate_limit_progress(rate_limit_progress_hook(settings))
        .build()
}

/// Build the closure the limiter calls before each cooldown / spacing
/// sleep. Renders the short-token format
/// `rate-limit {kind} {elapsed}s/{total}s ({remaining}s left)` and
/// routes it through `output::info_with`, which honours `--quiet` and
/// the `--color` decision for stderr.
fn rate_limit_progress_hook(settings: &Settings) -> ProgressHook {
    let ctx = OutputCtx::from_settings(settings);
    Arc::new(move |progress: RateLimitProgress| {
        let elapsed = progress.elapsed.as_secs();
        let total = progress.total.as_secs();
        let remaining = progress.remaining.as_secs();
        let kind = progress.kind.as_token();
        let suffix = if progress.consecutive_blocks > 1 {
            format!(" (block #{})", progress.consecutive_blocks)
        } else {
            String::new()
        };
        let message = format!("rate-limit {kind} {elapsed}s/{total}s ({remaining}s left){suffix}");
        output::info_with(&ctx, &message);
    })
}
