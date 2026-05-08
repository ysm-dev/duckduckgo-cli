use std::fs;
use std::path::PathBuf;

use duckduckgo_core::Region;
use duckduckgo_core::paths::RuntimePaths;
use httpmock::prelude::*;
use tempfile::TempDir;

use crate::config::{ColorWhen, Settings};
use crate::update_check::{check_auto, check_now};

fn settings(dir: &TempDir, github_url: String) -> Settings {
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
        quiet: true,
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
        ddg_url: "https://html.duckduckgo.com/html/".to_owned(),
        github_url,
    }
}

#[tokio::test(flavor = "current_thread")]
async fn check_now_fetches_and_writes_cache() {
    let dir = TempDir::new().unwrap();
    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(GET)
            .path("/repos/ysm-dev/duckduckgo-cli/releases/latest");
        then.status(200)
            .json_body_obj(&serde_json::json!({ "tag_name": "v9.9.9" }));
    });
    let settings = settings(&dir, server.base_url());
    check_now(&settings).await.unwrap();
    assert!(settings.paths.cache_dir.join("update.json").exists());
}

#[tokio::test(flavor = "current_thread")]
async fn check_auto_returns_early_for_fresh_cache() {
    let dir = TempDir::new().unwrap();
    let settings = settings(&dir, "http://127.0.0.1:1".to_owned());
    fs::create_dir_all(&settings.paths.cache_dir).unwrap();
    fs::write(
        settings.paths.cache_dir.join("update.json"),
        format!(
            r#"{{ "last_check": "{}" }}"#,
            time::OffsetDateTime::now_utc()
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap()
        ),
    )
    .unwrap();
    check_auto(&settings).await.unwrap();
}
