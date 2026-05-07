mod color;
mod plain;

use is_terminal::IsTerminal;

use duckduckgo_core::{Result, SearchResponse};

use crate::config::Settings;

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
    if !settings.quiet {
        eprintln!(
            "{} {message}",
            color::paint("[WARN]", "33", color::stderr(settings))
        );
    }
}

pub fn info(settings: &Settings, message: &str) {
    if !settings.quiet {
        eprintln!(
            "{} {message}",
            color::paint("[INFO]", "36", color::stderr(settings))
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
