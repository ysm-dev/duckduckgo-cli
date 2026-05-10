# Changelog

All notable changes to this project will be documented in this file.

## [0.1.6]

### Changed

- Match `ddgr --noua` search requests more closely: post to `/html`
  without the trailing slash, send `Accept-Encoding: gzip` with empty
  `User-Agent` and `DNT: 1`, avoid replaying DDG cookies, allow the
  default redirect policy, and build subsequent-page form bodies using
  ddgr's field shape.

## [0.1.3]

### Changed

- Internal refactor of the CLI's `output/` module into per-concept files
  (`color.rs`, `plain.rs`, `mod.rs`) and the core's `rate_limit/` and
  `search/` modules into smaller per-concept files to honour the
  workspace's per-file line cap. No CLI flag, JSON envelope, or library
  surface change.
- `duckduckgo_core::ClientBuilder` now accepts injectable clock and
  rate-limit constants via `duckduckgo_core::clock` and
  `rate_limit::config`, used by tests to make rate-limit, cooldown, and
  spacing behaviour deterministic. Defaults are unchanged.

### Added

- 90% line-coverage gate (`cargo llvm-cov`) enforced in CI for both
  workspace crates, with new unit/integration suites covering args
  parsing, config env/flag/file/print/raw/validate paths,
  update-check HTTP, output color/plain rendering, parser block /
  decode / pagination / results edge cases, rate-limit
  store/post-flight/wait/progress/runner branches, and search
  client/fetch/form/http/state/types coverage.
- Parser fixtures (`tests/fixtures/anomaly-2026-05.html`,
  `results-2026-05.html`) refreshed and wired into deterministic unit
  tests.

### Fixed

- Root `duckduckgo-cli` npm package now pins its
  `optionalDependencies` to the matching `0.1.3` platform packages
  (previously stuck at `0.1.1` after the 0.1.2 bump).

## [0.1.2]

### Fixed

- Anomaly classifier no longer reports a soft-block on legitimate
  `HTTP 200` responses whose result snippets contain the bare word
  `"challenge"` (or `"captcha"`) — the kind of phrase that appears in
  ordinary DDG search results for queries like `rust async tutorial`.
  The classifier now matches DDG's actual anomaly-modal markers
  (`anomaly-modal__`, `anomaly_modal`, `/anomaly.js`, the literal
  "Unfortunately, bots use DuckDuckGo too." sentence, and
  `id="challenge-form"`), all taken from the captured anomaly response
  recorded in [`docs/en/ddgr.md`](docs/en/ddgr.md). Five new unit tests
  in `crates/duckduckgo-core/src/parser/block.rs` pin down the
  legitimate-vs-anomaly distinction. This unblocks parallel CLI loads
  that the rate-limit gate already prevented from bursting at the
  network layer.

### Added

- `docs/en/ddgr.md` first-party rate-limit field report (sliding-window
  counter at ≈ 8 starts with refill ≈ 1 / 1.5 s, soft-block returns
  `HTTP 202` with a 14 KB anomaly modal, recovery ≈ 60 s while
  probing). Referenced from spec.md §8 and used as the empirical basis
  for the v0.1.1 limiter constants.

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
