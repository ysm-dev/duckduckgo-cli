mod codes;

pub use codes::REGION_CODES;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Region(String);

impl Region {
    pub fn parse(value: &str) -> Option<Self> {
        REGION_CODES
            .binary_search_by_key(&value, |(code, _)| *code)
            .ok()
            .map(|_| Self(value.to_owned()))
    }

    #[must_use]
    pub fn code(&self) -> &str {
        &self.0
    }
}

impl Default for Region {
    fn default() -> Self {
        Self("us-en".to_owned())
    }
}

impl std::fmt::Display for Region {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::Region;

    #[test]
    fn parse_accepts_known_region() {
        let region = Region::parse("us-en").unwrap();
        assert_eq!(region.code(), "us-en");
        assert_eq!(region.to_string(), "us-en");
    }

    #[test]
    fn parse_rejects_unknown_and_wrong_case() {
        assert!(Region::parse("xx-yy").is_none());
        assert!(Region::parse("US-EN").is_none());
    }

    #[test]
    fn default_region_is_us_en() {
        assert_eq!(Region::default().code(), "us-en");
    }
}
