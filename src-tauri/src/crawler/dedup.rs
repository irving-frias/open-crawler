use ahash::AHashSet;
use url::Url;

pub struct Deduplicator {
    seen: AHashSet<String>,
}

impl Deduplicator {
    pub fn new() -> Self {
        Self {
            seen: AHashSet::default(),
        }
    }

    pub fn normalize(url: &str) -> String {
        let Ok(mut parsed) = Url::parse(url) else {
            return url.to_string();
        };

        // Remove fragment
        parsed.set_fragment(None);

        // Lowercase host
        if let Some(host) = parsed.host_str() {
            let lower_host = host.to_lowercase();
            let _ = parsed.set_host(Some(&lower_host));
        }

        // Remove default ports
        let default_port = match parsed.scheme() {
            "http" => Some(80u16),
            "https" => Some(443u16),
            _ => None,
        };
        if let Some(port) = default_port {
            if parsed.port() == Some(port) {
                let _ = parsed.set_port(None);
            }
        }

        // Normalize path: remove trailing slash (except root)
        let path = parsed.path().to_string();
        if path.len() > 1 && path.ends_with('/') {
            let normalized = path.trim_end_matches('/').to_string();
            parsed.set_path(&normalized);
        }

        // Sort query params
        let mut pairs: Vec<(String, String)> = parsed
            .query_pairs()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        if !pairs.is_empty() {
            pairs.sort_by(|a, b| a.0.cmp(&b.0));
            let sorted_query: String = pairs
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join("&");
            parsed.set_query(Some(&sorted_query));
        }

        parsed.to_string()
    }

    pub fn is_duplicate(&mut self, url: &str) -> bool {
        let normalized = Self::normalize(url);
        !self.seen.insert(normalized)
    }

    pub fn seen_count(&self) -> usize {
        self.seen.len()
    }

    pub fn clear(&mut self) {
        self.seen.clear();
    }
}

impl Default for Deduplicator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_strips_fragment() {
        assert_eq!(
            Deduplicator::normalize("https://example.com/page#section"),
            "https://example.com/page"
        );
    }

    #[test]
    fn test_normalize_lowercases_host() {
        assert_eq!(
            Deduplicator::normalize("https://Example.COM/Page"),
            "https://example.com/Page"
        );
    }

    #[test]
    fn test_normalize_removes_default_port() {
        assert_eq!(
            Deduplicator::normalize("https://example.com:443/page"),
            "https://example.com/page"
        );
        assert_eq!(
            Deduplicator::normalize("http://example.com:80/page"),
            "http://example.com/page"
        );
    }

    #[test]
    fn test_normalize_keeps_non_default_port() {
        assert_eq!(
            Deduplicator::normalize("https://example.com:8080/page"),
            "https://example.com:8080/page"
        );
    }

    #[test]
    fn test_normalize_removes_trailing_slash() {
        assert_eq!(
            Deduplicator::normalize("https://example.com/page/"),
            "https://example.com/page"
        );
        // Root path should keep trailing slash
        assert_eq!(
            Deduplicator::normalize("https://example.com/"),
            "https://example.com/"
        );
    }

    #[test]
    fn test_normalize_sorts_query_params() {
        assert_eq!(
            Deduplicator::normalize("https://example.com/page?z=1&a=2&m=3"),
            "https://example.com/page?a=2&m=3&z=1"
        );
    }

    #[test]
    fn test_dedup_detects_duplicates() {
        let mut dedup = Deduplicator::new();
        assert!(!dedup.is_duplicate("https://example.com/page"));
        assert!(dedup.is_duplicate("https://example.com/page"));
        assert!(!dedup.is_duplicate("https://example.com/other"));
    }

    #[test]
    fn test_dedup_normalizes_before_checking() {
        let mut dedup = Deduplicator::new();
        assert!(!dedup.is_duplicate("https://Example.COM/page#section"));
        assert!(dedup.is_duplicate("https://example.com/page"));
    }
}
