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
