use serde::Serialize;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TimeFilter {
    Day,
    Week,
    Month,
    Year,
}

#[derive(Clone, Debug, Serialize)]
pub struct SearchResult {
    pub position: usize,
    pub title: String,
    pub url: String,
    pub snippet: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct SearchResponse {
    pub schema: u8,
    pub query: String,
    #[serde(rename = "type")]
    pub kind: &'static str,
    pub region: String,
    pub page: usize,
    pub count: usize,
    pub instant_answer: Option<String>,
    pub results: Vec<SearchResult>,
    pub meta: SearchMeta,
}

#[derive(Clone, Debug, Serialize)]
pub struct SearchMeta {
    pub rate_limit: RateLimitJson,
    pub fetched_pages: usize,
    pub elapsed_ms: u128,
}

#[derive(Clone, Debug, Serialize)]
pub struct RateLimitJson {
    pub next_allowed_at: Option<String>,
    pub blocked_until: Option<String>,
    pub slowdown_until: Option<String>,
    pub retried_count: u32,
}

impl TimeFilter {
    pub(crate) fn as_ddg(self) -> &'static str {
        match self {
            Self::Day => "d",
            Self::Week => "w",
            Self::Month => "m",
            Self::Year => "y",
        }
    }
}
