use crate::Client;
use crate::parser::ParsedPage;
use crate::search::SearchResult;

use super::state::FetchState;

fn result(title: &str) -> SearchResult {
    SearchResult {
        position: 0,
        title: title.to_owned(),
        url: format!("https://example.com/{title}"),
        snippet: String::new(),
    }
}

#[test]
fn slice_results_renumbers_logical_page_positions() {
    let mut state = FetchState::new(4);
    let builder = Client::builder().num(2).build().unwrap().search("rust");
    state
        .accept_page(
            ParsedPage {
                instant_answer: None,
                results: vec![result("one"), result("two"), result("three")],
                next_fields: None,
                no_results: false,
            },
            &builder,
        )
        .unwrap();
    let page_two = state.slice_results(2, 2);
    assert_eq!(page_two.len(), 1);
    assert_eq!(page_two[0].position, 3);
    assert_eq!(page_two[0].title, "three");
}

#[test]
fn missing_next_fields_for_later_page_is_parse_error() {
    let builder = Client::builder()
        .num(2)
        .build()
        .unwrap()
        .search("rust")
        .page(2);
    let mut state = FetchState::new(4);
    let err = state
        .accept_page(
            ParsedPage {
                instant_answer: None,
                results: vec![result("one")],
                next_fields: None,
                no_results: false,
            },
            &builder,
        )
        .unwrap_err();
    assert!(err.to_string().contains("missing next-page fields"));
}

#[test]
fn first_instant_answer_wins_across_pages() {
    let builder = Client::builder().build().unwrap().search("rust");
    let mut state = FetchState::new(2);
    for answer in ["first", "second"] {
        state
            .accept_page(
                ParsedPage {
                    instant_answer: Some(answer.to_owned()),
                    results: vec![result(answer)],
                    next_fields: Some(vec![("s".to_owned(), "30".to_owned())]),
                    no_results: false,
                },
                &builder,
            )
            .unwrap();
    }
    assert_eq!(state.instant_answer.as_deref(), Some("first"));
}
