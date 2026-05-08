use std::env;

use duckduckgo_core::paths::RuntimePaths;
use duckduckgo_core::{Error, Region, Result, TimeFilter};
use url::Url;

use crate::args::Cli;

use super::Settings;
use super::raw::{ColorWhen, Raw};

pub(crate) fn validate(
    cli: &Cli,
    raw: Raw,
    paths: RuntimePaths,
    mut warnings: Vec<String>,
) -> Result<Settings> {
    let region_raw = raw.region.unwrap_or_else(|| "us-en".to_owned());
    let region = Region::parse(&region_raw)
        .ok_or_else(|| Error::Usage(format!("Invalid region code '{region_raw}'")))?;
    let num = parse_range(raw.num.as_deref().unwrap_or("10"), "--num", 1, 100)? as usize;
    let page = parse_range(raw.page.as_deref().unwrap_or("1"), "--page", 1, u64::MAX)? as usize;
    let timeout = parse_range(
        raw.timeout.as_deref().unwrap_or("30"),
        "--timeout",
        1,
        u64::MAX,
    )?;
    let retry = parse_range(raw.retry.as_deref().unwrap_or("3"), "--retry", 0, 10)? as u8;
    let time = raw.time.as_deref().map(parse_time).transpose()?;
    let color = parse_color(raw.color.as_deref().unwrap_or("auto"))?;
    let proxy = raw.proxy.map(validate_proxy).transpose()?;
    let user_agent = validate_user_agent(raw.user_agent, &mut warnings)?;
    let verbose = raw.verbose.unwrap_or(0).min(2);
    let quiet = raw.quiet.unwrap_or(false);
    if quiet && verbose > 0 {
        return Err(Error::Usage(
            "--quiet and --verbose are mutually exclusive".to_owned(),
        ));
    }
    Ok(Settings {
        region,
        num,
        page,
        safe: raw.safe.unwrap_or(true),
        time,
        sites: cli.sites.clone(),
        json: cli.json,
        color,
        verbose,
        quiet,
        timeout,
        proxy,
        user_agent,
        retry,
        no_wait: cli.no_wait,
        no_update_check: raw.no_update_check.unwrap_or(false),
        rate_limit: !raw.no_rate_limit.unwrap_or(false),
        paths,
        warnings,
        // Trailing slash is mandatory: posting to `/html` returns
        // `HTTP 202` with the anomaly modal, while `/html/` returns
        // `HTTP 200` with results. Verified empirically against the
        // live endpoint (2026-05-08).
        ddg_url: env::var("DUCKDUCKGO_DDG_URL")
            .unwrap_or_else(|_| "https://html.duckduckgo.com/html/".to_owned()),
        github_url: env::var("DUCKDUCKGO_GITHUB_URL")
            .unwrap_or_else(|_| "https://api.github.com".to_owned()),
    })
}

pub(super) fn validate_user_agent(
    value: Option<String>,
    warnings: &mut Vec<String>,
) -> Result<Option<String>> {
    if value.as_deref() == Some("") {
        return Err(Error::Usage("Invalid --user-agent ''".to_owned()));
    }
    if value.as_deref().is_some_and(browser_ua) {
        warnings.push("Browser-shaped User-Agent will likely increase block rates.".to_owned());
    }
    Ok(value)
}

pub(super) fn parse_range(value: &str, name: &str, min: u64, max: u64) -> Result<u64> {
    let parsed = value
        .parse::<u64>()
        .map_err(|_| Error::Usage(format!("Invalid {name} '{value}'")))?;
    if parsed < min || parsed > max {
        return Err(Error::Usage(format!("Invalid {name} '{value}'")));
    }
    Ok(parsed)
}

pub(super) fn parse_time(value: &str) -> Result<TimeFilter> {
    match value {
        "d" => Ok(TimeFilter::Day),
        "w" => Ok(TimeFilter::Week),
        "m" => Ok(TimeFilter::Month),
        "y" => Ok(TimeFilter::Year),
        _ => Err(Error::Usage(format!("Invalid --time '{value}'"))),
    }
}

pub(super) fn parse_color(value: &str) -> Result<ColorWhen> {
    match value {
        "auto" => Ok(ColorWhen::Auto),
        "always" => Ok(ColorWhen::Always),
        "never" => Ok(ColorWhen::Never),
        _ => Err(Error::Usage(format!("Invalid --color '{value}'"))),
    }
}

pub(super) fn validate_proxy(value: String) -> Result<String> {
    let url = Url::parse(&value).map_err(|_| Error::Usage(format!("Invalid --proxy '{value}'")))?;
    match url.scheme() {
        "http" | "https" | "socks5" | "socks5h" => Ok(value),
        _ => Err(Error::Usage(format!("Invalid --proxy '{value}'"))),
    }
}

pub(super) fn browser_ua(value: &str) -> bool {
    value.starts_with("Mozilla/")
        || ["Chrome/", "Firefox/", "Safari/", "Edge/", "Opera/"]
            .iter()
            .any(|needle| value.contains(needle))
}
