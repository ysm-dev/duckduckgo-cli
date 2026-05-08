use super::client::SearchBuilder;

pub(crate) fn effective_query(builder: &SearchBuilder) -> String {
    if builder.sites.is_empty() {
        return builder.query.clone();
    }
    let site = if builder.sites.len() == 1 {
        format!("site:{}", builder.sites[0])
    } else {
        format!(
            "({})",
            builder
                .sites
                .iter()
                .map(|s| format!("site:{s}"))
                .collect::<Vec<_>>()
                .join(" OR ")
        )
    };
    format!("{site} {}", builder.query)
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

#[cfg(test)]
mod tests {
    use crate::Client;

    use super::{effective_query, first_form};

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
        assert_eq!(effective_query(&builder), "site:example.com rust");
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
            "(site:example.com OR site:rust-lang.org) rust"
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
}
