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
    normalize_input(&bytes)
}

fn normalize_input(bytes: &[u8]) -> Result<String> {
    let mut text = String::from_utf8(bytes.to_vec())
        .map_err(|_| Error::Usage("Invalid UTF-8 on stdin".to_owned()))?;
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

#[cfg(test)]
mod tests {
    use super::normalize_input;

    #[test]
    fn normalize_input_strips_bom_and_coalesces_whitespace() {
        assert_eq!(
            normalize_input("\u{feff} rust\n\tasync   search".as_bytes()).unwrap(),
            "rust async search"
        );
    }

    #[test]
    fn normalize_input_handles_crlf() {
        assert_eq!(normalize_input(b"rust\r\nbook").unwrap(), "rust book");
    }

    #[test]
    fn normalize_input_rejects_empty_and_invalid_utf8() {
        assert!(normalize_input(b" \n\t ").is_err());
        assert!(normalize_input(&[0xff]).is_err());
    }
}
