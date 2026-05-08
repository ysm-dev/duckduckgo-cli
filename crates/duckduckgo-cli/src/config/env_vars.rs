use std::env;

use duckduckgo_core::{Error, Result};

use super::raw::{Raw, set_env_string};

pub(crate) fn apply_standard_proxy(raw: &mut Raw) {
    if raw.proxy.is_none() {
        raw.proxy = env::var("HTTPS_PROXY")
            .or_else(|_| env::var("HTTP_PROXY"))
            .or_else(|_| env::var("ALL_PROXY"))
            .ok();
    }
}

pub(crate) fn apply_env(raw: &mut Raw) -> Result<()> {
    set_env_string(&mut raw.region, "DUCKDUCKGO_REGION");
    set_env_string(&mut raw.num, "DUCKDUCKGO_NUM");
    set_env_string(&mut raw.time, "DUCKDUCKGO_TIME");
    set_env_string(&mut raw.timeout, "DUCKDUCKGO_TIMEOUT");
    set_env_string(&mut raw.proxy, "DUCKDUCKGO_PROXY");
    set_env_string(&mut raw.user_agent, "DUCKDUCKGO_USER_AGENT");
    set_env_string(&mut raw.retry, "DUCKDUCKGO_RETRY");
    apply_color_env(raw);
    apply_bool_envs(raw)?;
    if let Ok(value) = env::var("DUCKDUCKGO_VERBOSE") {
        raw.verbose = Some(
            value
                .parse()
                .map_err(|_| Error::Usage("Invalid DUCKDUCKGO_VERBOSE".to_owned()))?,
        );
    }
    Ok(())
}

fn apply_color_env(raw: &mut Raw) {
    let duck_color = env::var("DUCKDUCKGO_COLOR").ok();
    if let Some(value) = color_from_env(duck_color, env::var_os("NO_COLOR").is_some()) {
        raw.color = Some(value);
    }
}

fn color_from_env(duck_color: Option<String>, no_color: bool) -> Option<String> {
    duck_color.or_else(|| no_color.then(|| "never".to_owned()))
}

fn apply_bool_envs(raw: &mut Raw) -> Result<()> {
    if let Ok(value) = env::var("DUCKDUCKGO_SAFE") {
        raw.safe = Some(parse_bool_env("DUCKDUCKGO_SAFE", &value)?);
    }
    if let Ok(value) = env::var("DUCKDUCKGO_NO_UPDATE_CHECK") {
        raw.no_update_check = Some(parse_bool_env("DUCKDUCKGO_NO_UPDATE_CHECK", &value)?);
    }
    if let Ok(value) = env::var("DUCKDUCKGO_NO_RATE_LIMIT") {
        raw.no_rate_limit = Some(parse_bool_env("DUCKDUCKGO_NO_RATE_LIMIT", &value)?);
    }
    if let Ok(value) = env::var("DUCKDUCKGO_QUIET") {
        raw.quiet = Some(parse_bool_env("DUCKDUCKGO_QUIET", &value)?);
    }
    Ok(())
}

fn parse_bool_env(key: &str, value: &str) -> Result<bool> {
    match value.to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Ok(true),
        "0" | "false" | "no" | "off" => Ok(false),
        _ => Err(Error::Usage(format!("Invalid {key} value '{value}'"))),
    }
}

#[cfg(test)]
mod tests {
    use super::{color_from_env, parse_bool_env};

    #[test]
    fn parse_bool_env_accepts_documented_true_values() {
        for value in ["1", "true", "TRUE", "yes", "on"] {
            assert!(parse_bool_env("KEY", value).unwrap(), "{value}");
        }
    }

    #[test]
    fn parse_bool_env_accepts_documented_false_values() {
        for value in ["0", "false", "FALSE", "no", "off"] {
            assert!(!parse_bool_env("KEY", value).unwrap(), "{value}");
        }
    }

    #[test]
    fn parse_bool_env_rejects_unknown_values() {
        let err = parse_bool_env("DUCKDUCKGO_SAFE", "maybe").unwrap_err();
        assert!(err.to_string().contains("Invalid DUCKDUCKGO_SAFE"));
    }

    #[test]
    fn duckduckgo_color_precedes_no_color() {
        assert_eq!(
            color_from_env(Some("always".to_owned()), true),
            Some("always".to_owned())
        );
        assert_eq!(color_from_env(None, true), Some("never".to_owned()));
        assert_eq!(color_from_env(None, false), None);
    }
}
