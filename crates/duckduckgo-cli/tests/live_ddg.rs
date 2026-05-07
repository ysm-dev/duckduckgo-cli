use predicates::prelude::*;

#[test]
#[ignore = "requires LIVE_DDG=1 and live DuckDuckGo access"]
fn live_rust_query_returns_rust_lang_when_enabled() {
    if std::env::var("LIVE_DDG").as_deref() != Ok("1") {
        return;
    }
    assert_cmd::Command::cargo_bin("duckduckgo")
        .unwrap()
        .args(["--no-rate-limit", "-n", "5", "rust programming language"])
        .assert()
        .success()
        .stdout(predicate::str::contains("rust-lang.org"));
}
