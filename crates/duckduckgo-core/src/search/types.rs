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

#[cfg(test)]
mod tests {
    use super::{RateLimitJson, SearchMeta, SearchResponse, SearchResult, TimeFilter};

    #[test]
    fn time_filter_tokens_match_ddg_form_values() {
        assert_eq!(TimeFilter::Day.as_ddg(), "d");
        assert_eq!(TimeFilter::Week.as_ddg(), "w");
        assert_eq!(TimeFilter::Month.as_ddg(), "m");
        assert_eq!(TimeFilter::Year.as_ddg(), "y");
    }

    #[test]
    fn search_response_serializes_contract_shape() {
        let response = SearchResponse {
            schema: 1,
            query: "rust".to_owned(),
            kind: "web",
            region: "us-en".to_owned(),
            page: 1,
            count: 1,
            instant_answer: Some("answer".to_owned()),
            results: vec![SearchResult {
                position: 1,
                title: "Rust".to_owned(),
                url: "https://www.rust-lang.org/".to_owned(),
                snippet: "Systems language".to_owned(),
            }],
            meta: SearchMeta {
                rate_limit: RateLimitJson {
                    next_allowed_at: Some("2026-05-09T00:00:00Z".to_owned()),
                    blocked_until: None,
                    slowdown_until: None,
                    retried_count: 2,
                },
                fetched_pages: 1,
                elapsed_ms: 7,
            },
        };
        let json = serde_json::to_value(response).unwrap();
        assert_eq!(json["type"], "web");
        assert_eq!(json["results"][0]["position"], 1);
        assert_eq!(json["meta"]["rate_limit"]["retried_count"], 2);
    }
}
