use super::raw::ColorWhen;
use super::validate::{
    browser_ua, parse_color, parse_range, parse_time, validate_proxy, validate_user_agent,
};
use duckduckgo_core::TimeFilter;

#[test]
fn parse_range_enforces_bounds_and_numeric_input() {
    assert_eq!(parse_range("1", "--num", 1, 100).unwrap(), 1);
    assert_eq!(parse_range("100", "--num", 1, 100).unwrap(), 100);
    assert!(parse_range("0", "--num", 1, 100).is_err());
    assert!(parse_range("101", "--num", 1, 100).is_err());
    assert!(parse_range("many", "--num", 1, 100).is_err());
}

#[test]
fn parse_time_accepts_documented_tokens() {
    assert_eq!(parse_time("d").unwrap(), TimeFilter::Day);
    assert_eq!(parse_time("w").unwrap(), TimeFilter::Week);
    assert_eq!(parse_time("m").unwrap(), TimeFilter::Month);
    assert_eq!(parse_time("y").unwrap(), TimeFilter::Year);
    assert!(parse_time("hour").is_err());
}

#[test]
fn parse_color_accepts_documented_values() {
    assert_eq!(parse_color("auto").unwrap(), ColorWhen::Auto);
    assert_eq!(parse_color("always").unwrap(), ColorWhen::Always);
    assert_eq!(parse_color("never").unwrap(), ColorWhen::Never);
    assert!(parse_color("sometimes").is_err());
}

#[test]
fn validate_proxy_accepts_supported_schemes() {
    for value in [
        "http://proxy.example",
        "https://proxy.example",
        "socks5://proxy.example",
        "socks5h://proxy.example",
    ] {
        assert_eq!(validate_proxy(value.to_owned()).unwrap(), value);
    }
    assert!(validate_proxy("ftp://proxy.example".to_owned()).is_err());
    assert!(validate_proxy("not a url".to_owned()).is_err());
}

#[test]
fn browser_user_agents_are_warned() {
    for value in [
        "Mozilla/5.0",
        "tool Chrome/120",
        "tool Firefox/120",
        "tool Safari/17",
        "tool Edge/120",
        "tool Opera/100",
    ] {
        assert!(browser_ua(value), "{value}");
    }
    assert!(!browser_ua("duckduckgo-cli/1.0"));
}

#[test]
fn validate_user_agent_rejects_empty_and_warns_browser_shapes() {
    let mut warnings = Vec::new();
    assert!(validate_user_agent(Some(String::new()), &mut warnings).is_err());
    assert_eq!(
        validate_user_agent(Some("Mozilla/5.0".to_owned()), &mut warnings).unwrap(),
        Some("Mozilla/5.0".to_owned())
    );
    assert_eq!(warnings.len(), 1);
}
