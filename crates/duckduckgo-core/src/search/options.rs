use std::path::PathBuf;
use std::sync::Arc;

use crate::clock::{SharedClock, SystemClock};
use crate::rate_limit::{Limits, ProgressHook};
use crate::region::Region;

#[derive(Clone)]
pub(crate) struct ClientOptions {
    pub endpoint: String,
    pub region: Region,
    pub num: usize,
    pub safe: bool,
    pub timeout: u64,
    pub proxy: Option<String>,
    pub user_agent: Option<String>,
    pub retry: u8,
    pub no_wait: bool,
    pub no_rate_limit: bool,
    pub state_dir: PathBuf,
    pub limits: Limits,
    pub clock: SharedClock,
    pub progress_hook: Option<ProgressHook>,
}

impl std::fmt::Debug for ClientOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClientOptions")
            .field("endpoint", &self.endpoint)
            .field("region", &self.region)
            .field("num", &self.num)
            .field("safe", &self.safe)
            .field("timeout", &self.timeout)
            .field("proxy", &self.proxy)
            .field("user_agent", &self.user_agent)
            .field("retry", &self.retry)
            .field("no_wait", &self.no_wait)
            .field("no_rate_limit", &self.no_rate_limit)
            .field("state_dir", &self.state_dir)
            .field("limits", &self.limits)
            .field("clock", &"<clock>")
            .field(
                "progress_hook",
                &self.progress_hook.as_ref().map(|_| "<hook>"),
            )
            .finish()
    }
}

impl Default for ClientOptions {
    fn default() -> Self {
        Self {
            // Trailing slash matters: posting to `/html` returns
            // `HTTP 202` (anomaly) while `/html/` returns `HTTP 200`.
            // See `docs/en/ddgr.md` for the empirical evidence.
            endpoint: "https://html.duckduckgo.com/html/".to_owned(),
            region: Region::default(),
            num: 10,
            safe: true,
            timeout: 30,
            proxy: None,
            user_agent: None,
            retry: 3,
            no_wait: false,
            no_rate_limit: false,
            state_dir: PathBuf::from("."),
            limits: Limits::from_env(),
            clock: Arc::new(SystemClock),
            progress_hook: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ClientOptions;
    use std::path::PathBuf;

    #[test]
    fn defaults_match_public_cli_contract() {
        let options = ClientOptions::default();
        assert_eq!(options.endpoint, "https://html.duckduckgo.com/html/");
        assert!(options.endpoint.ends_with('/'));
        assert_eq!(options.region.code(), "us-en");
        assert_eq!(options.num, 10);
        assert!(options.safe);
        assert_eq!(options.timeout, 30);
        assert_eq!(options.retry, 3);
        assert_eq!(options.state_dir, PathBuf::from("."));
    }

    #[test]
    fn debug_redacts_non_debuggable_hooks_and_clock() {
        let options = ClientOptions::default();
        let debug = format!("{options:?}");
        assert!(debug.contains("endpoint"));
        assert!(debug.contains("<clock>"));
        assert!(debug.contains("progress_hook"));
    }
}
