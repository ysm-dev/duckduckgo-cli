use std::time::Duration;

use url::Url;

use super::options::ClientOptions;
use crate::{Error, Result};

pub(crate) fn build_client(options: &ClientOptions) -> Result<wreq::Client> {
    let total = Duration::from_secs(options.timeout);
    let mut builder = wreq::Client::builder();
    if options.endpoint.starts_with("https://") {
        builder = builder.emulation(wreq::EmulationProvider::default());
    }
    builder = builder
        .no_proxy()
        .timeout(total)
        .connect_timeout(total.min(Duration::from_secs(5)))
        .read_timeout(total.min(Duration::from_secs(15)))
        .redirect(wreq::redirect::Policy::none())
        .cookie_store(true);
    if let Some(proxy) = &options.proxy {
        builder = builder.proxy(wreq::Proxy::all(proxy).map_err(|e| Error::Usage(e.to_string()))?);
    }
    if let Some(ua) = &options.user_agent {
        builder = builder.user_agent(ua.as_str());
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
