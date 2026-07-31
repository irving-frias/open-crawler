use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone)]
pub struct UrlEntry {
    pub url: String,
    pub depth: u32,
}

pub struct Frontier {
    domains: VecDeque<String>,
    queues: HashMap<String, VecDeque<UrlEntry>>,
    domain_index: usize,
    max_depth: u32,
    max_urls: usize,
    total_pending: usize,
}

impl Frontier {
    pub fn new(max_depth: u32, max_urls: usize) -> Self {
        Self {
            domains: VecDeque::new(),
            queues: HashMap::new(),
            domain_index: 0,
            max_depth,
            max_urls,
            total_pending: 0,
        }
    }

    fn extract_domain(url: &str) -> String {
        url.split("://")
            .nth(1)
            .unwrap_or(url)
            .split('/')
            .next()
            .unwrap_or(url)
            .to_string()
    }

    pub fn push(&mut self, url: String, depth: u32) -> bool {
        if depth > self.max_depth {
            return false;
        }
        if self.total_pending >= self.max_urls {
            return false;
        }

        let domain = Self::extract_domain(&url);

        if !self.queues.contains_key(&domain) {
            self.domains.push_back(domain.clone());
            self.queues.insert(domain.clone(), VecDeque::new());
        }

        let entry = UrlEntry { url, depth };
        if let Some(queue) = self.queues.get_mut(&domain) {
            queue.push_back(entry);
            self.total_pending += 1;
            true
        } else {
            false
        }
    }

    pub fn pop(&mut self) -> Option<UrlEntry> {
        if self.domains.is_empty() {
            return None;
        }

        let attempts = self.domains.len();
        for _ in 0..attempts {
            let domain = self
                .domains
                .get(self.domain_index % self.domains.len())
                .cloned();

            if let Some(domain) = domain {
                self.domain_index = self.domain_index.wrapping_add(1);

                if let Some(queue) = self.queues.get_mut(&domain) {
                    if let Some(entry) = queue.pop_front() {
                        self.total_pending -= 1;

                        // Clean up empty domains
                        if queue.is_empty() {
                            self.queues.remove(&domain);
                            if let Some(pos) = self.domains.iter().position(|d| d == &domain) {
                                self.domains.remove(pos);
                                if self.domain_index > pos && self.domain_index > 0 {
                                    self.domain_index -= 1;
                                }
                            }
                        }

                        return Some(entry);
                    }
                }
            }

            self.domain_index = self.domain_index.wrapping_add(1);
        }

        None
    }

    pub fn len(&self) -> usize {
        self.total_pending
    }

    pub fn is_empty(&self) -> bool {
        self.total_pending == 0
    }

    pub fn domain_count(&self) -> usize {
        self.domains.len()
    }

    pub fn drain_all(&self) -> Vec<(String, u32)> {
        let mut entries = Vec::new();
        for queue in self.queues.values() {
            for entry in queue {
                entries.push((entry.url.clone(), entry.depth));
            }
        }
        entries
    }

    pub fn restore(&mut self, entries: Vec<(String, u32)>) {
        for (url, depth) in entries {
            self.push(url, depth);
        }
    }

    #[cfg(test)]
    pub fn clear(&mut self) {
        self.domains.clear();
        self.queues.clear();
        self.domain_index = 0;
        self.total_pending = 0;
    }
}

impl Default for Frontier {
    fn default() -> Self {
        Self::new(10, 100_000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_push_pop() {
        let mut frontier = Frontier::new(10, 1000);
        assert!(frontier.push("https://example.com/page1".to_string(), 0));
        assert!(frontier.push("https://example.com/page2".to_string(), 1));
        assert!(frontier.push("https://other.com/page1".to_string(), 0));
        assert_eq!(frontier.len(), 3);
        assert_eq!(frontier.domain_count(), 2);

        let entry = frontier.pop();
        assert!(entry.is_some());
        assert_eq!(frontier.len(), 2);
    }

    #[test]
    fn test_domain_rotation() {
        let mut frontier = Frontier::new(10, 1000);
        frontier.push("https://a.com/1".to_string(), 0);
        frontier.push("https://b.com/1".to_string(), 0);
        frontier.push("https://a.com/2".to_string(), 1);
        frontier.push("https://b.com/2".to_string(), 1);

        let e1 = frontier.pop().unwrap();
        let e2 = frontier.pop().unwrap();
        // Should alternate between domains
        let d1 = Frontier::extract_domain(&e1.url);
        let d2 = Frontier::extract_domain(&e2.url);
        assert_ne!(d1, d2);
    }

    #[test]
    fn test_max_depth() {
        let mut frontier = Frontier::new(2, 1000);
        assert!(frontier.push("https://a.com/1".to_string(), 0));
        assert!(frontier.push("https://a.com/2".to_string(), 2));
        assert!(!frontier.push("https://a.com/3".to_string(), 3));
        assert_eq!(frontier.len(), 2);
    }

    #[test]
    fn test_max_urls() {
        let mut frontier = Frontier::new(10, 2);
        assert!(frontier.push("https://a.com/1".to_string(), 0));
        assert!(frontier.push("https://b.com/1".to_string(), 0));
        assert!(!frontier.push("https://c.com/1".to_string(), 0));
        assert_eq!(frontier.len(), 2);
    }

    #[test]
    fn test_drain_and_restore() {
        let mut frontier = Frontier::new(10, 1000);
        frontier.push("https://a.com/1".to_string(), 0);
        frontier.push("https://b.com/1".to_string(), 1);

        let drained = frontier.drain_all();
        assert_eq!(drained.len(), 2);

        frontier.clear();
        assert!(frontier.is_empty());

        frontier.restore(drained);
        assert_eq!(frontier.len(), 2);
    }

    #[test]
    fn test_empty_frontier() {
        let mut frontier = Frontier::new(10, 1000);
        assert!(frontier.is_empty());
        assert!(frontier.pop().is_none());
    }
}
