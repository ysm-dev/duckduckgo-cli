use duckduckgo_core::paths::RuntimePaths;
use duckduckgo_core::{Error, Result};
use serde_json::Value;

use super::raw::{Raw, bool_field, num_field, str_field};

pub(crate) fn load_file(
    paths: &RuntimePaths,
    raw: &mut Raw,
    warnings: &mut Vec<String>,
) -> Result<()> {
    if !paths.config_file.exists() {
        return Ok(());
    }
    let text = std::fs::read_to_string(&paths.config_file)?;
    let value: Value =
        jsonc_parser::parse_to_serde_value(&text, &Default::default()).map_err(|e| {
            Error::Usage(format!(
                "Config file is malformed. (Parsing {}: {e})",
                paths.config_file.display()
            ))
        })?;
    let Some(obj) = value.as_object() else {
        return Err(Error::Usage(
            "Config file root must be an object".to_owned(),
        ));
    };
    warn_unknown_keys(obj, warnings);
    validate_types(obj)?;
    raw.region = str_field(obj, "region");
    raw.num = num_field(obj, "num");
    raw.safe = bool_field(obj, "safe");
    raw.time = str_field(obj, "time");
    raw.timeout = num_field(obj, "timeout");
    raw.proxy = str_field(obj, "proxy");
    raw.user_agent = str_field(obj, "user_agent");
    raw.retry = num_field(obj, "retry");
    raw.color = str_field(obj, "color");
    raw.no_update_check = bool_field(obj, "no_update_check");
    Ok(())
}

fn validate_types(obj: &serde_json::Map<String, Value>) -> Result<()> {
    for key in ["region", "time", "proxy", "user_agent", "color"] {
        if obj.get(key).is_some_and(|v| !v.is_null() && !v.is_string()) {
            return Err(Error::Usage(format!(
                "Config key '{key}' must be a string or null"
            )));
        }
    }
    for key in ["num", "timeout", "retry"] {
        if obj
            .get(key)
            .is_some_and(|v| !v.is_null() && v.as_u64().is_none())
        {
            return Err(Error::Usage(format!(
                "Config key '{key}' must be an integer or null"
            )));
        }
    }
    for key in ["safe", "no_update_check"] {
        if obj
            .get(key)
            .is_some_and(|v| !v.is_null() && !v.is_boolean())
        {
            return Err(Error::Usage(format!(
                "Config key '{key}' must be a bool or null"
            )));
        }
    }
    Ok(())
}

fn warn_unknown_keys(obj: &serde_json::Map<String, Value>, warnings: &mut Vec<String>) {
    for key in obj.keys() {
        if !matches!(
            key.as_str(),
            "region"
                | "num"
                | "safe"
                | "time"
                | "timeout"
                | "proxy"
                | "user_agent"
                | "retry"
                | "color"
                | "no_update_check"
        ) {
            warnings.push(format!("Unknown config key '{key}' ignored."));
        }
    }
}
