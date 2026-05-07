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
