# duckduckgo-cli

Agent-first DuckDuckGo search from the command line.

## Install

```sh
npm i -g duckduckgo-cli
cargo install duckduckgo-cli
```

Homebrew and `curl | sh` release artifacts are produced by the release workflow.

## Quickstart

```sh
duckduckgo rust async tutorial
ddg -n 5 "best Rust web framework 2026"
duckduckgo --json -n 5 "best Rust web framework 2026"
duckduckgo -r kr-ko -t w "AI 규제"
echo "tokio runtime overhead" | duckduckgo --json
```

Plain text is the default and recommended surface for LLM agents. Use `--json` when a compact structured envelope is required.

## Configuration

The config file is JSONC at `${XDG_CONFIG_HOME:-$HOME/.config}/duckduckgo-cli/config.jsonc` on macOS/Linux and `%APPDATA%\duckduckgo-cli\config.jsonc` on Windows. Defaults are overridden by config, then environment variables, then flags.

```jsonc
{
  "region": "us-en",
  "num": 10,
  "safe": true,
  "time": null,
  "timeout": 30,
  "proxy": null,
  "user_agent": null,
  "retry": 3,
  "color": "auto",
  "no_update_check": false
}
```

Run `duckduckgo --print-config` to see the merged effective config.
