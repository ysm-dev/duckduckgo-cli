use serde_json::Value;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ColorWhen {
    Auto,
    Always,
    Never,
}

#[derive(Default)]
pub(crate) struct Raw {
    pub region: Option<String>,
    pub num: Option<String>,
    pub page: Option<String>,
    pub safe: Option<bool>,
    pub time: Option<String>,
    pub color: Option<String>,
    pub timeout: Option<String>,
    pub proxy: Option<String>,
    pub user_agent: Option<String>,
    pub retry: Option<String>,
    pub no_update_check: Option<bool>,
    pub no_rate_limit: Option<bool>,
    pub verbose: Option<u8>,
    pub quiet: Option<bool>,
}

pub(crate) fn str_field(obj: &serde_json::Map<String, Value>, key: &str) -> Option<String> {
    obj.get(key).and_then(Value::as_str).map(str::to_owned)
}

pub(crate) fn num_field(obj: &serde_json::Map<String, Value>, key: &str) -> Option<String> {
    obj.get(key).and_then(Value::as_u64).map(|v| v.to_string())
}

pub(crate) fn bool_field(obj: &serde_json::Map<String, Value>, key: &str) -> Option<bool> {
    obj.get(key).and_then(Value::as_bool)
}

pub(crate) fn set_env_string(slot: &mut Option<String>, key: &str) {
    if let Ok(value) = std::env::var(key) {
        *slot = Some(value);
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{bool_field, num_field, str_field};

    #[test]
    fn typed_fields_extract_expected_values() {
        let value = json!({ "region": "us-en", "num": 5, "safe": false, "nullish": null });
        let obj = value.as_object().unwrap();
        assert_eq!(str_field(obj, "region"), Some("us-en".to_owned()));
        assert_eq!(num_field(obj, "num"), Some("5".to_owned()));
        assert_eq!(bool_field(obj, "safe"), Some(false));
        assert_eq!(str_field(obj, "missing"), None);
        assert_eq!(num_field(obj, "nullish"), None);
    }
}
