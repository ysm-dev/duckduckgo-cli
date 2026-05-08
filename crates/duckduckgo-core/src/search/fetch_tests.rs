use super::fetch::{CallResult, classify_response};
use crate::parser::BlockReason;
use crate::rate_limit::AttemptOutcome;
use url::Url;

fn endpoint() -> Url {
    Url::parse("https://html.duckduckgo.com/html/").unwrap()
}

#[test]
fn classify_response_parses_successful_html() {
    let body = include_str!("../../../../tests/fixtures/results-2026-05.html");
    let (outcome, result) = classify_response(Ok((200, endpoint(), body.to_owned())), &endpoint());
    assert!(matches!(outcome, AttemptOutcome::Success));
    let CallResult::Parsed(page) = result else {
        panic!("expected parsed page")
    };
    assert_eq!(page.results.len(), 2);
}

#[test]
fn classify_response_detects_blocks_before_parsing() {
    let (outcome, result) = classify_response(
        Ok((202, endpoint(), "anomaly_modal".to_owned())),
        &endpoint(),
    );
    assert!(matches!(
        outcome,
        AttemptOutcome::Block(BlockReason::Http202)
    ));
    assert!(matches!(result, CallResult::Block(BlockReason::Http202)));
}

#[test]
fn classify_response_splits_transient_and_permanent_http() {
    let (outcome, result) = classify_response(Ok((503, endpoint(), String::new())), &endpoint());
    assert!(matches!(outcome, AttemptOutcome::Other));
    assert!(matches!(result, CallResult::Transient5xx(503)));

    let (outcome, result) = classify_response(Ok((404, endpoint(), String::new())), &endpoint());
    assert!(matches!(outcome, AttemptOutcome::Other));
    assert!(matches!(result, CallResult::PermanentHttp(404)));
}

#[test]
fn classify_response_preserves_network_and_parse_errors() {
    let (outcome, result) = classify_response(Err("dns failed".to_owned()), &endpoint());
    assert!(matches!(outcome, AttemptOutcome::Other));
    assert!(matches!(result, CallResult::Network(message) if message == "dns failed"));

    let (outcome, result) = classify_response(
        Ok((200, endpoint(), "<html></html>".to_owned())),
        &endpoint(),
    );
    assert!(matches!(outcome, AttemptOutcome::Success));
    assert!(matches!(result, CallResult::Parse(_)));
}
