use std::time::Duration;

use duckduckgo_core::paths::ensure_parent;
use duckduckgo_core::{Error, Result};
use serde_json::json;
use time::OffsetDateTime;

use crate::config::Settings;
use crate::output;

pub async fn check_now(settings: &Settings) -> Result<()> {
    let latest = fetch_latest(settings).await?;
    write_cache(settings, Some(&latest))?;
    if version_gt(&latest, env!("CARGO_PKG_VERSION")) {
        println!(
            "duckduckgo-cli {latest} available (you have v{}).",
            env!("CARGO_PKG_VERSION")
        );
    } else {
        println!(
            "duckduckgo-cli is up to date (v{}).",
            env!("CARGO_PKG_VERSION")
        );
    }
    Ok(())
}

pub async fn check_auto(settings: &Settings) -> Result<()> {
    if cache_fresh(settings) {
        return Ok(());
    }
    match tokio::time::timeout(Duration::from_millis(750), fetch_latest(settings)).await {
        Ok(Ok(latest)) => {
            write_cache(settings, Some(&latest))?;
            if version_gt(&latest, env!("CARGO_PKG_VERSION")) && !settings.quiet {
                output::info(
                    settings,
                    &format!(
                        "duckduckgo-cli {latest} available (you have v{}). Run `npm i -g duckduckgo-cli@latest` to update.",
                        env!("CARGO_PKG_VERSION")
                    ),
                );
            }
        }
        _ => write_cache(settings, None)?,
    }
    Ok(())
}

pub(crate) async fn fetch_latest(settings: &Settings) -> Result<String> {
    let url = format!(
        "{}/repos/ysm-dev/duckduckgo-cli/releases/latest",
        settings.github_url.trim_end_matches('/')
    );
    let client = wreq::Client::builder()
        .no_proxy()
        .timeout(Duration::from_secs(30))
        .user_agent(format!("duckduckgo-cli/{}", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|e| Error::Network(e.to_string()))?;
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| Error::Network(e.to_string()))?;
    if !response.status().is_success() {
        return Err(Error::Network(format!("GitHub HTTP {}", response.status())));
    }
    let body = response
        .text()
        .await
        .map_err(|e| Error::Network(e.to_string()))?;
    let value: serde_json::Value =
        serde_json::from_str(&body).map_err(|e| Error::Network(e.to_string()))?;
    value["tag_name"]
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| Error::Network("Missing tag_name".to_owned()))
}

fn cache_fresh(settings: &Settings) -> bool {
    cache_fresh_at(settings, OffsetDateTime::now_utc())
}

pub(crate) fn cache_fresh_at(settings: &Settings, now: OffsetDateTime) -> bool {
    let path = settings.paths.cache_dir.join("update.json");
    let Ok(text) = std::fs::read_to_string(path) else {
        return false;
    };
    cache_value_fresh(&text, now)
}

pub(crate) fn cache_value_fresh(text: &str, now: OffsetDateTime) -> bool {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(text) else {
        return false;
    };
    let Some(last) = value["last_check"].as_str() else {
        return false;
    };
    let Ok(last) = OffsetDateTime::parse(last, &time::format_description::well_known::Rfc3339)
    else {
        return false;
    };
    now - last < time::Duration::hours(24)
}

fn write_cache(settings: &Settings, latest: Option<&str>) -> Result<()> {
    let path = settings.paths.cache_dir.join("update.json");
    ensure_parent(&path)?;
    let now = OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|e| Error::Io(e.to_string()))?;
    let last_success = latest
        .map(str::to_owned)
        .or_else(|| previous_success(&path));
    let body = json!({ "schema": 1, "last_check": now, "last_success": last_success }).to_string();
    std::fs::write(path, body)?;
    Ok(())
}

pub(crate) fn previous_success(path: &std::path::Path) -> Option<String> {
    let text = std::fs::read_to_string(path).ok()?;
    let value = serde_json::from_str::<serde_json::Value>(&text).ok()?;
    value["last_success"].as_str().map(str::to_owned)
}

pub(crate) fn version_gt(latest: &str, current: &str) -> bool {
    let parse = |v: &str| {
        v.trim_start_matches('v')
            .split('.')
            .filter_map(|p| p.parse::<u64>().ok())
            .collect::<Vec<_>>()
    };
    parse(latest) > parse(current)
}
