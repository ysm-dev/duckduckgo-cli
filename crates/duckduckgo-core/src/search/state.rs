use super::client::SearchBuilder;
use super::types::SearchResult;
use crate::parser::ParsedPage;
use crate::rate_limit::Snapshot;
use crate::{Error, Result};

pub(crate) struct FetchState {
    needed: usize,
    results: Vec<SearchResult>,
    pub(crate) instant_answer: Option<String>,
    pub(crate) next: Option<Vec<(String, String)>>,
    pub(crate) fetched_pages: usize,
    pub(crate) retried_count: u32,
    pub(crate) snapshot: Snapshot,
    done: bool,
}

impl FetchState {
    pub(crate) fn new(needed: usize) -> Self {
        Self {
            needed,
            results: Vec::new(),
            instant_answer: None,
            next: None,
            fetched_pages: 0,
            retried_count: 0,
            snapshot: Snapshot {
                tokens_remaining: None,
                blocked_until: None,
            },
            done: false,
        }
    }

    pub(crate) fn should_fetch(&self) -> bool {
        self.results.len() < self.needed && !self.done
    }

    pub(crate) fn accept_page(&mut self, page: ParsedPage, builder: &SearchBuilder) -> Result<()> {
        self.fetched_pages += 1;
        if self.instant_answer.is_none() {
            self.instant_answer = page.instant_answer;
        }
        self.results.extend(page.results);
        if page.no_results {
            self.done = true;
            return Ok(());
        }
        self.next = page.next_fields;
        if self.next.is_none() && builder.page > 1 && self.results.len() < self.needed {
            return Err(Error::Parse(
                "Parsing search response: missing next-page fields".to_owned(),
            ));
        }
        if self.next.is_none() {
            self.done = true;
        }
        Ok(())
    }

    pub(crate) fn slice_results(&self, page: usize, num: usize) -> Vec<SearchResult> {
        let start = (page - 1) * num;
        let mut results = self
            .results
            .iter()
            .skip(start)
            .take(num)
            .cloned()
            .collect::<Vec<_>>();
        for (idx, result) in results.iter_mut().enumerate() {
            result.position = start + idx + 1;
        }
        results
    }
}
