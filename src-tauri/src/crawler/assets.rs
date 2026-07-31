use reqwest::Client;
use scraper::{Html, Selector};
use std::collections::HashMap;
use url::Url;

use crate::error::AppError;

const MAX_RESOURCE_BYTES: usize = 512 * 1024;
const INLINE_TIMEOUT_MS: u64 = 10_000;

/// Fetch external CSS/images and inline them as data URIs into the HTML.
pub async fn inline_page_assets(html: &str, base_url: &Url) -> Result<String, AppError> {
    // Phase 1: Parse HTML and collect all external URLs (synchronous, not Send)
    let asset_urls = collect_asset_urls(html, base_url);

    // Phase 2: Fetch all assets concurrently (async, Send)
    let data_uris = fetch_assets(&asset_urls).await;

    // Phase 3: Replace URLs in HTML (synchronous)
    let result = replace_urls(html, base_url, &asset_urls, &data_uris);

    Ok(result)
}

fn collect_asset_urls(html: &str, base_url: &Url) -> Vec<(String, String)> {
    let document = Html::parse_document(html);
    let mut asset_urls: Vec<(String, String)> = Vec::new();

    // CSS links
    if let Ok(sel) = Selector::parse("link[rel='stylesheet']") {
        for el in document.select(&sel) {
            if let Some(href) = el.value().attr("href") {
                if let Some(abs) = resolve_url(base_url, href) {
                    asset_urls.push((abs, format!("link[href='{}']", href)));
                }
            }
        }
    }

    // Preload/prefetch CSS
    if let Ok(sel) = Selector::parse("link[rel='preload'][as='style']") {
        for el in document.select(&sel) {
            if let Some(href) = el.value().attr("href") {
                if let Some(abs) = resolve_url(base_url, href) {
                    asset_urls.push((abs, format!("link[href='{}']", href)));
                }
            }
        }
    }

    // Images
    if let Ok(sel) = Selector::parse("img[src]") {
        for el in document.select(&sel) {
            if let Some(src) = el.value().attr("src") {
                if !src.starts_with("data:") {
                    if let Some(abs) = resolve_url(base_url, src) {
                        asset_urls.push((abs, format!("img[src='{}']", src)));
                    }
                }
            }
        }
    }

    // Favicons
    if let Ok(sel) = Selector::parse("link[rel='icon'], link[rel='apple-touch-icon']") {
        for el in document.select(&sel) {
            if let Some(href) = el.value().attr("href") {
                if !href.starts_with("data:") {
                    if let Some(abs) = resolve_url(base_url, href) {
                        asset_urls.push((abs, format!("link[href='{}']", href)));
                    }
                }
            }
        }
    }

    // Deduplicate
    let mut seen = std::collections::HashSet::new();
    asset_urls.retain(|(url, _)| seen.insert(url.clone()));
    asset_urls
}

async fn fetch_assets(asset_urls: &[(String, String)]) -> HashMap<String, String> {
    let client = match Client::builder()
        .timeout(std::time::Duration::from_millis(INLINE_TIMEOUT_MS))
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()
    {
        Ok(c) => c,
        Err(_) => return HashMap::new(),
    };

    let mut data_uris: HashMap<String, String> = HashMap::new();

    for chunk in asset_urls.chunks(6) {
        let futures: Vec<_> = chunk
            .iter()
            .map(|(url, _)| fetch_asset_as_data_uri(&client, url.clone()))
            .collect();

        let results = futures::future::join_all(futures).await;

        for ((url, _), result) in chunk.iter().zip(results) {
            if let Ok(data_uri) = result {
                data_uris.insert(url.clone(), data_uri);
            }
        }
    }

    data_uris
}

fn replace_urls(
    html: &str,
    base_url: &Url,
    _asset_urls: &[(String, String)],
    data_uris: &HashMap<String, String>,
) -> String {
    let document = Html::parse_document(html);
    let mut result = html.to_string();

    // Replace CSS links with inline <style>
    if let Ok(sel) = Selector::parse("link[rel='stylesheet']") {
        for el in document.select(&sel) {
            if let Some(href) = el.value().attr("href") {
                if let Some(abs) = resolve_url(base_url, href) {
                    if let Some(data_uri) = data_uris.get(&abs) {
                        if let Some(css) = data_uri_to_string(data_uri) {
                            let tag = el.html();
                            let style_tag =
                                format!("<style data-inlined=\"{}\">{}</style>", href, css);
                            result = result.replace(&tag, &style_tag);
                        }
                    }
                }
            }
        }
    }

    // Replace image/favicon data URIs by URL substitution
    for (url, data_uri) in data_uris {
        if is_image_url(url) {
            result = result.replace(url, data_uri);
        }
    }

    // Replace favicon/apple-touch-icon link tags
    if let Ok(sel) = Selector::parse("link[rel='icon'], link[rel='apple-touch-icon']") {
        let doc2 = Html::parse_document(&result);
        for el in doc2.select(&sel) {
            if let Some(href) = el.value().attr("href") {
                if let Some(abs) = resolve_url(base_url, href) {
                    if let Some(data_uri) = data_uris.get(&abs) {
                        let tag = el.html();
                        let new_tag = tag.replace(href, data_uri);
                        result = result.replace(&tag, &new_tag);
                    }
                }
            }
        }
    }

    // Remove external <script> tags
    if let Ok(sel) = Selector::parse("script[src]") {
        let doc2 = Html::parse_document(&result);
        for el in doc2.select(&sel) {
            if let Some(src) = el.value().attr("src") {
                if !src.starts_with("data:") {
                    result = result.replace(&el.html(), "<!-- inlined: script removed -->");
                }
            }
        }
    }

    result
}

fn resolve_url(base: &Url, relative: &str) -> Option<String> {
    base.join(relative).ok().map(|u| u.to_string())
}

fn is_image_url(url: &str) -> bool {
    url.contains(".png")
        || url.contains(".jpg")
        || url.contains(".jpeg")
        || url.contains(".gif")
        || url.contains(".svg")
        || url.contains(".webp")
        || url.contains(".ico")
        || url.contains("/favicon")
        || url.contains("/images/")
        || url.contains("/img/")
        || url.contains("/assets/")
}

fn data_uri_to_string(data_uri: &str) -> Option<String> {
    if let Some(b64) = data_uri.strip_prefix("data:text/css;base64,") {
        use base64::Engine;
        base64::engine::general_purpose::STANDARD
            .decode(b64)
            .ok()
            .and_then(|bytes| String::from_utf8(bytes).ok())
    } else {
        data_uri
            .strip_prefix("data:text/css;,")
            .map(|raw| raw.to_string())
    }
}

async fn fetch_asset_as_data_uri(client: &Client, url: String) -> Result<String, AppError> {
    let resp = client.get(&url).send().await?;
    let content_type = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/octet-stream")
        .to_string();

    let bytes = resp.bytes().await?;
    if bytes.len() > MAX_RESOURCE_BYTES {
        return Err(AppError::Crawl(format!(
            "Asset too large: {} ({} bytes)",
            url,
            bytes.len()
        )));
    }

    let b64 = use_base64(&bytes);
    Ok(format!("data:{};base64,{}", content_type, b64))
}

fn use_base64(bytes: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(bytes)
}
