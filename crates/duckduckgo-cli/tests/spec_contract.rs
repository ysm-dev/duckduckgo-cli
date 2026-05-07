use std::fs;

use httpmock::prelude::*;
use predicates::prelude::*;
use serde_json::Value;
use tempfile::TempDir;

fn cmd(home: &TempDir) -> assert_cmd::Command {
    let mut command = assert_cmd::Command::cargo_bin("duckduckgo").unwrap();
    command
        .env("HOME", home.path())
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
        .env("XDG_CONFIG_HOME", home.path().join("config"))
        .env("XDG_STATE_HOME", home.path().join("state"))
        .env("XDG_CACHE_HOME", home.path().join("cache"))
        .env_remove("HTTPS_PROXY")
        .env_remove("HTTP_PROXY")
        .env_remove("ALL_PROXY");
    command
}

fn ddg_fixture() -> &'static str {
    r#"<!doctype html><html><body>
    <div class="zci__result">Rust is a systems programming language.</div>
    <div class="result results_links web-result">
      <div class="links_main result__body">
        <a class="result__a" href="https://duckduckgo.com/l/?uddg=https%3A%2F%2Fwww.rust-lang.org%2F">Rust Programming Language</a>
        <a class="result__snippet">A language empowering everyone to build reliable and efficient software.</a>
      </div>
    </div>
    <div class="result results_links web-result">
      <div class="links_main result__body">
        <a class="result__a" href="https://doc.rust-lang.org/book/">The Rust Book</a>
        <a class="result__snippet">The book teaches Rust from installation through ownership.</a>
      </div>
    </div>
    </body></html>"#
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
        .stdout(predicate::str::contains("duckduckgo-cli 0.1.0"));
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
        .stdout(predicate::str::contains("duckduckgo-cli 0.1.0"));
    alias_cmd(&home, "duckduckgo-cli")
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("duckduckgo-cli 0.1.0"));
}

#[test]
fn print_config_merges_file_env_and_flags() {
    let home = TempDir::new().unwrap();
    let cfg_dir = home.path().join("config/duckduckgo-cli");
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
        then.status(200)
            .body("<!doctype html><p>No results found.</p>");
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
        then.status(200).body(ddg_fixture());
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
    let state_dir = home.path().join("state/duckduckgo-cli");
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
        then.status(202).body("anomaly_modal");
    });
    cmd(&home)
        .env("DUCKDUCKGO_DDG_URL", format!("{}/html/", server.base_url()))
        .args(["--no-rate-limit", "--retry", "0", "rust"])
        .assert()
        .code(5)
        .stderr(predicate::str::contains("blocked"));
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
    assert!(
        home.path()
            .join("cache/duckduckgo-cli/update.json")
            .exists()
    );
}
