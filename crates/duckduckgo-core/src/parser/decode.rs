use url::Url;

#[must_use]
pub fn normalized_text(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[must_use]
pub fn result_url(raw: &str) -> String {
    let absolute = if raw.starts_with("//") {
        format!("https:{raw}")
    } else {
        raw.to_owned()
    };
    let Ok(url) = Url::parse(&absolute) else {
        return raw.to_owned();
    };
    if url
        .host_str()
        .is_some_and(|h| h.ends_with("duckduckgo.com"))
        && url.path() == "/l/"
        && let Some((_, value)) = url.query_pairs().find(|(key, _)| key == "uddg")
    {
        return value.into_owned();
    }
    url.to_string()
}

#[cfg(test)]
mod tests {
    use super::{normalized_text, result_url};

    #[test]
    fn normalized_text_collapses_whitespace() {
        assert_eq!(
            normalized_text(" rust\n\t async   search "),
            "rust async search"
        );
    }

    #[test]
    fn result_url_unwraps_duckduckgo_redirect() {
        let raw = "https://duckduckgo.com/l/?uddg=https%3A%2F%2Fwww.rust-lang.org%2Flearn";
        assert_eq!(result_url(raw), "https://www.rust-lang.org/learn");
    }

    #[test]
    fn result_url_keeps_protocol_relative_absolute() {
        assert_eq!(result_url("//example.com/path"), "https://example.com/path");
    }
}
