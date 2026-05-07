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
    let lowered = body.to_ascii_lowercase();
    if lowered.contains("anomaly_modal")
        || lowered.contains("captcha")
        || lowered.contains("are you a robot")
        || lowered.contains("challenge")
    {
        return Some(BlockReason::AnomalyMarker);
    }
    if (300..400).contains(&status) && final_url.host_str() != endpoint_url.host_str() {
        return Some(BlockReason::ChallengeRedirect);
    }
    None
}
