use std::collections::HashMap;
use std::time::{Duration, Instant};

use reqwest::Client;
use tracing::{info, warn};
use url::Url;

use crate::crawler::{apply_basic_auth, client_with_proxy, cookie_header_value};
use crate::error::AppError;
use crate::models::{ProxyConfig, SiteAuth};

const ROBOTS_CACHE_TTL_SECS: u64 = 3600;
const DEFAULT_CRAWL_DELAY_MS: u64 = 1000;

#[derive(Debug, Clone)]
struct RobotsData {
    disallow_paths: Vec<String>,
    crawl_delay_ms: u64,
    sitemaps: Vec<String>,
    fetched_at: Instant,
}

pub struct RobotsChecker {
    cache: HashMap<String, RobotsData>,
    user_agent: String,
    client: Client,
    cookies: Vec<String>,
    site_auth: Option<SiteAuth>,
}

impl RobotsChecker {
    pub fn new(
        user_agent: &str,
        cookies: Vec<String>,
        site_auth: Option<SiteAuth>,
        proxy: Option<&ProxyConfig>,
    ) -> Result<Self, AppError> {
        let client = client_with_proxy(proxy)?
            .user_agent(user_agent)
            .timeout(Duration::from_secs(10))
            .build()?;

        Ok(Self {
            cache: HashMap::new(),
            user_agent: user_agent.to_string(),
            client,
            cookies,
            site_auth,
        })
    }

    pub async fn can_fetch(&mut self, url: &Url) -> bool {
        let domain = match url.host_str() {
            Some(h) => h.to_string(),
            None => return true,
        };

        let robots = self.get_robots(&domain).await;
        let path = url.path();

        for disallowed in &robots.disallow_paths {
            if path.starts_with(disallowed) {
                info!("Blocked by robots.txt: {} (disallow: {})", url, disallowed);
                return false;
            }
        }

        true
    }

    pub async fn get_crawl_delay(&mut self, domain: &str) -> Duration {
        let robots = self.get_robots(domain).await;
        Duration::from_millis(robots.crawl_delay_ms)
    }

    pub async fn get_sitemaps(&mut self, domain: &str) -> Vec<String> {
        let robots = self.get_robots(domain).await;
        robots.sitemaps.clone()
    }

    async fn get_robots(&mut self, domain: &str) -> RobotsData {
        if let Some(data) = self.cache.get(domain) {
            if data.fetched_at.elapsed().as_secs() < ROBOTS_CACHE_TTL_SECS {
                return data.clone();
            }
        }

        let data = self.fetch_robots(domain).await;
        self.cache.insert(domain.to_string(), data.clone());
        data
    }

    async fn fetch_robots(&self, domain: &str) -> RobotsData {
        let url = format!("https://{}/robots.txt", domain);

        let mut request = self.client.get(&url);
        if let Some(cookie) = cookie_header_value(&self.cookies) {
            request = request.header(reqwest::header::COOKIE, cookie);
        }
        request = apply_basic_auth(request, &self.site_auth);

        let response = match request.send().await {
            Ok(r) => r,
            Err(e) => {
                warn!("Failed to fetch robots.txt for {}: {}", domain, e);
                return RobotsData {
                    disallow_paths: Vec::new(),
                    crawl_delay_ms: DEFAULT_CRAWL_DELAY_MS,
                    sitemaps: Vec::new(),
                    fetched_at: Instant::now(),
                };
            }
        };

        if !response.status().is_success() {
            info!(
                "No robots.txt for {} (status: {})",
                domain,
                response.status()
            );
            return RobotsData {
                disallow_paths: Vec::new(),
                crawl_delay_ms: DEFAULT_CRAWL_DELAY_MS,
                sitemaps: Vec::new(),
                fetched_at: Instant::now(),
            };
        }

        let body = match response.text().await {
            Ok(b) => b,
            Err(e) => {
                warn!("Failed to read robots.txt body for {}: {}", domain, e);
                return RobotsData {
                    disallow_paths: Vec::new(),
                    crawl_delay_ms: DEFAULT_CRAWL_DELAY_MS,
                    sitemaps: Vec::new(),
                    fetched_at: Instant::now(),
                };
            }
        };

        self.parse_robots(&body, domain)
    }

    fn parse_robots(&self, body: &str, domain: &str) -> RobotsData {
        let mut sitemaps = Vec::new();

        let mut in_wildcard_section = false;
        let mut in_specific_section = false;
        let mut wildcard_disallow = Vec::new();
        let mut wildcard_delay = DEFAULT_CRAWL_DELAY_MS;
        let mut specific_disallow = Vec::new();
        let mut specific_delay = DEFAULT_CRAWL_DELAY_MS;

        for line in body.lines() {
            let line = line.trim();

            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once(':') {
                let key = key.trim().to_lowercase();
                let value = value.trim();

                match key.as_str() {
                    "user-agent" => {
                        let ua_lower = self.user_agent.to_lowercase();
                        let agent_lower = value.to_lowercase();
                        let matches = agent_lower == "*" || ua_lower.contains(&agent_lower);

                        if matches {
                            if agent_lower == "*" {
                                in_wildcard_section = true;
                                in_specific_section = false;
                            } else {
                                in_specific_section = true;
                                in_wildcard_section = false;
                            }
                        } else {
                            in_wildcard_section = false;
                            in_specific_section = false;
                        }
                    }
                    "disallow" if in_wildcard_section && !value.is_empty() => {
                        wildcard_disallow.push(value.to_string());
                    }
                    "disallow" if in_specific_section && !value.is_empty() => {
                        specific_disallow.push(value.to_string());
                    }
                    "crawl-delay" if in_wildcard_section => {
                        if let Ok(delay) = value.parse::<u64>() {
                            wildcard_delay = delay * 1000;
                        }
                    }
                    "crawl-delay" if in_specific_section => {
                        if let Ok(delay) = value.parse::<u64>() {
                            specific_delay = delay * 1000;
                        }
                    }
                    "sitemap" => {
                        sitemaps.push(value.to_string());
                    }
                    _ => {}
                }
            }
        }

        // Specific section overrides wildcard
        let (disallow_paths, crawl_delay_ms) = if in_specific_section || !specific_disallow.is_empty() {
            (specific_disallow, specific_delay)
        } else {
            (wildcard_disallow, wildcard_delay)
        };

        info!(
            "Parsed robots.txt for {}: {} disallow rules, {}ms delay, {} sitemaps",
            domain,
            disallow_paths.len(),
            crawl_delay_ms,
            sitemaps.len()
        );

        RobotsData {
            disallow_paths,
            crawl_delay_ms,
            sitemaps,
            fetched_at: Instant::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_robots_basic() {
        let checker = RobotsChecker::new("TestBot/1.0", vec![], None, None).unwrap();
        let body = r#"
User-agent: *
Disallow: /admin/
Disallow: /private/
Crawl-delay: 5

User-agent: BadBot
Disallow: /

Sitemap: https://example.com/sitemap.xml
Sitemap: https://example.com/sitemap2.xml
"#;

        let data = checker.parse_robots(body, "example.com");
        assert_eq!(data.disallow_paths.len(), 2);
        assert!(data.disallow_paths.contains(&"/admin/".to_string()));
        assert!(data.disallow_paths.contains(&"/private/".to_string()));
        assert_eq!(data.crawl_delay_ms, 5000);
        assert_eq!(data.sitemaps.len(), 2);
    }

    #[test]
    fn test_parse_robots_specific_agent() {
        let checker = RobotsChecker::new("MyBot/1.0", vec![], None, None).unwrap();
        let body = r#"
User-agent: *
Disallow: /admin/

User-agent: MyBot
Disallow: /secret/
Crawl-delay: 10
"#;

        let data = checker.parse_robots(body, "example.com");
        // Should match MyBot specifically
        assert_eq!(data.disallow_paths.len(), 1);
        assert!(data.disallow_paths.contains(&"/secret/".to_string()));
        assert_eq!(data.crawl_delay_ms, 10000);
    }

    #[test]
    fn test_parse_robots_empty() {
        let checker = RobotsChecker::new("TestBot/1.0", vec![], None, None).unwrap();
        let data = checker.parse_robots("", "example.com");
        assert!(data.disallow_paths.is_empty());
        assert_eq!(data.crawl_delay_ms, DEFAULT_CRAWL_DELAY_MS);
        assert!(data.sitemaps.is_empty());
    }
}
