use std::fs;

use duckduckgo_core::paths::RuntimePaths;
use tempfile::TempDir;

use super::file::load_file;
use super::raw::Raw;

fn paths(dir: &TempDir) -> RuntimePaths {
    RuntimePaths {
        config_file: dir.path().join("config.jsonc"),
        state_dir: dir.path().join("state"),
        cache_dir: dir.path().join("cache"),
    }
}

#[test]
fn missing_config_file_is_silent() {
    let dir = TempDir::new().unwrap();
    let mut raw = Raw::default();
    let mut warnings = Vec::new();
    load_file(&paths(&dir), &mut raw, &mut warnings).unwrap();
    assert!(raw.region.is_none());
    assert!(warnings.is_empty());
}

#[test]
fn jsonc_config_populates_raw_values_and_unknown_warnings() {
    let dir = TempDir::new().unwrap();
    let paths = paths(&dir);
    fs::write(
        &paths.config_file,
        r#"{ // comment
          "region": "uk-en", "num": 7, "safe": false,
          "retry": 2, "unknown": "ignored"
        }"#,
    )
    .unwrap();
    let mut raw = Raw::default();
    let mut warnings = Vec::new();
    load_file(&paths, &mut raw, &mut warnings).unwrap();
    assert_eq!(raw.region.as_deref(), Some("uk-en"));
    assert_eq!(raw.num.as_deref(), Some("7"));
    assert_eq!(raw.safe, Some(false));
    assert_eq!(raw.retry.as_deref(), Some("2"));
    assert_eq!(warnings, vec!["Unknown config key 'unknown' ignored."]);
}

#[test]
fn config_type_errors_are_rejected() {
    let dir = TempDir::new().unwrap();
    let paths = paths(&dir);
    fs::write(&paths.config_file, r#"{ "num": "ten" }"#).unwrap();
    let err = load_file(&paths, &mut Raw::default(), &mut Vec::new()).unwrap_err();
    assert!(err.to_string().contains("Config key 'num'"));
}

#[test]
fn malformed_and_non_object_config_are_usage_errors() {
    let dir = TempDir::new().unwrap();
    let paths = paths(&dir);
    fs::write(&paths.config_file, "{").unwrap();
    assert!(load_file(&paths, &mut Raw::default(), &mut Vec::new()).is_err());

    fs::write(&paths.config_file, "[]").unwrap();
    let err = load_file(&paths, &mut Raw::default(), &mut Vec::new()).unwrap_err();
    assert!(err.to_string().contains("root must be an object"));
}
