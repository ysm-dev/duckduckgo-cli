use is_terminal::IsTerminal;

use crate::config::{ColorWhen, Settings};

pub(crate) fn stdout(settings: &Settings) -> bool {
    match settings.color {
        ColorWhen::Always => true,
        ColorWhen::Never => false,
        ColorWhen::Auto => std::io::stdout().is_terminal(),
    }
}

pub(crate) fn stderr(settings: &Settings) -> bool {
    match settings.color {
        ColorWhen::Always => true,
        ColorWhen::Never => false,
        ColorWhen::Auto => std::io::stderr().is_terminal(),
    }
}

pub(crate) fn paint(value: &str, code: &str, enabled: bool) -> String {
    if enabled {
        format!("\u{1b}[{code}m{value}\u{1b}[0m")
    } else {
        value.to_owned()
    }
}
