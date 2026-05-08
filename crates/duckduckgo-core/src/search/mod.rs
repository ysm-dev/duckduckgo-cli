mod client;
mod execute;
mod fetch;
mod form;
mod http;
mod options;
mod state;
mod types;

#[cfg(test)]
#[path = "client_tests.rs"]
mod client_tests;
#[cfg(test)]
#[path = "fetch_tests.rs"]
mod fetch_tests;
#[cfg(test)]
#[path = "state_tests.rs"]
mod state_tests;

pub use client::{Client, ClientBuilder, SearchBuilder};
pub use types::{RateLimitJson, SearchMeta, SearchResponse, SearchResult, TimeFilter};
