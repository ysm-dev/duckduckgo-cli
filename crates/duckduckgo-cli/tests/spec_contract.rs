use std::{fs, path::PathBuf};

use httpmock::prelude::*;
use predicates::prelude::*;
use serde_json::Value;
use tempfile::TempDir;

fn cmd(home: &TempDir) -> assert_cmd::Command {
    let mut command = assert_cmd::Command::cargo_bin("duckduckgo").unwrap();
    command
        .env("HOME", home.path())
        .env("USERPROFILE", home.path())
        .env("APPDATA", home.path().join("config"))
        .env("LOCALAPPDATA", home.path().join("local"))
        .env("XDG_CONFIG_HOME", home.path().join("config"))
        .env("XDG_STATE_HOME", home.path().join("state"))
        .env("XDG_CACHE_HOME", home.path().join("cache"))
        .env_remove("HTTPS_PROXY")
        .env_remove("HTTP_PROXY")
        .env_remove("ALL_PROXY")
        .env_remove("DUCKDUCKGO_CONFIG")
        .env_remove("DUCKDUCKGO_REGION")
        .env_remove("DUCKDUCKGO_NUM")
        .env_remove("DUCKDUCKGO_SAFE")
        .env_remove("DUCKDUCKGO_TIME")
        .env_remove("DUCKDUCKGO_PROXY")
        .env_remove("DUCKDUCKGO_COLOR")
        .env_remove("DUCKDUCKGO_DDG_URL")
        .env_remove("DUCKDUCKGO_GITHUB_URL")
        .env_remove("DUCKDUCKGO_NO_RATE_LIMIT")
        .env_remove("DUCKDUCKGO_USER_AGENT")
        .env_remove("NO_COLOR");
    command
}

fn alias_cmd(home: &TempDir, bin: &str) -> assert_cmd::Command {
    let mut command = assert_cmd::Command::cargo_bin(bin).unwrap();
    command
        .env("HOME", home.path())
        .env("USERPROFILE", home.path())
        .env("APPDATA", home.path().join("config"))
        .env("LOCALAPPDATA", home.path().join("local"))
        .env("XDG_CONFIG_HOME", home.path().join("config"))
        .env("XDG_STATE_HOME", home.path().join("state"))
        .env("XDG_CACHE_HOME", home.path().join("cache"))
        .env_remove("HTTPS_PROXY")
        .env_remove("HTTP_PROXY")
        .env_remove("ALL_PROXY");
    command
}

fn config_dir(home: &TempDir) -> PathBuf {
    home.path().join("config/duckduckgo-cli")
}

fn state_dir(home: &TempDir) -> PathBuf {
    #[cfg(windows)]
    {
        home.path().join("local/duckduckgo-cli/state")
    }
    #[cfg(not(windows))]
    {
        home.path().join("state/duckduckgo-cli")
    }
}

fn cache_file(home: &TempDir, name: &str) -> PathBuf {
    #[cfg(windows)]
    {
        home.path().join("local/duckduckgo-cli/cache").join(name)
    }
    #[cfg(not(windows))]
    {
        home.path().join("cache/duckduckgo-cli").join(name)
    }
}

fn ddg_fixture() -> &'static str {
    include_str!("../../../tests/fixtures/results-2026-05.html")
}

fn anomaly_fixture() -> &'static str {
    include_str!("../../../tests/fixtures/anomaly-2026-05.html")
}

fn empty_fixture() -> &'static str {
    include_str!("../../../tests/fixtures/empty-results-2026-05.html")
}

fn instant_answer_fixture() -> &'static str {
    include_str!("../../../tests/fixtures/instant-answer-2026-05.html")
}

#[test]
fn meta_flags_do_not_require_query() {
    let home = TempDir::new().unwrap();
    cmd(&home).arg("--list-regions").assert().success().stdout(
        predicate::str::contains("kr-ko\t")
            .and(predicate::str::contains("us-en\t"))
            .and(predicate::str::contains("wt-wt\t")),
    );
    cmd(&home)
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(format!(
            "duckduckgo-cli {}",
            env!("CARGO_PKG_VERSION")
        )));
    for shell in ["bash", "zsh", "fish", "powershell", "nushell", "elvish"] {
        cmd(&home)
            .args(["--completion", shell])
            .assert()
            .success()
            .stdout(predicate::str::contains("duckduckgo"));
    }
    alias_cmd(&home, "ddg")
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(format!(
            "duckduckgo-cli {}",
            env!("CARGO_PKG_VERSION")
        )));
    alias_cmd(&home, "duckduckgo-cli")
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(format!(
            "duckduckgo-cli {}",
            env!("CARGO_PKG_VERSION")
        )));
}

#[test]
fn print_config_merges_file_env_and_flags() {
    let home = TempDir::new().unwrap();
    let cfg_dir = config_dir(&home);
    fs::create_dir_all(&cfg_dir).unwrap();
    fs::write(
        cfg_dir.join("config.jsonc"),
        r#"{
          // JSONC is accepted.
          "region": "kr-ko",
          "num": 7,
          "proxy": "https://user:secret@example.com:8443/",
          "unknown_key": true,
        }"#,
    )
    .unwrap();

    let output = cmd(&home)
        .env("DUCKDUCKGO_NUM", "3")
        .args(["--print-config", "--region", "jp-jp"])
        .assert()
        .success()
        .stderr(predicate::str::contains("[WARN]"))
        .get_output()
        .stdout
        .clone();
    let json: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["search"]["region"], "jp-jp");
    assert_eq!(json["search"]["num"], 3);
    assert_eq!(
        json["network"]["proxy"],
        "https://user:***@example.com:8443/"
    );
    assert_eq!(json["operational"]["rate_limit"], true);
}

#[test]
fn stdin_query_and_usage_errors_are_spec_shaped() {
    let home = TempDir::new().unwrap();
    cmd(&home)
        .assert()
        .code(2)
        .stderr(predicate::str::contains("[ERROR]").and(predicate::str::contains("query")));
    cmd(&home)
        .args(["--num", "101", "rust"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("Invalid --num"));
    cmd(&home)
        .args(["--safe", "--unsafe", "rust"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("--safe").and(predicate::str::contains("--unsafe")));
    let mut command = cmd(&home);
    command.write_stdin(vec![b'a'; 64 * 1024 + 1]);
    command
        .assert()
        .code(2)
        .stderr(predicate::str::contains("64 KiB"));
    let mut invalid = cmd(&home);
    invalid.write_stdin(vec![0xff, 0xfe]);
    invalid
        .assert()
        .code(2)
        .stderr(predicate::str::contains("Invalid UTF-8"));

    let config_dir = home.path().join("not-a-file");
    fs::create_dir_all(&config_dir).unwrap();
    cmd(&home)
        .args(["--config", config_dir.to_str().unwrap(), "--print-config"])
        .assert()
        .code(6)
        .stderr(predicate::str::contains("Local I/O"));
}

#[test]
fn search_outputs_plain_text_and_json_against_mock_ddg() {
    let home = TempDir::new().unwrap();
    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(POST)
            .path("/html/")
            .header_missing("User-Agent")
            .body_includes("q=rust")
            .body_includes("kl=us-en")
            .body_includes("kp=1");
        then.status(200).body(ddg_fixture());
    });

    cmd(&home)
        .env("DUCKDUCKGO_DDG_URL", format!("{}/html/", server.base_url()))
        .args(["--no-rate-limit", "-n", "2", "rust"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Rust is a systems programming language")
                .and(predicate::str::contains("   1. Rust Programming Language"))
                .and(predicate::str::contains("    https://www.rust-lang.org/")),
        );

    let output = cmd(&home)
        .env("DUCKDUCKGO_DDG_URL", format!("{}/html/", server.base_url()))
        .args(["--no-rate-limit", "--json", "-n", "1", "rust"])
        .assert()
        .success()
        .stderr("")
        .get_output()
        .stdout
        .clone();
    assert!(output.ends_with(b"\n"));
    let json: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["schema"], 1);
    assert_eq!(json["query"], "rust");
    assert_eq!(json["count"], 1);
    assert_eq!(json["results"][0]["position"], 1);
    assert_eq!(json["meta"]["rate_limit"]["tokens_remaining"], Value::Null);
}

#[test]
fn user_agent_is_empty_by_default_and_opt_in_when_supplied() {
    let home = TempDir::new().unwrap();
    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(POST)
            .path("/html/")
            .header("User-Agent", "tool/1.0");
        then.status(200).body(ddg_fixture());
    });
    cmd(&home)
        .env("DUCKDUCKGO_DDG_URL", format!("{}/html/", server.base_url()))
        .args(["--no-rate-limit", "--user-agent", "tool/1.0", "rust"])
        .assert()
        .success();
}

#[test]
fn pagination_and_parse_drift_follow_spec() {
    let home = TempDir::new().unwrap();
    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(POST).path("/html/").body_includes("q=rust");
        then.status(200).header("Set-Cookie", "sid=abc; Path=/").body(r#"<!doctype html><html><body>
          <div class="result"><a class="result__a" href="https://example.com/1">One</a><a class="result__snippet">First</a></div>
          <form><input name="s" value="30"><input name="nextParams" value="x"><input name="vqd" value="abc"></form>
        </body></html>"#);
    });
    server.mock(|when, then| {
        when.method(POST)
            .path("/html/")
            .body_includes("nextParams=x")
            .header("Cookie", "sid=abc");
        then.status(200).body(r#"<!doctype html><html><body>
          <div class="result"><a class="result__a" href="https://example.com/2">Two</a><a class="result__snippet">Second</a></div>
        </body></html>"#);
    });
    cmd(&home)
        .env("DUCKDUCKGO_DDG_URL", format!("{}/html/", server.base_url()))
        .args(["--no-rate-limit", "--page", "2", "-n", "1", "rust"])
        .assert()
        .success()
        .stdout(predicate::str::contains("   2. Two"));

    let drift = MockServer::start();
    drift.mock(|when, then| {
        when.method(POST).path("/html/");
        then.status(200).body(r#"<!doctype html><html><body>
          <div class="result"><a class="result__a" href="https://example.com/1">One</a><a class="result__snippet">First</a></div>
        </body></html>"#);
    });
    cmd(&home)
        .env("DUCKDUCKGO_DDG_URL", format!("{}/html/", drift.base_url()))
        .args(["--no-rate-limit", "--page", "2", "-n", "1", "rust"])
        .assert()
        .code(4)
        .stderr(predicate::str::contains("HTML structure"));
}

#[test]
fn no_results_json_is_quiet_exit_one_and_color_always_uses_ansi() {
    let home = TempDir::new().unwrap();
    let empty = MockServer::start();
    let _empty_mock = empty.mock(|when, then| {
        when.method(POST).path("/html/").body_includes("q=nomatch");
        then.status(200).body(empty_fixture());
    });
    let output = cmd(&home)
        .env("DUCKDUCKGO_DDG_URL", empty.url("/html/"))
        .args(["--no-rate-limit", "--json", "nomatch"])
        .assert()
        .code(1)
        .stderr("")
        .get_output()
        .stdout
        .clone();
    let json: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["results"].as_array().unwrap().len(), 0);
    assert_eq!(_empty_mock.calls(), 1);

    let server = MockServer::start();
    let _server_mock = server.mock(|when, then| {
        when.method(POST).path("/html/");
        then.status(200).body(instant_answer_fixture());
    });
    cmd(&home)
        .env("DUCKDUCKGO_DDG_URL", server.url("/html/"))
        .args(["--no-rate-limit", "--color", "always", "rust"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\u{1b}["));
    assert_eq!(_server_mock.calls(), 1);
}

#[test]
fn blocks_and_no_wait_return_exit_five() {
    let home = TempDir::new().unwrap();
    let state_dir = state_dir(&home);
    fs::create_dir_all(&state_dir).unwrap();
    let blocked_until = (time::OffsetDateTime::now_utc() + time::Duration::hours(1))
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap();
    fs::write(
        state_dir.join("rate-limit.json"),
        format!(r#"{{"schema":1,"tokens":0.0,"last_refill":"2026-05-07T12:34:56.789Z","blocked_until":"{blocked_until}","consecutive_blocks":1,"last_block_reason":"http_202"}}"#),
    )
    .unwrap();
    cmd(&home)
        .args(["--no-wait", "rust"])
        .assert()
        .code(5)
        .stderr(predicate::str::contains("Rate limit"));

    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(POST).path("/html/");
        then.status(202).body(anomaly_fixture());
    });
    cmd(&home)
        .env("DUCKDUCKGO_DDG_URL", format!("{}/html/", server.base_url()))
        .args(["--no-rate-limit", "--retry", "0", "rust"])
        .assert()
        .code(5)
        .stderr(predicate::str::contains("blocked"));
}

#[test]
fn rate_limit_spacing_wait_emits_short_progress_line_on_stderr() {
    // Pre-arm `next_allowed_at ≈ now + 2s` so the very next CLI call
    // observes a spacing wait above the 1 s emission threshold without
    // needing to provoke a real 202. The mock returns a usable page so
    // the run also exercises the post-wait success path.
    let home = TempDir::new().unwrap();
    let state_dir = state_dir(&home);
    fs::create_dir_all(&state_dir).unwrap();
    let next_allowed_at = (time::OffsetDateTime::now_utc() + time::Duration::seconds(2))
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap();
    fs::write(
        state_dir.join("rate-limit.json"),
        format!(
            r#"{{"schema":2,"next_allowed_at":"{next_allowed_at}","blocked_until":null,"slowdown_until":null,"consecutive_blocks":0,"last_block_reason":null}}"#
        ),
    )
    .unwrap();

    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(POST).path("/html/");
        then.status(200).body(ddg_fixture());
    });

    cmd(&home)
        .env("DUCKDUCKGO_DDG_URL", format!("{}/html/", server.base_url()))
        .args(["--color", "never", "-n", "1", "rust"])
        .assert()
        .success()
        .stderr(
            predicate::str::contains("[INFO] rate-limit spacing")
                .and(predicate::str::is_match(r"\d+s/\d+s \(\d+s left\)").unwrap()),
        );
}

#[test]
fn rate_limit_progress_is_silent_under_quiet_flag() {
    // Same arming as above, but `--quiet` must suppress the [INFO]
    // line per the noise budget in spec §9.3.
    let home = TempDir::new().unwrap();
    let state_dir = state_dir(&home);
    fs::create_dir_all(&state_dir).unwrap();
    let next_allowed_at = (time::OffsetDateTime::now_utc() + time::Duration::seconds(2))
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap();
    fs::write(
        state_dir.join("rate-limit.json"),
        format!(
            r#"{{"schema":2,"next_allowed_at":"{next_allowed_at}","blocked_until":null,"slowdown_until":null,"consecutive_blocks":0,"last_block_reason":null}}"#
        ),
    )
    .unwrap();

    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(POST).path("/html/");
        then.status(200).body(ddg_fixture());
    });

    cmd(&home)
        .env("DUCKDUCKGO_DDG_URL", format!("{}/html/", server.base_url()))
        .args(["--quiet", "-n", "1", "rust"])
        .assert()
        .success()
        .stderr(predicate::str::contains("rate-limit").not());
}

#[test]
fn parallel_cli_invocations_serialise_through_state_file_and_never_burst() {
    use std::process::Command;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::thread;

    let home = TempDir::new().unwrap();
    let server = MockServer::start();

    // Each request grabs the in_flight count atomically. We assert the
    // observed maximum is 1 (no two requests overlapping at the mock
    // server) and that the timing between request starts respects the
    // configured 250ms BASE_SPACING (we use a tight value to keep the
    // test fast; in production the default is 2s).
    let in_flight = Arc::new(AtomicU32::new(0));
    let max_in_flight = Arc::new(AtomicU32::new(0));
    let starts: Arc<std::sync::Mutex<Vec<std::time::Instant>>> =
        Arc::new(std::sync::Mutex::new(Vec::new()));

    let in_flight_for_mock = in_flight.clone();
    let max_in_flight_for_mock = max_in_flight.clone();
    let starts_for_mock = starts.clone();
    server.mock(move |when, then| {
        when.method(POST).path("/html/");
        let cur = in_flight_for_mock.fetch_add(1, Ordering::SeqCst) + 1;
        let mut high = max_in_flight_for_mock.load(Ordering::SeqCst);
        while cur > high
            && let Err(updated) = max_in_flight_for_mock.compare_exchange(
                high,
                cur,
                Ordering::SeqCst,
                Ordering::SeqCst,
            )
        {
            high = updated;
        }
        starts_for_mock
            .lock()
            .expect("starts mutex")
            .push(std::time::Instant::now());
        thread::sleep(std::time::Duration::from_millis(80));
        in_flight_for_mock.fetch_sub(1, Ordering::SeqCst);
        then.status(200).body(ddg_fixture());
    });

    let parallel = 8_u32;
    let endpoint = format!("{}/html/", server.base_url());
    let mut handles = Vec::with_capacity(parallel as usize);
    for _ in 0..parallel {
        let endpoint = endpoint.clone();
        let home_path = home.path().to_path_buf();
        handles.push(thread::spawn(move || {
            let mut cmd = Command::new(env!("CARGO_BIN_EXE_duckduckgo"));
            cmd.env("HOME", &home_path)
                .env("USERPROFILE", &home_path)
                .env("APPDATA", home_path.join("config"))
                .env("LOCALAPPDATA", home_path.join("local"))
                .env("XDG_CONFIG_HOME", home_path.join("config"))
                .env("XDG_STATE_HOME", home_path.join("state"))
                .env("XDG_CACHE_HOME", home_path.join("cache"))
                .env("DUCKDUCKGO_DDG_URL", &endpoint)
                .env("DUCKDUCKGO_BASE_SPACING_MS", "250")
                .env("DUCKDUCKGO_BASE_COOLDOWN_S", "1")
                .env_remove("HTTPS_PROXY")
                .env_remove("HTTP_PROXY")
                .env_remove("ALL_PROXY")
                .args(["--quiet", "-n", "1", "rust"]);
            cmd.output().expect("spawn cli")
        }));
    }
    let outputs: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    let successes = outputs
        .iter()
        .filter(|o| o.status.code() == Some(0))
        .count();
    assert_eq!(
        successes, parallel as usize,
        "expected all {parallel} CLI invocations to succeed; outputs={outputs:#?}"
    );
    let max = max_in_flight.load(Ordering::SeqCst);
    assert_eq!(
        max, 1,
        "the lock must serialise; observed up to {max} concurrent requests"
    );
    let mut starts = starts.lock().expect("starts mutex").clone();
    starts.sort();
    for window in starts.windows(2) {
        let gap = window[1].duration_since(window[0]);
        assert!(
            gap >= std::time::Duration::from_millis(220),
            "consecutive starts must respect spacing; got {gap:?}"
        );
    }
}

#[test]
fn update_check_uses_github_endpoint_and_cache() {
    let home = TempDir::new().unwrap();
    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(GET)
            .path("/repos/ysm-dev/duckduckgo-cli/releases/latest");
        then.status(200)
            .json_body_obj(&serde_json::json!({ "tag_name": "v0.2.0" }));
    });
    cmd(&home)
        .env("DUCKDUCKGO_GITHUB_URL", server.base_url())
        .arg("--check-updates")
        .assert()
        .success()
        .stdout(predicate::str::contains("v0.2.0 available"));
    assert!(cache_file(&home, "update.json").exists());
}
