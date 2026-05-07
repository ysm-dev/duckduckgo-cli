use serde_json::json;
use url::Url;

use super::Settings;
use super::raw::ColorWhen;
use duckduckgo_core::TimeFilter;

pub fn print_json(settings: &Settings) -> String {
    json!({
        "schema": 1,
        "search": { "region": settings.region.code(), "num": settings.num, "safe": settings.safe, "time": time_code(settings.time) },
        "network": { "timeout": settings.timeout, "proxy": settings.proxy.as_deref().map(redact_proxy), "user_agent": settings.user_agent.as_deref(), "retry": settings.retry },
        "output": { "color": color_name(&settings.color) },
        "operational": { "no_update_check": settings.no_update_check, "rate_limit": settings.rate_limit },
        "paths": {
            "config_file": settings.paths.config_file.display().to_string(),
            "state_dir": settings.paths.state_dir.display().to_string(),
            "cache_dir": settings.paths.cache_dir.display().to_string()
        }
    }).to_string()
}

fn redact_proxy(value: &str) -> String {
    let Ok(mut url) = Url::parse(value) else {
        return value.to_owned();
    };
    if url.password().is_some() {
        let _ = url.set_password(Some("***"));
    }
    url.to_string()
}

fn color_name(value: &ColorWhen) -> &'static str {
    match value {
        ColorWhen::Auto => "auto",
        ColorWhen::Always => "always",
        ColorWhen::Never => "never",
    }
}

fn time_code(value: Option<TimeFilter>) -> Option<&'static str> {
    value.map(|v| match v {
        TimeFilter::Day => "d",
        TimeFilter::Week => "w",
        TimeFilter::Month => "m",
        TimeFilter::Year => "y",
    })
}
