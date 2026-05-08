use std::path::PathBuf;

use crate::rate_limit::ProgressHook;
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
            progress_hook: None,
        }
    }
}
