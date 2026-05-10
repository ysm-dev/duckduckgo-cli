use std::fs;
use std::path::PathBuf;

use duckduckgo_core::Region;
use duckduckgo_core::paths::RuntimePaths;
use httpmock::prelude::*;
use tempfile::TempDir;
use time::macros::datetime;

use crate::config::{ColorWhen, Settings};
use crate::update_check::{
    cache_fresh_at, cache_value_fresh, fetch_latest, previous_success, version_gt,
};

fn test_settings(dir: &TempDir, github_url: String) -> Settings {
    Settings {
        region: Region::default(),
        num: 10,
        page: 1,
        safe: true,
        time: None,
        sites: Vec::new(),
        json: false,
        color: ColorWhen::Never,
        verbose: 0,
        quiet: false,
        timeout: 30,
        proxy: None,
        user_agent: None,
        retry: 3,
        no_wait: false,
        no_update_check: false,
        rate_limit: true,
        paths: RuntimePaths {
            config_file: PathBuf::from("config.jsonc"),
            state_dir: dir.path().join("state"),
            cache_dir: dir.path().join("cache"),
        },
        warnings: Vec::new(),
        ddg_url: "https://html.duckduckgo.com/html".to_owned(),
        github_url,
    }
}

#[test]
fn version_gt_handles_v_prefix_and_missing_components() {
    assert!(version_gt("v1.2.4", "1.2.3"));
    assert!(version_gt("1.3", "1.2.9"));
    assert!(!version_gt("v1.2.3", "1.2.3"));
    assert!(!version_gt("1.2.3", "1.2.4"));
}

#[test]
fn version_gt_ignores_non_numeric_suffixes() {
    assert!(!version_gt("v1.2.3-beta", "1.2.3"));
    assert!(version_gt("v2.0.0", "1.99.99"));
}

#[test]
fn cache_value_fresh_uses_24_hour_window() {
    let now = datetime!(2026-05-09 12:00 UTC);
    let fresh = r#"{ "last_check": "2026-05-08T12:00:01Z" }"#;
    let stale = r#"{ "last_check": "2026-05-08T12:00:00Z" }"#;
    assert!(cache_value_fresh(fresh, now));
    assert!(!cache_value_fresh(stale, now));
    assert!(!cache_value_fresh("not json", now));
    assert!(!cache_value_fresh(r#"{ "last_check": "bad" }"#, now));
}

#[test]
fn previous_success_reads_only_valid_success_value() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("update.json");
    assert_eq!(previous_success(&path), None);
    fs::write(&path, r#"{ "last_success": "v1.2.3" }"#).unwrap();
    assert_eq!(previous_success(&path), Some("v1.2.3".to_owned()));
    fs::write(&path, "not json").unwrap();
    assert_eq!(previous_success(&path), None);
}

#[test]
fn cache_fresh_at_reads_cache_file() {
    let dir = TempDir::new().unwrap();
    let settings = test_settings(&dir, "https://api.github.com".to_owned());
    fs::create_dir_all(&settings.paths.cache_dir).unwrap();
    fs::write(
        settings.paths.cache_dir.join("update.json"),
        r#"{ "last_check": "2026-05-09T11:59:59Z" }"#,
    )
    .unwrap();
    assert!(cache_fresh_at(&settings, datetime!(2026-05-09 12:00 UTC)));
}

#[tokio::test(flavor = "current_thread")]
async fn fetch_latest_reads_release_tag() {
    let dir = TempDir::new().unwrap();
    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(GET)
            .path("/repos/ysm-dev/duckduckgo-cli/releases/latest");
        then.status(200)
            .json_body_obj(&serde_json::json!({ "tag_name": "v9.9.9" }));
    });
    let settings = test_settings(&dir, server.base_url());
    assert_eq!(fetch_latest(&settings).await.unwrap(), "v9.9.9");
}

#[tokio::test(flavor = "current_thread")]
async fn fetch_latest_rejects_http_error_and_missing_tag() {
    let dir = TempDir::new().unwrap();
    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(GET)
            .path("/repos/ysm-dev/duckduckgo-cli/releases/latest");
        then.status(500).body("{} ");
    });
    let settings = test_settings(&dir, server.base_url());
    assert!(
        fetch_latest(&settings)
            .await
            .unwrap_err()
            .to_string()
            .contains("GitHub HTTP")
    );

    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(GET)
            .path("/repos/ysm-dev/duckduckgo-cli/releases/latest");
        then.status(200).json_body_obj(&serde_json::json!({}));
    });
    let settings = test_settings(&dir, server.base_url());
    assert!(
        fetch_latest(&settings)
            .await
            .unwrap_err()
            .to_string()
            .contains("Missing tag_name")
    );
}
