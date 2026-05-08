use scraper::{Html, Selector};

use super::decode::{normalized_text, result_url};
use super::pagination;
use crate::search::SearchResult;
use crate::{Error, Result};

#[derive(Clone, Debug)]
pub struct ParsedPage {
    pub instant_answer: Option<String>,
    pub results: Vec<SearchResult>,
    pub next_fields: Option<Vec<(String, String)>>,
    pub no_results: bool,
}

pub fn parse_html(body: &str) -> Result<ParsedPage> {
    let document = Html::parse_document(body);
    let results = organic_results(&document);
    let no_results = is_no_results(body);
    if results.is_empty() && !no_results {
        return Err(Error::Parse("Parsing search response".to_owned()));
    }
    Ok(ParsedPage {
        instant_answer: instant_answer(&document),
        results,
        next_fields: pagination::next_fields(&document),
        no_results,
    })
}

fn organic_results(document: &Html) -> Vec<SearchResult> {
    let result_sel = Selector::parse("div.result, div.web-result").expect("valid selector");
    let title_sel = Selector::parse("a.result__a").expect("valid selector");
    let snippet_sel =
        Selector::parse("a.result__snippet, div.result__snippet").expect("valid selector");
    document
        .select(&result_sel)
        .filter_map(|node| {
            let title = node.select(&title_sel).next()?;
            let href = title.value().attr("href").unwrap_or_default();
            let snippet = node.select(&snippet_sel).next();
            Some(SearchResult {
                position: 0,
                title: normalized_text(&title.text().collect::<String>()),
                url: result_url(href),
                snippet: snippet
                    .map(|s| normalized_text(&s.text().collect::<String>()))
                    .unwrap_or_default(),
            })
        })
        .collect()
}

fn instant_answer(document: &Html) -> Option<String> {
    let selector = Selector::parse("div.zci__result, div.zci__main").expect("valid selector");
    document.select(&selector).find_map(|node| {
        let text = normalized_text(&node.text().collect::<String>());
        (!text.is_empty()).then_some(text)
    })
}

fn is_no_results(body: &str) -> bool {
    let lowered = body.to_ascii_lowercase();
    lowered.contains("no results") || lowered.contains("not find any results")
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::parse_html;

    const RESULTS: &str = include_str!("../../../../tests/fixtures/results-2026-05.html");
    const INSTANT: &str = include_str!("../../../../tests/fixtures/instant-answer-2026-05.html");
    const EMPTY: &str = include_str!("../../../../tests/fixtures/empty-results-2026-05.html");

    #[test]
    fn results_fixture_parses_expected_results() {
        let parsed = parse_html(RESULTS).unwrap();
        assert_eq!(
            parsed.instant_answer.as_deref(),
            Some("Rust is a systems programming language.")
        );
        assert_eq!(parsed.results.len(), 2);
        assert_eq!(parsed.results[0].title, "Rust Programming Language");
        assert_eq!(parsed.results[0].url, "https://www.rust-lang.org/");
        assert_eq!(parsed.results[1].title, "The Rust Book");
    }

    #[test]
    fn instant_answer_fixture_extracts_answer() {
        let parsed = parse_html(INSTANT).unwrap();
        assert_eq!(
            parsed.instant_answer.as_deref(),
            Some("Instant answer text.")
        );
        assert_eq!(parsed.results.len(), 1);
    }

    #[test]
    fn empty_results_fixture_is_not_parse_error() {
        let parsed = parse_html(EMPTY).unwrap();
        assert!(parsed.no_results);
        assert!(parsed.results.is_empty());
    }

    proptest! {
        #[test]
        fn arbitrary_html_never_panics(input in ".*") {
            let _ = parse_html(&input);
        }
    }
}
