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

    // Strip Subresource Integrity / CORS attrs from <link>/<script> tags. The
    // preview may load the original external asset (inlining can be partial),
    // and a stale `integrity` hash blocks it with an SRI failure.
    result = strip_sri_attrs(&result);

    result
}

/// Remove `integrity` and `crossorigin` attributes from `<link ...>` and
/// `<script ...>` opening tags so stale SRI hashes can't block the preview.
fn strip_sri_attrs(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut rest = html;
    loop {
        match rest.find('<') {
            None => {
                out.push_str(rest);
                break;
            }
            Some(pos) => {
                out.push_str(&rest[..pos]);
                let after = &rest[pos + 1..];
                let is_target = (after.len() >= 4 && after[..4].eq_ignore_ascii_case("link"))
                    || (after.len() >= 6 && after[..6].eq_ignore_ascii_case("script"));
                if !is_target {
                    out.push('<');
                    rest = after;
                    continue;
                }
                let mut in_quote = false;
                let mut quote = b'\0';
                let mut end = after.len();
                for (idx, c) in after.char_indices() {
                    if !in_quote && c == '>' {
                        end = idx;
                        break;
                    }
                    if !in_quote && (c == '"' || c == '\'') {
                        in_quote = true;
                        quote = c as u8;
                    } else if in_quote && (c as u8) == quote {
                        in_quote = false;
                    }
                }
                let tag = format!("<{}", &after[..=end]);
                out.push_str(&strip_sri_tag(&tag));
                rest = &after[end + 1..];
            }
        }
    }
    out
}

/// Rebuild a single tag, dropping `integrity`/`crossorigin` attributes.
fn strip_sri_tag(tag: &str) -> String {
    let mut out = String::with_capacity(tag.len());
    let bytes = tag.as_bytes();
    let n = bytes.len();

    let mut i = 0;
    while i < n && !bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    out.push_str(&tag[..i]);

    while i < n {
        while i < n && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= n {
            break;
        }
        if bytes[i] == b'>' {
            out.push('>');
            break;
        }
        let mut k = i;
        let mut in_quote = false;
        let mut quote = b'\0';
        while k < n {
            let c = bytes[k];
            if in_quote {
                if c == quote {
                    in_quote = false;
                }
            } else if c == b'"' || c == b'\'' {
                in_quote = true;
                quote = c;
            } else if c == b' ' || c == b'\t' || c == b'\n' || c == b'\r' || c == b'>' {
                break;
            }
            k += 1;
        }
        let attr = &tag[i..k];
        let name = attr
            .split('=')
            .next()
            .unwrap_or("")
            .trim()
            .to_ascii_lowercase();
        if name != "integrity" && name != "crossorigin" {
            out.push(' ');
            out.push_str(attr);
        }
        i = k;
    }
    out
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_sri_removes_integrity_and_crossorigin() {
        let html = r#"<html><head>
<link rel="stylesheet" href="/css/a.css" integrity="sha256-AAAA" crossorigin>
<link rel="stylesheet" href="/css/b.css" crossorigin="anonymous" integrity="sha256-BBBB">
<script src="/app.js" integrity="sha256-CCCC"></script>
<link rel="icon" href="/favicon.ico">
</head></html>"#;
        let out = strip_sri_attrs(html);
        assert!(!out.contains("integrity"), "integrity not stripped: {out}");
        assert!(!out.contains("crossorigin"), "crossorigin not stripped: {out}");
        assert!(out.contains("href=\"/css/a.css\""), "href lost: {out}");
        assert!(out.contains("href=\"/css/b.css\""), "href lost: {out}");
        assert!(out.contains("src=\"/app.js\""), "script src lost: {out}");
        assert!(out.contains("href=\"/favicon.ico\""), "other link lost: {out}");
    }

    #[test]
    fn strip_sri_preserves_other_attrs_and_text() {
        let html = r#"<p>integrity="keep this text"</p><link data-x="a>b" rel="stylesheet" href="/x.css" integrity="sha256-X">"#;
        let out = strip_sri_attrs(html);
        assert!(!out.contains("integrity=\"sha256-X\""), "integrity attr not stripped: {out}");
        assert!(out.contains("data-x=\"a>b\""), "attr with > mangled: {out}");
        assert!(out.contains("integrity=\"keep this text\""), "text content altered: {out}");
    }
}
