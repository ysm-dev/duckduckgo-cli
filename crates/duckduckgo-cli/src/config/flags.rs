use duckduckgo_core::{Error, Result};

use crate::args::{Cli, retry_from_args};

use super::raw::Raw;

pub(crate) fn apply_flags(cli: &Cli, raw: &mut Raw) -> Result<()> {
    apply_flags_with_args(cli, raw, std::env::args().skip(1))
}

fn apply_flags_with_args(
    cli: &Cli,
    raw: &mut Raw,
    args: impl IntoIterator<Item = String>,
) -> Result<()> {
    if cli.safe && cli.unsafe_search {
        return Err(Error::Usage(
            "--safe and --unsafe are mutually exclusive".to_owned(),
        ));
    }
    if let Some(v) = &cli.region {
        raw.region = Some(v.clone());
    }
    if let Some(v) = &cli.num {
        raw.num = Some(v.clone());
    }
    if let Some(v) = &cli.page {
        raw.page = Some(v.clone());
    }
    if let Some(v) = &cli.time {
        raw.time = Some(v.clone());
    }
    if cli.safe {
        raw.safe = Some(true);
    }
    if cli.unsafe_search {
        raw.safe = Some(false);
    }
    if let Some(v) = &cli.color {
        raw.color = Some(v.clone());
    }
    if let Some(v) = &cli.timeout {
        raw.timeout = Some(v.clone());
    }
    if let Some(v) = &cli.proxy {
        raw.proxy = Some(v.clone());
    }
    if let Some(v) = &cli.user_agent {
        raw.user_agent = Some(v.clone());
    }
    if let Some(v) = &cli.retry {
        raw.retry = Some(v.clone());
    }
    if cli.no_retry {
        raw.retry = Some("0".to_owned());
    }
    if let Some(v) = retry_from_args(args) {
        raw.retry = Some(v);
    }
    if cli.no_update_check {
        raw.no_update_check = Some(true);
    }
    if cli.no_rate_limit {
        raw.no_rate_limit = Some(true);
    }
    if cli.verbose > 0 {
        raw.verbose = Some(cli.verbose);
    }
    if cli.quiet {
        raw.quiet = Some(true);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::apply_flags_with_args;
    use crate::args::Cli;
    use crate::config::raw::Raw;

    fn cli(args: &[&str]) -> Cli {
        Cli::parse_from(std::iter::once("duckduckgo").chain(args.iter().copied()))
    }

    #[test]
    fn safe_and_unsafe_are_mutually_exclusive() {
        let cli = cli(&["--safe", "--unsafe", "rust"]);
        let err =
            apply_flags_with_args(&cli, &mut Raw::default(), Vec::<String>::new()).unwrap_err();
        assert!(err.to_string().contains("mutually exclusive"));
    }

    #[test]
    fn flags_override_existing_raw_values() {
        let cli = cli(&["--num", "7", "--region", "uk-en", "--unsafe", "rust"]);
        let mut raw = Raw {
            num: Some("3".to_owned()),
            region: Some("us-en".to_owned()),
            safe: Some(true),
            ..Raw::default()
        };
        apply_flags_with_args(&cli, &mut raw, Vec::<String>::new()).unwrap();
        assert_eq!(raw.num.as_deref(), Some("7"));
        assert_eq!(raw.region.as_deref(), Some("uk-en"));
        assert_eq!(raw.safe, Some(false));
    }

    #[test]
    fn retry_precedence_uses_original_arg_order() {
        let cli = cli(&["--retry", "5", "--no-retry", "rust"]);
        let mut raw = Raw::default();
        apply_flags_with_args(
            &cli,
            &mut raw,
            ["--retry", "5", "--no-retry"]
                .into_iter()
                .map(str::to_owned),
        )
        .unwrap();
        assert_eq!(raw.retry.as_deref(), Some("0"));
    }
}
