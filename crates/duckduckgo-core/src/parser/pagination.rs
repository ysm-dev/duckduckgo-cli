use scraper::{Html, Selector};

pub fn next_fields(document: &Html) -> Option<Vec<(String, String)>> {
    let form_sel = Selector::parse("form").expect("valid selector");
    let input_sel = Selector::parse("input").expect("valid selector");
    for form in document.select(&form_sel) {
        let fields = form
            .select(&input_sel)
            .filter_map(|input| {
                let name = input.value().attr("name")?;
                let value = input.value().attr("value").unwrap_or_default();
                Some((name.to_owned(), value.to_owned()))
            })
            .collect::<Vec<_>>();
        let has_next_marker = fields
            .iter()
            .any(|(name, _)| matches!(name.as_str(), "s" | "nextParams" | "vqd"));
        if has_next_marker {
            return Some(fields);
        }
    }
    None
}
