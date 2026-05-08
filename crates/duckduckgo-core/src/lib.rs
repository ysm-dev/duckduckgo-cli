#![forbid(unsafe_code)]

//! Core DuckDuckGo search client and parser.

mod clock;
mod error;
mod parser;
pub mod paths;
mod rate_limit;
pub mod region;
mod search;

pub use clock::{Clock, ManualClock, SharedClock, SystemClock};
pub use error::{Error, Result};
pub use rate_limit::{Limits, ProgressHook, RateLimitProgress, RateLimitWait};
pub use region::Region;
pub use search::{
    Client, ClientBuilder, RateLimitJson, SearchBuilder, SearchMeta, SearchResponse, SearchResult,
    TimeFilter,
};
