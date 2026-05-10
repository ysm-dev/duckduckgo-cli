use super::client::SearchBuilder;

pub(crate) fn effective_query(builder: &SearchBuilder) -> String {
    if builder.sites.is_empty() {
        return builder.query.clone();
    }
    let mut query = builder.query.clone();
    query.push_str(" (");
    for site in &builder.sites {
        query.push_str(" site:");
        query.push_str(site);
        query.push_str(" OR");
    }
    query.push_str(" )");
    query
}

pub(crate) fn first_form(builder: &SearchBuilder) -> Vec<(String, String)> {
    vec![
        ("q".to_owned(), effective_query(builder)),
        ("b".to_owned(), String::new()),
        (
            "df".to_owned(),
            builder
                .time
                .map_or("", super::types::TimeFilter::as_ddg)
                .to_owned(),
        ),
        ("kf".to_owned(), "-1".to_owned()),
        ("kh".to_owned(), "1".to_owned()),
        ("kl".to_owned(), builder.options.region.code().to_owned()),
        (
            "kp".to_owned(),
            if builder.options.safe { "1" } else { "-2" }.to_owned(),
        ),
        ("k1".to_owned(), "-1".to_owned()),
    ]
}

pub(crate) fn next_form(
    builder: &SearchBuilder,
    source_fields: &[(String, String)],
    ddg_page: usize,
    result_count: usize,
) -> Option<Vec<(String, String)>> {
    let next_params = field_value(source_fields, "nextParams")?;
    let vqd = field_value(source_fields, "vqd").unwrap_or_default();
    Some(vec![
        ("q".to_owned(), effective_query(builder)),
        (
            "s".to_owned(),
            (50_usize
                .saturating_mul(ddg_page.saturating_sub(1))
                .saturating_add(30))
            .to_string(),
        ),
        ("nextParams".to_owned(), next_params),
        ("v".to_owned(), "l".to_owned()),
        ("o".to_owned(), "json".to_owned()),
        ("dc".to_owned(), result_count.saturating_add(1).to_string()),
        (
            "df".to_owned(),
            builder
                .time
                .map_or("", super::types::TimeFilter::as_ddg)
                .to_owned(),
        ),
        ("api".to_owned(), "/d.js".to_owned()),
        ("kf".to_owned(), "-1".to_owned()),
        ("kh".to_owned(), "1".to_owned()),
        ("kl".to_owned(), builder.options.region.code().to_owned()),
        (
            "kp".to_owned(),
            if builder.options.safe { "1" } else { "-2" }.to_owned(),
        ),
        ("k1".to_owned(), "-1".to_owned()),
        ("vqd".to_owned(), vqd),
    ])
}

fn field_value(fields: &[(String, String)], name: &str) -> Option<String> {
    fields
        .iter()
        .find_map(|(field, value)| (field == name).then(|| value.clone()))
}

#[cfg(test)]
mod tests {
    use crate::Client;

    use super::{effective_query, first_form, next_form};

    #[test]
    fn effective_query_preserves_plain_query_without_sites() {
        let client = Client::builder().build().unwrap();
        let builder = client.search("rust async");
        assert_eq!(effective_query(&builder), "rust async");
    }

    #[test]
    fn effective_query_prefixes_single_site() {
        let client = Client::builder().build().unwrap();
        let builder = client.search("rust").site("example.com".to_owned());
        assert_eq!(effective_query(&builder), "rust ( site:example.com OR )");
    }

    #[test]
    fn effective_query_groups_multiple_sites_with_or() {
        let client = Client::builder().build().unwrap();
        let builder = client
            .search("rust")
            .site("example.com".to_owned())
            .site("rust-lang.org".to_owned());
        assert_eq!(
            effective_query(&builder),
            "rust ( site:example.com OR site:rust-lang.org OR )"
        );
    }

    #[test]
    fn first_form_includes_region_safety_and_time() {
        let client = Client::builder().safe(false).build().unwrap();
        let builder = client.search("rust").time(Some(crate::TimeFilter::Week));
        let fields = first_form(&builder);
        assert!(fields.contains(&("q".to_owned(), "rust".to_owned())));
        assert!(fields.contains(&("df".to_owned(), "w".to_owned())));
        assert!(fields.contains(&("kl".to_owned(), "us-en".to_owned())));
        assert!(fields.contains(&("kp".to_owned(), "-2".to_owned())));
    }

    #[test]
    fn next_form_matches_ddgr_later_page_shape() {
        let client = Client::builder().safe(false).build().unwrap();
        let builder = client.search("rust").time(Some(crate::TimeFilter::Week));
        let fields = next_form(
            &builder,
            &[
                ("s".to_owned(), "999".to_owned()),
                ("nextParams".to_owned(), "x".to_owned()),
                ("vqd".to_owned(), "abc".to_owned()),
            ],
            2,
            80,
        )
        .unwrap();
        assert_eq!(fields[0], ("q".to_owned(), "rust".to_owned()));
        assert_eq!(fields[1], ("s".to_owned(), "80".to_owned()));
        assert_eq!(fields[2], ("nextParams".to_owned(), "x".to_owned()));
        assert_eq!(fields[3], ("v".to_owned(), "l".to_owned()));
        assert_eq!(fields[4], ("o".to_owned(), "json".to_owned()));
        assert_eq!(fields[5], ("dc".to_owned(), "81".to_owned()));
        assert!(fields.contains(&("df".to_owned(), "w".to_owned())));
        assert!(fields.contains(&("api".to_owned(), "/d.js".to_owned())));
        assert!(fields.contains(&("kp".to_owned(), "-2".to_owned())));
        assert_eq!(
            fields.last().unwrap(),
            &("vqd".to_owned(), "abc".to_owned())
        );
    }
}
