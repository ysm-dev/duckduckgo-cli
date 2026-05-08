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

#[cfg(test)]
mod tests {
    use super::paint;

    #[test]
    fn paint_is_identity_when_disabled() {
        assert_eq!(paint("value", "31", false), "value");
    }

    #[test]
    fn paint_wraps_ansi_when_enabled() {
        assert_eq!(paint("value", "31", true), "\u{1b}[31mvalue\u{1b}[0m");
    }
}
