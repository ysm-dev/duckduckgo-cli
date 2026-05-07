use std::io::Read;

use is_terminal::IsTerminal;

use duckduckgo_core::{Error, Result};

const LIMIT: usize = 64 * 1024;

pub fn query(args: &[String]) -> Result<String> {
    if !args.is_empty() {
        let query = args.join(" ").trim().to_owned();
        return non_empty(query);
    }
    if std::io::stdin().is_terminal() {
        return Err(Error::Usage(
            "Missing query. Pass QUERY... or pipe stdin.".to_owned(),
        ));
    }
    let mut bytes = Vec::new();
    std::io::stdin()
        .take((LIMIT + 1) as u64)
        .read_to_end(&mut bytes)?;
    if bytes.len() > LIMIT {
        return Err(Error::Usage("Stdin query exceeds 64 KiB".to_owned()));
    }
    let mut text =
        String::from_utf8(bytes).map_err(|_| Error::Usage("Invalid UTF-8 on stdin".to_owned()))?;
    if text.starts_with('\u{feff}') {
        text = text.trim_start_matches('\u{feff}').to_owned();
    }
    non_empty(text.split_whitespace().collect::<Vec<_>>().join(" "))
}

fn non_empty(query: String) -> Result<String> {
    if query.is_empty() {
        Err(Error::Usage("Empty query".to_owned()))
    } else {
        Ok(query)
    }
}
