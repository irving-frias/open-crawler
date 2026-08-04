pub mod assets;
pub mod db_writer;
pub mod dedup;
pub mod engine;
pub mod fetcher;
pub mod frontier;
pub mod parser;
pub mod robots;
pub mod screenshot;
pub mod sitemap;

pub use engine::CrawlEngine;
pub use parser::SemanticIssue;

use crate::error::AppError;
use crate::models::ProxyConfig;

/// Joins raw cookie strings into a single `Cookie` header value, or `None` if
/// there is nothing to send.
pub(crate) fn cookie_header_value(cookies: &[String]) -> Option<String> {
    let joined = cookies
        .iter()
        .map(|c| c.trim())
        .filter(|c| !c.is_empty())
        .collect::<Vec<_>>()
        .join("; ");
    if joined.is_empty() {
        None
    } else {
        Some(joined)
    }
}

/// Applies HTTP Basic Auth to a request builder when credentials are set.
pub(crate) fn apply_basic_auth(
    request: reqwest::RequestBuilder,
    auth: &Option<crate::models::SiteAuth>,
) -> reqwest::RequestBuilder {
    match auth {
        Some(a) if !a.username.is_empty() => {
            request.basic_auth(&a.username, Some(&a.password))
        }
        _ => request,
    }
}

pub(crate) fn client_with_proxy(
    proxy: Option<&ProxyConfig>,
) -> Result<reqwest::ClientBuilder, AppError> {
    let builder = reqwest::Client::builder();

    #[cfg(not(target_os = "android"))]
    let builder = builder.use_preconfigured_tls(
        native_tls::TlsConnector::new()
            .map_err(|e| AppError::Io(std::io::Error::other(e)))?,
    );

    let Some(proxy) = proxy else {
        return Ok(builder);
    };

    let url = proxy.url.trim();
    if url.is_empty() {
        return Ok(builder);
    }

    let mut reqwest_proxy = reqwest::Proxy::all(url)?;
    if let (Some(user), Some(pass)) = (&proxy.username, &proxy.password) {
        if !user.is_empty() {
            reqwest_proxy = reqwest_proxy.basic_auth(user, pass);
        }
    }

    Ok(builder.proxy(reqwest_proxy))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_with_proxy_none() {
        assert!(client_with_proxy(None).is_ok());
    }

    #[test]
    fn test_client_with_proxy_valid() {
        let proxy = ProxyConfig {
            url: "http://127.0.0.1:8080".to_string(),
            username: Some("user".to_string()),
            password: Some("pass".to_string()),
        };
        assert!(client_with_proxy(Some(&proxy)).is_ok());
    }

    #[test]
    fn test_client_with_proxy_empty_url_ok() {
        let proxy = ProxyConfig::default();
        assert!(client_with_proxy(Some(&proxy)).is_ok());
    }

    #[test]
    fn test_client_with_proxy_invalid_url() {
        let proxy = ProxyConfig {
            url: "not a valid proxy url".to_string(),
            ..Default::default()
        };
        assert!(client_with_proxy(Some(&proxy)).is_err());
    }
}
