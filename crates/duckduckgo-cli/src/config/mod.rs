mod env_vars;
mod file;
mod flags;
mod print;
mod raw;
mod validate;

use std::env;
use std::path::PathBuf;

use duckduckgo_core::paths::{RuntimePaths, runtime_paths};
use duckduckgo_core::{Region, Result, TimeFilter};

use crate::args::Cli;

pub use print::print_json;
pub use raw::ColorWhen;

#[derive(Clone, Debug)]
pub struct Settings {
    pub region: Region,
    pub num: usize,
    pub page: usize,
    pub safe: bool,
    pub time: Option<TimeFilter>,
    pub sites: Vec<String>,
    pub json: bool,
    pub color: ColorWhen,
    pub verbose: u8,
    pub quiet: bool,
    pub timeout: u64,
    pub proxy: Option<String>,
    pub user_agent: Option<String>,
    pub retry: u8,
    pub no_wait: bool,
    pub no_update_check: bool,
    pub rate_limit: bool,
    pub paths: RuntimePaths,
    pub warnings: Vec<String>,
    pub ddg_url: String,
    pub github_url: String,
}

pub fn load(cli: &Cli) -> Result<Settings> {
    let config_path = cli
        .config
        .clone()
        .or_else(|| env::var("DUCKDUCKGO_CONFIG").ok())
        .map(PathBuf::from);
    let paths = runtime_paths(config_path)?;
    let mut raw = raw::Raw::default();
    let mut warnings = Vec::new();
    file::load_file(&paths, &mut raw, &mut warnings)?;
    env_vars::apply_standard_proxy(&mut raw);
    env_vars::apply_env(&mut raw)?;
    flags::apply_flags(cli, &mut raw)?;
    validate::validate(cli, raw, paths, warnings)
}
