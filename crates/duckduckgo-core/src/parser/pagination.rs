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

#[cfg(test)]
mod tests {
    use scraper::Html;

    use super::next_fields;

    #[test]
    fn extracts_hidden_fields_from_next_form() {
        let document = Html::parse_document(
            r#"<form><input name="s" value="30"><input name="nextParams" value="x"><input name="vqd" value="abc"></form>"#,
        );
        let fields = next_fields(&document).unwrap();
        assert_eq!(fields[0], ("s".to_owned(), "30".to_owned()));
        assert!(fields.contains(&("nextParams".to_owned(), "x".to_owned())));
        assert!(fields.contains(&("vqd".to_owned(), "abc".to_owned())));
    }

    #[test]
    fn ignores_forms_without_next_marker() {
        let document = Html::parse_document(r#"<form><input name="q" value="rust"></form>"#);
        assert!(next_fields(&document).is_none());
    }
}
