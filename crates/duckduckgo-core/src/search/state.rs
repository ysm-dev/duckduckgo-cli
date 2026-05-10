use super::client::SearchBuilder;
use super::form::next_form;
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
            snapshot: Snapshot::default(),
            done: false,
        }
    }

    pub(crate) fn should_fetch(&self) -> bool {
        self.results.len() < self.needed && !self.done
    }

    pub(crate) fn accept_page(&mut self, page: ParsedPage, builder: &SearchBuilder) -> Result<()> {
        let ParsedPage {
            instant_answer,
            results,
            next_fields,
            no_results,
        } = page;
        self.fetched_pages += 1;
        if self.instant_answer.is_none() {
            self.instant_answer = instant_answer;
        }
        self.results.extend(results);
        if no_results {
            self.done = true;
            return Ok(());
        }
        self.next = next_fields
            .as_deref()
            .and_then(|fields| next_form(builder, fields, self.fetched_pages, self.results.len()));
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
