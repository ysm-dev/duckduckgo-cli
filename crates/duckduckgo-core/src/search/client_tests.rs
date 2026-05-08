use std::sync::Arc;

use crate::rate_limit::Limits;
use crate::{ManualClock, Region};

use super::client::Client;

#[test]
fn builder_chaining_updates_client_options() {
    let clock = Arc::new(ManualClock::new(time::OffsetDateTime::now_utc()));
    let client = Client::builder()
        .region(Region::parse("wt-wt").unwrap())
        .num(7)
        .safe(false)
        .timeout(5)
        .proxy(Some("http://proxy.example".to_owned()))
        .user_agent(Some("tool/1.0".to_owned()))
        .retry(2)
        .no_wait(true)
        .no_rate_limit(true)
        .state_dir("state".into())
        .endpoint("https://example.com/html/".to_owned())
        .limits(Limits::test_fast(1, 2, 1))
        .clock(clock)
        .build()
        .unwrap();
    let search = client.search("rust").page(3);
    assert_eq!(search.options.region.code(), "wt-wt");
    assert_eq!(search.options.num, 7);
    assert!(!search.options.safe);
    assert_eq!(search.options.endpoint, "https://example.com/html/");
    assert_eq!(search.page, 3);
}

#[test]
fn search_builder_collects_time_and_sites() {
    let client = Client::builder().build().unwrap();
    let search = client
        .search("rust")
        .time(Some(crate::TimeFilter::Day))
        .site("example.com".to_owned());
    assert_eq!(search.query, "rust");
    assert_eq!(search.time, Some(crate::TimeFilter::Day));
    assert_eq!(search.sites, vec!["example.com"]);
}
