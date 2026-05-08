use is_terminal::IsTerminal;
use unicode_width::UnicodeWidthStr;

use duckduckgo_core::SearchResponse;

use super::color::paint;

pub(crate) fn render(response: &SearchResponse, color: bool) -> String {
    let mut out = String::new();
    if let Some(answer) = &response.instant_answer {
        out.push('\n');
        out.push_str("    ");
        out.push_str(&paint(answer, "2;3", color));
        out.push_str("\n\n");
    }
    append_results(response, color, &mut out);
    out
}

fn append_results(response: &SearchResponse, color: bool, out: &mut String) {
    let index_width = response
        .results
        .iter()
        .map(|r| r.position.to_string().len())
        .max()
        .unwrap_or(1)
        .max(4);
    for result in &response.results {
        let index = format!("{:>index_width$}.", result.position);
        out.push_str(&format!(
            "{} {}\n",
            paint(&index, "1;36", color),
            paint(&result.title, "1;37", color)
        ));
        out.push_str(&format!("    {}\n", paint(&result.url, "32", color)));
        for line in wrap(&result.snippet, wrap_width().saturating_sub(4)) {
            out.push_str("    ");
            out.push_str(&line);
            out.push('\n');
        }
        out.push('\n');
    }
}

fn wrap_width() -> usize {
    if std::io::stdout().is_terminal() {
        terminal_size::terminal_size()
            .map(|(width, _)| usize::from(width.0))
            .unwrap_or(80)
    } else {
        80
    }
}

fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    let mut current_width = 0;
    for word in text.split_whitespace() {
        let word_width = UnicodeWidthStr::width(word);
        let space = usize::from(!current.is_empty());
        if current_width + space + word_width > width && !current.is_empty() {
            lines.push(current);
            current = String::new();
            current_width = 0;
        }
        if !current.is_empty() {
            current.push(' ');
            current_width += 1;
        }
        current.push_str(word);
        current_width += word_width;
    }
    if current.is_empty() {
        vec![String::new()]
    } else {
        lines.push(current);
        lines
    }
}

#[cfg(test)]
mod tests {
    use duckduckgo_core::{RateLimitJson, SearchMeta, SearchResponse, SearchResult};

    use super::{render, wrap};

    #[test]
    fn wrap_handles_ascii_words() {
        assert_eq!(wrap("one two three", 7), vec!["one two", "three"]);
    }

    #[test]
    fn wrap_counts_cjk_width_and_keeps_oversized_word() {
        assert_eq!(wrap("東京 rust", 5), vec!["東京", "rust"]);
        assert_eq!(
            wrap("supercalifragilistic", 4),
            vec!["supercalifragilistic"]
        );
    }

    #[test]
    fn wrap_empty_or_whitespace_returns_single_empty_line() {
        assert_eq!(wrap("", 80), vec![""]);
        assert_eq!(wrap(" \n\t ", 80), vec![""]);
    }

    #[test]
    fn render_plain_text_contract_without_color() {
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
                snippet: "A language empowering everyone.".to_owned(),
            }],
            meta: SearchMeta {
                rate_limit: RateLimitJson {
                    next_allowed_at: None,
                    blocked_until: None,
                    slowdown_until: None,
                    retried_count: 0,
                },
                fetched_pages: 1,
                elapsed_ms: 1,
            },
        };
        let out = render(&response, false);
        assert!(out.contains("answer"));
        assert!(out.contains("   1. Rust"));
        assert!(out.contains("https://www.rust-lang.org/"));
        assert!(!out.contains("\u{1b}["));
    }
}
