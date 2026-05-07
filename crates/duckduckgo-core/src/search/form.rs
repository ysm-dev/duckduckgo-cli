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
