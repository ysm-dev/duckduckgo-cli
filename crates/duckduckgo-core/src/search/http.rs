use std::time::Duration;

use url::Url;
use wreq::header::{HeaderMap, HeaderValue};

use super::options::ClientOptions;
use crate::{Error, Result};

pub(crate) fn build_client(options: &ClientOptions) -> Result<wreq::Client> {
    let total = Duration::from_secs(options.timeout);
    let mut default_headers = HeaderMap::new();
    default_headers.insert("Accept-Encoding", HeaderValue::from_static("gzip"));
    default_headers.insert("DNT", HeaderValue::from_static("1"));
    let mut builder = wreq::Client::builder();
    builder = builder
        .no_proxy()
        .default_headers(default_headers)
        .timeout(total)
        .connect_timeout(total.min(Duration::from_secs(5)))
        .read_timeout(total.min(Duration::from_secs(15)))
        // ddgr --noua sends an explicit empty User-Agent header.
        // Users can override via --user-agent.
        .user_agent(options.user_agent.as_deref().unwrap_or(""));
    if let Some(proxy) = &options.proxy {
        builder = builder.proxy(wreq::Proxy::all(proxy).map_err(|e| Error::Usage(e.to_string()))?);
    }
    builder.build().map_err(|e| Error::Network(e.to_string()))
}

pub(crate) async fn send_once(
    client: &wreq::Client,
    options: &ClientOptions,
    fields: Vec<(String, String)>,
) -> std::result::Result<(u16, Url, String), String> {
    let response = client
        .post(&options.endpoint)
        .form(&fields)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let status = response.status().as_u16();
    let mut final_url = response.url().clone();
    if (300..400).contains(&status)
        && let Some(location) = response.headers().get(wreq::header::LOCATION)
        && let Ok(location) = location.to_str()
        && let Ok(parsed) = Url::parse(location)
    {
        final_url = parsed;
    }
    let body = response.text().await.map_err(|e| e.to_string())?;
    Ok((status, final_url, body))
}

#[cfg(test)]
mod tests {
    use super::build_client;
    use crate::search::options::ClientOptions;

    #[test]
    fn build_client_rejects_invalid_proxy() {
        let options = ClientOptions {
            proxy: Some("not a proxy url".to_owned()),
            ..ClientOptions::default()
        };
        let err = build_client(&options).unwrap_err();
        assert!(err.to_string().contains("Usage error"));
    }

    #[test]
    fn build_client_accepts_timeout_and_user_agent() {
        let options = ClientOptions {
            timeout: 1,
            user_agent: Some("duckduckgo-test/1.0".to_owned()),
            ..ClientOptions::default()
        };
        let _client = build_client(&options).unwrap();
    }
}
