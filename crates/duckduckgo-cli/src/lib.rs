#![forbid(unsafe_code)]

mod args;
mod config;
mod output;
mod stdin_query;
mod update_check;

use std::io::Write;
use std::process::ExitCode;

use clap::Parser;
use duckduckgo_core::{Client, Error};

use crate::args::Cli;
use crate::config::Settings;

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
    let cli = Cli::try_parse().map_err(|e| Error::Usage(e.to_string()))?;
    if cli.help_short || cli.help_long {
        print_help(cli.help_long);
        return Ok(0);
    }
    if cli.version {
        println!(
            "duckduckgo-cli {} (commit unknown, target {})",
            env!("CARGO_PKG_VERSION"),
            env!("DUCKDUCKGO_TARGET")
        );
        return Ok(0);
    }
    if let Some(shell) = &cli.completion {
        args::print_completion(shell)?;
        return Ok(0);
    }
    if cli.list_regions {
        for (code, name) in duckduckgo_core::region::REGION_CODES {
            println!("{code}\t{name}");
        }
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

    let query = stdin_query::query(&cli.query)?;
    let client = build_client(&settings)?;
    let mut search = client.search(query).page(settings.page).time(settings.time);
    for site in &settings.sites {
        search = search.site(site.clone());
    }
    let response = search.send().await?;
    let code = if response.results.is_empty() { 1 } else { 0 };
    output::render(&settings, &response)?;
    if code == 1 && !settings.json && !settings.quiet {
        output::warn(&settings, "No results.");
    }
    if code <= 1 && output::should_auto_update(&settings) {
        let _ = std::io::stdout().flush();
        let _ = update_check::check_auto(&settings).await;
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
        .build()
}

fn print_help(long: bool) {
    if long {
        println!("{}", args::long_help());
    } else {
        println!("{}", args::short_help());
    }
}
