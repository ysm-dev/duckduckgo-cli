# Changelog

All notable changes to this project will be documented in this file.

## [0.1.1]

### Added

- Transparent rate-limit progress messages on stderr while a request is
  gated. Format: `[INFO] rate-limit {kind} {elapsed}s/{total}s ({remaining}s left)`
  where `{kind}` is `cooldown` or `spacing`. Emitted once per acquire
  cycle when the total wait is ≥ 1 s; respects `--quiet` and is silent
  on the `--no-wait` abort path. Implements
  [`docs/en/spec.md`](docs/en/spec.md) §8 / §9.3.
- `duckduckgo_core::ClientBuilder::on_rate_limit_progress` and the
  public types `RateLimitProgress`, `RateLimitWait`, `ProgressHook` so
  library callers can subscribe to the same events without going
  through the CLI's stderr.
- Schema-2 rate-limit state (`next_allowed_at` / `slowdown_until` /
  `consecutive_blocks`) with backwards-compatible reader for the
  schema-1 token-bucket layout.
- `examples/ddg_probe.rs` for empirical rate-limit characterisation
  scenarios (preview / serial / burst / recovery / pulse).

### Changed

- `rate_limit` module split into per-concept files (`runner.rs`,
  `post_flight.rs`, `wait.rs`, `progress.rs`, `outcome.rs`, sibling
  test files) to honour the workspace's per-file line cap.
- `meta.rate_limit` JSON envelope replaces `tokens_remaining` with
  `next_allowed_at` / `slowdown_until` to match the spacing-based
  algorithm.

## [0.1.0]

### Added

- Initial Rust workspace with `duckduckgo-core` and `duckduckgo-cli`.
- One-shot DuckDuckGo web search CLI with plain-text and JSON output.
- Config, environment, stdin, rate-limit state, update-check, and npm shim support.
