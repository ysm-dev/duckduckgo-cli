use url::Url;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BlockReason {
    Http202,
    Http403,
    Http429,
    AnomalyMarker,
    ChallengeRedirect,
}

impl BlockReason {
    #[must_use]
    pub fn as_state_value(self) -> &'static str {
        match self {
            Self::Http202 => "http_202",
            Self::Http403 => "http_403",
            Self::Http429 => "http_429",
            Self::AnomalyMarker => "anomaly_marker",
            Self::ChallengeRedirect => "challenge_redirect",
        }
    }
}

pub fn classify_block(
    status: u16,
    body: &str,
    final_url: &Url,
    endpoint_url: &Url,
) -> Option<BlockReason> {
    match status {
        202 => return Some(BlockReason::Http202),
        403 => return Some(BlockReason::Http403),
        429 => return Some(BlockReason::Http429),
        _ => {}
    }
    // Only match markers unique to DDG's actual anomaly modal HTML; the
    // bare words `challenge` and `captcha` appear in legitimate search
    // results (think "rust async tutorial", which returns many hits for
    // "Async/Await Challenge" articles) and produced false-positive
    // 202-classifications on a real `HTTP 200` body. The strings below
    // are taken verbatim from the captured anomaly response saved in
    // `docs/en/ddgr.md` §1 and are not present in normal DDG result
    // pages.
    let lowered = body.to_ascii_lowercase();
    if lowered.contains("anomaly-modal__")
        || lowered.contains("anomaly_modal")
        || lowered.contains("/anomaly.js")
        || lowered.contains("unfortunately, bots use duckduckgo")
        || lowered.contains("id=\"challenge-form\"")
    {
        return Some(BlockReason::AnomalyMarker);
    }
    if (300..400).contains(&status) && final_url.host_str() != endpoint_url.host_str() {
        return Some(BlockReason::ChallengeRedirect);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{BlockReason, classify_block};
    use url::Url;

    fn endpoint() -> Url {
        Url::parse("https://html.duckduckgo.com/html/").unwrap()
    }

    #[test]
    fn http_202_is_a_block_regardless_of_body() {
        assert_eq!(
            classify_block(202, "", &endpoint(), &endpoint()),
            Some(BlockReason::Http202)
        );
    }

    #[test]
    fn http_200_with_anomaly_modal_class_is_a_block() {
        let body = r#"<div class="anomaly-modal__box">…</div>"#;
        assert_eq!(
            classify_block(200, body, &endpoint(), &endpoint()),
            Some(BlockReason::AnomalyMarker)
        );
    }

    #[test]
    fn http_200_with_legitimate_challenge_word_is_not_a_block() {
        // Real DDG result snippet for `rust async tutorial`:
        let body =
            r#"<a class="result__snippet">An async/await challenge for the curious developer.</a>"#;
        assert!(classify_block(200, body, &endpoint(), &endpoint()).is_none());
    }

    #[test]
    fn http_200_with_captcha_word_inside_a_result_is_not_a_block() {
        let body = r#"<a class="result__snippet">Implementing a custom captcha in Rust.</a>"#;
        assert!(classify_block(200, body, &endpoint(), &endpoint()).is_none());
    }

    #[test]
    fn http_200_with_challenge_form_id_is_a_block() {
        let body = r#"<form id="challenge-form" action="//duckduckgo.com/anomaly.js"…>"#;
        assert_eq!(
            classify_block(200, body, &endpoint(), &endpoint()),
            Some(BlockReason::AnomalyMarker)
        );
    }

    #[test]
    fn redirect_to_other_host_is_a_challenge_redirect() {
        let elsewhere = Url::parse("https://example.com/blocked").unwrap();
        assert_eq!(
            classify_block(302, "", &elsewhere, &endpoint()),
            Some(BlockReason::ChallengeRedirect)
        );
    }
}
