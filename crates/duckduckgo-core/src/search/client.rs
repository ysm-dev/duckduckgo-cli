use std::path::PathBuf;
use std::sync::Arc;

use super::execute;
use super::options::ClientOptions;
use super::types::{SearchResponse, TimeFilter};
use crate::Clock;
use crate::Result;
use crate::rate_limit::Limits;
use crate::rate_limit::ProgressHook;
use crate::region::Region;

#[derive(Clone, Debug)]
pub struct Client {
    options: ClientOptions,
}

#[derive(Clone, Debug)]
pub struct ClientBuilder {
    options: ClientOptions,
}

#[derive(Clone, Debug)]
pub struct SearchBuilder {
    pub(crate) options: ClientOptions,
    pub(crate) query: String,
    pub(crate) page: usize,
    pub(crate) time: Option<TimeFilter>,
    pub(crate) sites: Vec<String>,
}

impl Client {
    #[must_use]
    pub fn builder() -> ClientBuilder {
        ClientBuilder {
            options: ClientOptions::default(),
        }
    }

    #[must_use]
    pub fn search(&self, query: impl Into<String>) -> SearchBuilder {
        SearchBuilder {
            options: self.options.clone(),
            query: query.into(),
            page: 1,
            time: None,
            sites: Vec::new(),
        }
    }
}

impl ClientBuilder {
    pub fn region(mut self, region: Region) -> Self {
        self.options.region = region;
        self
    }
    pub fn num(mut self, num: usize) -> Self {
        self.options.num = num;
        self
    }
    pub fn safe(mut self, safe: bool) -> Self {
        self.options.safe = safe;
        self
    }
    pub fn timeout(mut self, seconds: u64) -> Self {
        self.options.timeout = seconds;
        self
    }
    pub fn proxy(mut self, proxy: Option<String>) -> Self {
        self.options.proxy = proxy;
        self
    }
    pub fn user_agent(mut self, value: Option<String>) -> Self {
        self.options.user_agent = value;
        self
    }
    pub fn retry(mut self, retry: u8) -> Self {
        self.options.retry = retry;
        self
    }
    pub fn no_wait(mut self, value: bool) -> Self {
        self.options.no_wait = value;
        self
    }
    pub fn no_rate_limit(mut self, value: bool) -> Self {
        self.options.no_rate_limit = value;
        self
    }
    pub fn state_dir(mut self, value: PathBuf) -> Self {
        self.options.state_dir = value;
        self
    }
    pub fn endpoint(mut self, value: String) -> Self {
        self.options.endpoint = value;
        self
    }
    pub fn limits(mut self, value: Limits) -> Self {
        self.options.limits = value;
        self
    }
    pub fn clock(mut self, value: Arc<dyn Clock>) -> Self {
        self.options.clock = value;
        self
    }
    /// Install a callback that fires before every cooldown / spacing
    /// sleep cycle. Used by the CLI to emit `[INFO] rate-limit ...`
    /// progress lines on stderr; library callers typically leave this
    /// unset.
    #[must_use]
    pub fn on_rate_limit_progress(mut self, hook: ProgressHook) -> Self {
        self.options.progress_hook = Some(hook);
        self
    }
    pub fn build(self) -> Result<Client> {
        Ok(Client {
            options: self.options,
        })
    }
}

impl SearchBuilder {
    #[must_use]
    pub fn page(mut self, page: usize) -> Self {
        self.page = page;
        self
    }
    #[must_use]
    pub fn time(mut self, time: Option<TimeFilter>) -> Self {
        self.time = time;
        self
    }
    #[must_use]
    pub fn site(mut self, site: String) -> Self {
        self.sites.push(site);
        self
    }
    pub async fn send(self) -> Result<SearchResponse> {
        execute::execute(self).await
    }
}
