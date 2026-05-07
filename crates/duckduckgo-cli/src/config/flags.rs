use duckduckgo_core::{Error, Result};

use crate::args::{Cli, retry_from_args};

use super::raw::Raw;

pub(crate) fn apply_flags(cli: &Cli, raw: &mut Raw) -> Result<()> {
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
    if let Some(v) = retry_from_args(cli) {
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
