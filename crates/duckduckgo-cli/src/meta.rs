use duckduckgo_core::{Error, Result};

use crate::args::{self, Cli};

pub fn dispatch(cli: &Cli) -> Result<bool> {
    if cli.help_short || cli.help_long {
        print_help(cli.help_long);
        return Ok(true);
    }
    if cli.version {
        println!(
            "duckduckgo-cli {} (commit unknown, target {})",
            env!("CARGO_PKG_VERSION"),
            env!("DUCKDUCKGO_TARGET")
        );
        return Ok(true);
    }
    if let Some(shell) = &cli.completion {
        args::print_completion(shell)?;
        return Ok(true);
    }
    if cli.list_regions {
        for (code, name) in duckduckgo_core::region::REGION_CODES {
            println!("{code}\t{name}");
        }
        return Ok(true);
    }
    Ok(false)
}

fn print_help(long: bool) {
    if long {
        println!("{}", args::long_help());
    } else {
        println!("{}", args::short_help());
    }
}

pub fn clap_usage(error: clap::Error) -> Error {
    Error::Usage(error.to_string())
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::{clap_usage, dispatch};
    use crate::args::Cli;

    #[test]
    fn dispatch_ignores_search_invocation() {
        let cli = Cli::parse_from(["duckduckgo", "rust"]);
        assert!(!dispatch(&cli).unwrap());
    }

    #[test]
    fn dispatch_rejects_invalid_completion_shell() {
        let cli = Cli::parse_from(["duckduckgo", "--completion", "bogus"]);
        let err = dispatch(&cli).unwrap_err();
        assert!(err.to_string().contains("Invalid --completion"));
    }

    #[test]
    fn clap_usage_maps_to_usage_error() {
        let err = Cli::try_parse_from(["duckduckgo", "--unknown"]).unwrap_err();
        assert_eq!(clap_usage(err).exit_code(), 2);
    }
}
