use flate2::read::GzDecoder;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::io::Read;
use tracing::{info, warn};

use crate::crawler::{apply_basic_auth, client_with_proxy, cookie_header_value};
use crate::error::AppError;
use crate::models::{ProxyConfig, SiteAuth};

#[derive(Debug, Clone)]
pub struct SitemapUrl {
    pub loc: String,
    pub lastmod: Option<String>,
    pub changefreq: Option<String>,
    pub priority: Option<f64>,
}

#[derive(Debug, Clone, Default)]
pub struct SitemapResult {
    pub urls: Vec<SitemapUrl>,
    pub sitemaps_checked: Vec<String>,
}

pub struct SitemapParser {
    client: reqwest::Client,
    cookies: Vec<String>,
    site_auth: Option<SiteAuth>,
}

impl SitemapParser {
    pub fn new(
        user_agent: &str,
        cookies: Vec<String>,
        site_auth: Option<SiteAuth>,
        proxy: Option<&ProxyConfig>,
    ) -> Result<Self, AppError> {
        let client = client_with_proxy(proxy)?
            .user_agent(user_agent)
            .timeout(std::time::Duration::from_secs(15))
            .redirect(reqwest::redirect::Policy::limited(5))
            .build()?;
        Ok(Self {
            client,
            cookies,
            site_auth,
        })
    }

    pub async fn discover(&self, origin: &str) -> SitemapResult {
        let mut result = SitemapResult::default();

        // 1. Try robots.txt first
        let robots_url = format!("{}/robots.txt", origin);
        if let Ok(content) = self.fetch_text(&robots_url).await {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.to_lowercase().starts_with("sitemap:") {
                    let url = trimmed["sitemap:".len()..].trim().to_string();
                    if !url.is_empty() {
                        info!("Found sitemap in robots.txt: {}", url);
                        self.parse_sitemap_recursive(&url, &mut result, 0).await;
                    }
                }
            }
        }

        // 2. If no sitemaps found in robots.txt, try /sitemap.xml
        if result.urls.is_empty() {
            let default_url = format!("{}/sitemap.xml", origin);
            info!("Trying default sitemap: {}", default_url);
            self.parse_sitemap_recursive(&default_url, &mut result, 0)
                .await;
        }

        info!(
            "Sitemap discovery complete: {} URLs from {} sitemaps",
            result.urls.len(),
            result.sitemaps_checked.len()
        );
        result
    }

    async fn parse_sitemap_recursive(&self, url: &str, result: &mut SitemapResult, depth: u32) {
        if depth > 5 {
            warn!("Sitemap recursion depth exceeded for: {}", url);
            return;
        }

        if result.sitemaps_checked.contains(&url.to_string()) {
            return;
        }

        let bytes = match self.fetch_bytes(url).await {
            Ok(b) => b,
            Err(e) => {
                warn!("Failed to fetch sitemap {}: {}", url, e);
                return;
            }
        };

        result.sitemaps_checked.push(url.to_string());

        let content = match String::from_utf8(bytes) {
            Ok(s) => s,
            Err(e) => {
                warn!("Sitemap not valid UTF-8 {}: {}", url, e);
                return;
            }
        };

        if Self::is_sitemap_index(&content) {
            let child_sitemaps = Self::parse_sitemap_index(&content);
            info!(
                "Sitemap index {} contains {} child sitemaps",
                url,
                child_sitemaps.len()
            );
            for child in child_sitemaps {
                Box::pin(self.parse_sitemap_recursive(&child, result, depth + 1)).await;
            }
        } else {
            let urls = Self::parse_sitemap_urls(&content);
            info!("Sitemap {} contains {} URLs", url, urls.len());
            result.urls.extend(urls);
        }
    }

    fn is_sitemap_index(content: &str) -> bool {
        content.contains("<sitemapindex") || content.contains("<sitemap>")
    }

    fn parse_sitemap_index(content: &str) -> Vec<String> {
        let mut reader = Reader::from_str(content);
        let mut urls = Vec::new();
        let mut in_loc = false;
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    if e.name().as_ref() == b"loc" {
                        in_loc = true;
                    }
                }
                Ok(Event::Text(ref e)) => {
                    if in_loc {
                        if let Ok(text) = std::str::from_utf8(&e.clone().into_inner()) {
                            urls.push(text.trim().to_string());
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    if e.name().as_ref() == b"loc" {
                        in_loc = false;
                    }
                }
                Ok(Event::Eof) => break,
                Err(_) => break,
                _ => {}
            }
            buf.clear();
        }
        urls
    }

    fn parse_sitemap_urls(content: &str) -> Vec<SitemapUrl> {
        let mut reader = Reader::from_str(content);
        let mut urls = Vec::new();
        let mut current = SitemapUrl {
            loc: String::new(),
            lastmod: None,
            changefreq: None,
            priority: None,
        };
        let mut in_tag: Option<String> = None;
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    match name.as_str() {
                        "url" | "sitemap" => {
                            current = SitemapUrl {
                                loc: String::new(),
                                lastmod: None,
                                changefreq: None,
                                priority: None,
                            };
                        }
                        "loc" | "lastmod" | "changefreq" | "priority" => {
                            in_tag = Some(name);
                        }
                        _ => {}
                    }
                }
                Ok(Event::Text(ref e)) => {
                    if let Some(ref tag) = in_tag {
                        if let Ok(text) = std::str::from_utf8(&e.clone().into_inner()) {
                            let val = text.trim().to_string();
                            match tag.as_str() {
                                "loc" => current.loc = val,
                                "lastmod" => current.lastmod = Some(val),
                                "changefreq" => current.changefreq = Some(val),
                                "priority" => {
                                    current.priority = val.parse().ok();
                                }
                                _ => {}
                            }
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    match name.as_str() {
                        "url" | "sitemap" => {
                            if !current.loc.is_empty() {
                                urls.push(current.clone());
                            }
                        }
                        "loc" | "lastmod" | "changefreq" | "priority" => {
                            in_tag = None;
                        }
                        _ => {}
                    }
                }
                Ok(Event::Eof) => break,
                Err(_) => break,
                _ => {}
            }
            buf.clear();
        }
        urls
    }

    async fn fetch_text(&self, url: &str) -> Result<String, AppError> {
        let bytes = self.fetch_bytes(url).await?;
        Ok(String::from_utf8_lossy(&bytes).to_string())
    }

    async fn fetch_bytes(&self, url: &str) -> Result<Vec<u8>, AppError> {
        let mut request = self.client.get(url);
        if let Some(cookie) = cookie_header_value(&self.cookies) {
            request = request.header(reqwest::header::COOKIE, cookie);
        }
        request = apply_basic_auth(request, &self.site_auth);
        let response = request.send().await?.error_for_status()?;

        let is_gzip = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .map(|ct| ct.contains("gzip") || ct.contains("x-gzip"))
            .unwrap_or(false);

        let bytes = response.bytes().await?.to_vec();

        if is_gzip {
            let mut decoder = GzDecoder::new(&bytes[..]);
            let mut decoded = Vec::new();
            decoder.read_to_end(&mut decoded)?;
            Ok(decoded)
        } else {
            Ok(bytes)
        }
    }

    pub fn urls_as_strings(sitemap_urls: &[SitemapUrl]) -> Vec<String> {
        sitemap_urls.iter().map(|u| u.loc.clone()).collect()
    }
}
