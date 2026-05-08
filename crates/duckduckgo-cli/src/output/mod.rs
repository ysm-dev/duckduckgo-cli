mod color;
mod plain;

use is_terminal::IsTerminal;

use duckduckgo_core::{Result, SearchResponse};

use crate::config::Settings;

/// Minimal stderr-emission context for code paths that fire from
/// background callbacks (e.g. the rate-limit progress hook) and
/// therefore cannot borrow `Settings` for their full lifetime. Captures
/// only the fields `info_with` / `warn_with` need.
#[derive(Clone, Copy, Debug)]
pub struct OutputCtx {
    pub quiet: bool,
    pub stderr_color: bool,
}

impl OutputCtx {
    pub fn from_settings(settings: &Settings) -> Self {
        Self {
            quiet: settings.quiet,
            stderr_color: color::stderr(settings),
        }
    }
}

pub fn render(settings: &Settings, response: &SearchResponse) -> Result<()> {
    if settings.json {
        println!(
            "{}",
            serde_json::to_string(response)
                .map_err(|e| duckduckgo_core::Error::Io(e.to_string()))?
        );
    } else {
        print!("{}", plain::render(response, color::stdout(settings)));
    }
    Ok(())
}

pub fn emit_warnings(settings: &Settings) {
    if settings.quiet {
        return;
    }
    if settings.verbose >= 2 {
        info(settings, "Effective configuration loaded.");
    }
    for warning in &settings.warnings {
        warn(settings, warning);
    }
}

pub fn warn(settings: &Settings, message: &str) {
    warn_with(&OutputCtx::from_settings(settings), message);
}

pub fn info(settings: &Settings, message: &str) {
    info_with(&OutputCtx::from_settings(settings), message);
}

pub fn warn_with(ctx: &OutputCtx, message: &str) {
    if !ctx.quiet {
        eprintln!(
            "{} {message}",
            color::paint("[WARN]", "33", ctx.stderr_color)
        );
    }
}

pub fn info_with(ctx: &OutputCtx, message: &str) {
    if !ctx.quiet {
        eprintln!(
            "{} {message}",
            color::paint("[INFO]", "36", ctx.stderr_color)
        );
    }
}

pub fn should_auto_update(settings: &Settings) -> bool {
    !settings.no_update_check
        && !settings.json
        && std::io::stdin().is_terminal()
        && std::io::stdout().is_terminal()
        && std::io::stderr().is_terminal()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use duckduckgo_core::Region;
    use duckduckgo_core::paths::RuntimePaths;

    use crate::config::{ColorWhen, Settings};

    use super::{OutputCtx, should_auto_update};

    fn settings(color: ColorWhen) -> Settings {
        Settings {
            region: Region::default(),
            num: 10,
            page: 1,
            safe: true,
            time: None,
            sites: Vec::new(),
            json: false,
            color,
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
                state_dir: PathBuf::from("state"),
                cache_dir: PathBuf::from("cache"),
            },
            warnings: Vec::new(),
            ddg_url: "https://html.duckduckgo.com/html/".to_owned(),
            github_url: "https://api.github.com".to_owned(),
        }
    }

    #[test]
    fn output_context_captures_quiet_and_color() {
        let mut settings = settings(ColorWhen::Always);
        settings.quiet = true;
        let ctx = OutputCtx::from_settings(&settings);
        assert!(ctx.quiet);
        assert!(ctx.stderr_color);
    }

    #[test]
    fn auto_update_short_circuits_for_disabled_or_json_output() {
        let mut settings = settings(ColorWhen::Never);
        settings.no_update_check = true;
        assert!(!should_auto_update(&settings));
        settings.no_update_check = false;
        settings.json = true;
        assert!(!should_auto_update(&settings));
    }
}
