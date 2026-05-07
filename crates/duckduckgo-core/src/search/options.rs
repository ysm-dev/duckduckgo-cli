use std::path::PathBuf;

use crate::region::Region;

#[derive(Clone, Debug)]
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
        }
    }
}
