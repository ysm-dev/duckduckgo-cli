use clap::{ArgAction, CommandFactory, Parser};

use duckduckgo_core::{Error, Result};

#[derive(Debug, Parser)]
#[command(
    name = "duckduckgo",
    disable_help_flag = true,
    disable_version_flag = true
)]
pub struct Cli {
    #[arg(short = 'h', action = ArgAction::SetTrue)]
    pub help_short: bool,
    #[arg(long = "help", action = ArgAction::SetTrue)]
    pub help_long: bool,
    #[arg(short = 'V', long = "version", action = ArgAction::SetTrue)]
    pub version: bool,
    #[arg(long = "completion", value_name = "SHELL")]
    pub completion: Option<String>,
    #[arg(long = "list-regions", action = ArgAction::SetTrue)]
    pub list_regions: bool,
    #[arg(long = "print-config", action = ArgAction::SetTrue)]
    pub print_config: bool,
    #[arg(long = "check-updates", action = ArgAction::SetTrue)]
    pub check_updates: bool,
    #[arg(short = 'n', long = "num")]
    pub num: Option<String>,
    #[arg(short = 'r', long = "region")]
    pub region: Option<String>,
    #[arg(long = "page")]
    pub page: Option<String>,
    #[arg(short = 't', long = "time")]
    pub time: Option<String>,
    #[arg(short = 'w', long = "site")]
    pub sites: Vec<String>,
    #[arg(long = "safe", action = ArgAction::SetTrue)]
    pub safe: bool,
    #[arg(long = "unsafe", action = ArgAction::SetTrue)]
    pub unsafe_search: bool,
    #[arg(long = "json", action = ArgAction::SetTrue)]
    pub json: bool,
    #[arg(long = "color")]
    pub color: Option<String>,
    #[arg(short = 'v', long = "verbose", action = ArgAction::Count)]
    pub verbose: u8,
    #[arg(long = "quiet", action = ArgAction::SetTrue)]
    pub quiet: bool,
    #[arg(short = 'p', long = "proxy")]
    pub proxy: Option<String>,
    #[arg(long = "timeout")]
    pub timeout: Option<String>,
    #[arg(long = "user-agent")]
    pub user_agent: Option<String>,
    #[arg(long = "retry")]
    pub retry: Option<String>,
    #[arg(long = "no-retry", action = ArgAction::SetTrue)]
    pub no_retry: bool,
    #[arg(long = "no-wait", action = ArgAction::SetTrue)]
    pub no_wait: bool,
    #[arg(long = "config")]
    pub config: Option<String>,
    #[arg(long = "no-update-check", action = ArgAction::SetTrue)]
    pub no_update_check: bool,
    #[arg(long = "no-rate-limit", action = ArgAction::SetTrue)]
    pub no_rate_limit: bool,
    #[arg(long = "np", hide = true, action = ArgAction::SetTrue)]
    pub np: bool,
    #[arg(long = "noprompt", hide = true, action = ArgAction::SetTrue)]
    pub noprompt: bool,
    #[arg(long = "noua", hide = true, action = ArgAction::SetTrue)]
    pub noua: bool,
    #[arg(value_name = "QUERY")]
    pub query: Vec<String>,
}

pub fn retry_from_args(_cli: &Cli) -> Option<String> {
    let mut value = None;
    let mut retry_seen = false;
    for arg in std::env::args().skip(1) {
        if arg == "--no-retry" {
            value = Some("0".to_owned());
        } else if arg == "--retry" || arg.starts_with("--retry=") {
            if let Some((_, raw)) = arg.split_once('=') {
                value = Some(raw.to_owned());
            } else {
                retry_seen = true;
            }
        } else if retry_seen {
            value = Some(arg);
            retry_seen = false;
        }
    }
    value
}

pub fn print_completion(shell: &str) -> Result<()> {
    let shell = match shell {
        "bash" => clap_complete::Shell::Bash,
        "zsh" => clap_complete::Shell::Zsh,
        "fish" => clap_complete::Shell::Fish,
        "powershell" => clap_complete::Shell::PowerShell,
        "elvish" => clap_complete::Shell::Elvish,
        "nushell" => {
            let mut command = Cli::command();
            clap_complete::generate(
                clap_complete_nushell::Nushell,
                &mut command,
                "duckduckgo",
                &mut std::io::stdout(),
            );
            return Ok(());
        }
        other => return Err(Error::Usage(format!("Invalid --completion '{other}'"))),
    };
    let mut command = Cli::command();
    clap_complete::generate(shell, &mut command, "duckduckgo", &mut std::io::stdout());
    Ok(())
}

pub fn short_help() -> &'static str {
    "duckduckgo [OPTIONS] [QUERY...]\n\nSearch DuckDuckGo once and exit.\n\nOptions:\n  -n, --num <N>          Results to return (1..100)\n  -r, --region <CODE>   Region code (default us-en)\n      --page <N>        Logical page (>=1)\n      --json            Emit compact JSON\n  -h                    Short usage\n      --help            Full help\n  -V, --version         Print version"
}

pub fn long_help() -> &'static str {
    "duckduckgo [OPTIONS] [QUERY...]\n\nSearch DuckDuckGo once and exit. Plain text is the default and recommended for LLM agents.\n\nSearch Options:\n  -n, --num <N>          Results to return (1..100)\n  -r, --region <CODE>   Region code\n      --page <N>        Logical page (>=1)\n  -t, --time <SPAN>     d, w, m, or y\n  -w, --site <DOMAIN>   Repeatable site filter\n      --safe            Enable safe search\n      --unsafe          Disable safe search\n\nOutput Options:\n      --json            Emit compact JSON envelope\n      --color <WHEN>    auto, always, never\n  -v, --verbose         Increase stderr verbosity\n      --quiet           Suppress non-error stderr\n\nNetworking Options:\n  -p, --proxy <URL>     http, https, socks5, or socks5h proxy\n      --timeout <SEC>   Total timeout\n      --user-agent <S>  Opt-in User-Agent header\n      --retry <N>       Retries (0..10)\n      --no-retry        Alias for --retry 0\n      --no-wait         Refuse rate-limit waits\n\nMeta Options:\n      --completion <SHELL>\n      --list-regions\n      --print-config\n      --check-updates\n\nEXAMPLES:\n  duckduckgo rust async tutorial\n  duckduckgo -n 5 --json \"best Rust web framework 2026\"\n  echo \"tokio runtime overhead\" | duckduckgo"
}
