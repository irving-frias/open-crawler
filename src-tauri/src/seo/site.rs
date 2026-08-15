use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use url::Url;

/// One fetched site-level resource (robots.txt / sitemap.xml).
#[derive(Debug, Clone)]
pub struct SiteResource {
    pub status: u16,
    pub body: String,
}

/// Site-level resources that drive the `robots_txt_exists` and
/// `sitemap_xml_valid` checks. Fetched once per origin and cached for the
/// lifetime of the process, so per-page audits share the same result.
#[derive(Debug, Clone, Default)]
pub struct SiteResources {
    pub robots_txt: Option<SiteResource>,
    pub sitemap_xml: Option<SiteResource>,
}

static CACHE: OnceLock<Mutex<HashMap<String, Arc<SiteResources>>>> = OnceLock::new();

/// Origin (scheme://host[:port]) of a page URL, or `None` for unparsable URLs.
pub fn origin_of(page_url: &str) -> Option<String> {
    let parsed = Url::parse(page_url).ok()?;
    let host = parsed.host_str()?;
    let scheme = parsed.scheme();
    let origin = match parsed.port() {
        Some(port) => format!("{scheme}://{host}:{port}"),
        None => format!("{scheme}://{host}"),
    };
    Some(origin)
}

/// Fetches robots.txt and sitemap.xml for the origin of `page_url`, caching the
/// result per origin so the whole crawl (and any re-audit) pays the cost once.
pub async fn fetch_site_resources(client: &reqwest::Client, page_url: &str) -> Arc<SiteResources> {
    let Some(origin) = origin_of(page_url) else {
        return Arc::new(SiteResources::default());
    };
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Some(hit) = cache.lock().unwrap().get(&origin) {
        return hit.clone();
    }

    let robots_url = format!("{origin}/robots.txt");
    let sitemap_url = format!("{origin}/sitemap.xml");
    let (robots, sitemap) = tokio::join!(
        fetch_resource(client, &robots_url),
        fetch_resource(client, &sitemap_url),
    );
    let resources = Arc::new(SiteResources {
        robots_txt: robots,
        sitemap_xml: sitemap,
    });
    cache.lock().unwrap().insert(origin, resources.clone());
    resources
}

async fn fetch_resource(client: &reqwest::Client, url: &str) -> Option<SiteResource> {
    let response = client.get(url).send().await.ok()?;
    let status = response.status().as_u16();
    let body = response.text().await.unwrap_or_default();
    Some(SiteResource { status, body })
}

/// A sitemap is considered valid when it parses as well-formed XML (every
/// element closed, to EOF, without errors) and its root element is a sitemap
/// index or a urlset.
pub fn is_valid_sitemap(body: &str) -> bool {
    let mut reader = quick_xml::Reader::from_str(body);
    reader.config_mut().check_end_names = true;
    let mut buf = Vec::new();
    let mut root: Option<Vec<u8>> = None;
    let mut depth = 0usize;
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(quick_xml::events::Event::Start(e)) => {
                if root.is_none() {
                    root = Some(e.name().as_ref().to_vec());
                }
                depth += 1;
            }
            Ok(quick_xml::events::Event::End(_)) => {
                depth = depth.saturating_sub(1);
            }
            Ok(quick_xml::events::Event::Eof) => {
                return depth == 0
                    && (root.as_deref() == Some(b"sitemapindex")
                        || root.as_deref() == Some(b"urlset"));
            }
            Ok(_) => {}
            Err(_) => return false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_origin_of() {
        assert_eq!(
            origin_of("https://example.com/page?a=1").as_deref(),
            Some("https://example.com")
        );
        assert_eq!(
            origin_of("http://localhost:8080/x").as_deref(),
            Some("http://localhost:8080")
        );
        assert_eq!(origin_of("not a url"), None);
    }

    #[test]
    fn test_is_valid_sitemap() {
        assert!(is_valid_sitemap(
            r#"<?xml version="1.0" encoding="UTF-8"?><urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9"><url><loc>https://example.com/</loc></url></urlset>"#
        ));
        assert!(is_valid_sitemap(
            r#"<sitemapindex><sitemap><loc>https://example.com/sitemap1.xml</loc></sitemap></sitemapindex>"#
        ));
        assert!(!is_valid_sitemap("<html><body>oops</body></html>"));
        assert!(!is_valid_sitemap(""));
        assert!(!is_valid_sitemap("<urlset><url><loc>"));
    }
}
