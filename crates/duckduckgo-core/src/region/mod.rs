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
