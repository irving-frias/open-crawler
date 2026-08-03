use scraper::{Html, Selector};
use url::Url;

use crate::crawler::fetcher::{HtmlFetcher, HttpFetcher};
use crate::error::AppError;
use crate::models::crawl_config::IMPLICIT_USER_AGENT;

#[tauri::command]
pub fn is_mobile() -> bool {
    cfg!(mobile)
}

/// Best-effort favicon discovery: parses the page's `<link rel="icon">` tags,
/// falling back to `/favicon.ico` at the site origin. Returns `None` when the
/// URL is invalid or the icon cannot be resolved.
#[tauri::command]
pub async fn get_favicon(url: String) -> Result<Option<String>, AppError> {
    let page_url = match Url::parse(&url) {
        Ok(u) if matches!(u.scheme(), "http" | "https") => u,
        _ => return Ok(None),
    };

    let fetcher = HttpFetcher::new(IMPLICIT_USER_AGENT, 10_000, Vec::new(), None)?;

    if let Ok(response) = fetcher.fetch(&page_url).await {
        let document = Html::parse_document(&response.html);
        if let Ok(selector) = Selector::parse("link[rel~='icon']") {
            for el in document.select(&selector) {
                let Some(href) = el.value().attr("href").map(str::trim) else {
                    continue;
                };
                if href.is_empty() || href.starts_with("data:") || href.starts_with("javascript:") {
                    continue;
                }
                if let Ok(resolved) = response.url.join(href) {
                    if matches!(resolved.scheme(), "http" | "https") {
                        return Ok(Some(resolved.to_string()));
                    }
                }
            }
        }
        return Ok(origin_favicon(&response.url));
    }

    Ok(origin_favicon(&page_url))
}

fn origin_favicon(url: &Url) -> Option<String> {
    match url.scheme() {
        "http" | "https" => {
            let mut base = url.clone();
            base.set_path("/favicon.ico");
            base.set_query(None);
            base.set_fragment(None);
            Some(base.to_string())
        }
        _ => None,
    }
}
