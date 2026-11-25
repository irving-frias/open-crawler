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
    /// `/llms.txt` (T21): the optional markdown index of the site for LLM
    /// crawlers. Presence is informational only — Google states it is not
    /// required to appear in its AI features.
    pub llms_txt: Option<SiteResource>,
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
    let llms_url = format!("{origin}/llms.txt");
    let (robots, sitemap, llms) = tokio::join!(
        fetch_resource(client, &robots_url),
        fetch_resource(client, &sitemap_url),
        fetch_resource(client, &llms_url),
    );
    let resources = Arc::new(SiteResources {
        robots_txt: robots,
        sitemap_xml: sitemap,
        llms_txt: llms,
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

/// Whether `agent` may fetch `path` under the robots.txt in `body`
/// (T21, AI-crawler visibility). Simplified RFC 9309: groups are selected
/// by the longest matching `User-agent:` token (`*` is the fallback, agent
/// matching is case-insensitive); within the group the longest matching
/// `Allow`/`Disallow` rule wins and a tie goes to `Allow`. Rules support
/// `*` (any sequence) and a trailing `$` (end of path). An empty
/// `Disallow:` allows everything; a missing group allows everything.
pub fn is_path_allowed_by_robots(body: &str, agent: &str, path: &str) -> bool {
    let agent = agent.to_lowercase();
    // (agent tokens, allow rules, disallow rules) per group. Several
    // `User-agent:` lines before the first rule belong to one group.
    let mut groups: Vec<(Vec<String>, Vec<String>, Vec<String>)> = Vec::new();
    let mut current: Option<usize> = None;
    for raw_line in body.lines() {
        let line = raw_line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let key = key.trim().to_lowercase();
        let value = value.trim();
        match key.as_str() {
            "user-agent" => {
                let token = value.to_lowercase();
                // Still collecting agents for the current group (no rules
                // yet): the token joins it; otherwise start a new group.
                let joins_current = match current {
                    Some(idx) => groups[idx].1.is_empty() && groups[idx].2.is_empty(),
                    None => false,
                };
                if joins_current {
                    let idx = current.unwrap();
                    if !groups[idx].0.contains(&token) {
                        groups[idx].0.push(token);
                    }
                } else {
                    groups.push((vec![token], Vec::new(), Vec::new()));
                    current = Some(groups.len() - 1);
                }
            }
            "allow" | "disallow" => {
                let Some(idx) = current else { continue };
                if key == "allow" {
                    groups[idx].1.push(value.to_string());
                } else {
                    groups[idx].2.push(value.to_string());
                }
            }
            _ => {}
        }
    }

    // Most specific matching group wins (longest token); `*` matches all.
    let mut best: Option<&(Vec<String>, Vec<String>, Vec<String>)> = None;
    let mut best_len = 0usize;
    for group in &groups {
        let mut len = 0usize;
        let mut matches = false;
        for token in &group.0 {
            if token == "*" {
                matches = true;
            } else if !token.is_empty() && agent.starts_with(token.as_str()) {
                matches = true;
                len = len.max(token.len());
            }
        }
        if !matches {
            continue;
        }
        if best.is_none() || len > best_len {
            best = Some(group);
            best_len = len;
        }
    }
    let Some((_, allows, disallows)) = best else {
        return true;
    };

    // Longest matching rule wins; a tie goes to Allow.
    let mut best_allow = 0usize;
    let mut best_disallow = 0usize;
    for rule in allows {
        if rule.is_empty() {
            continue;
        }
        if let Some(len) = rule_match_len(rule, path) {
            best_allow = best_allow.max(len);
        }
    }
    for rule in disallows {
        if rule.is_empty() {
            continue; // Empty Disallow allows everything.
        }
        if let Some(len) = rule_match_len(rule, path) {
            best_disallow = best_disallow.max(len);
        }
    }
    if best_disallow == 0 && best_allow == 0 {
        return true;
    }
    best_allow >= best_disallow
}

/// Length of the literal prefix of `rule` that matches `path`, or None.
/// `*` matches any sequence, a trailing `$` anchors the end.
fn rule_match_len(rule: &str, path: &str) -> Option<usize> {
    let (pattern, anchored) = match rule.strip_suffix('$') {
        Some(p) => (p, true),
        None => (rule, false),
    };
    let path_bytes = path.as_bytes();
    let pat_bytes = pattern.as_bytes();
    // Greedy `*` handling: try every split point.
    fn matches(pat: &[u8], text: &[u8], anchored: bool) -> bool {
        if pat.is_empty() {
            return !anchored || text.is_empty();
        }
        if pat[0] == b'*' {
            for i in 0..=text.len() {
                if matches(&pat[1..], &text[i..], anchored) {
                    return true;
                }
            }
            return false;
        }
        if text.is_empty() || pat[0] != text[0] {
            return false;
        }
        matches(&pat[1..], &text[1..], anchored)
    }
    if !matches(pat_bytes, path_bytes, anchored) {
        return None;
    }
    // Score by the literal (non-`*`) length so the longest rule wins.
    Some(pat_bytes.iter().filter(|b| **b != b'*').count())
}

/// Whether an llms.txt body is well formed (T21): its first non-empty line
/// is an H1 (`# ...`) and it contains at least one markdown link.
pub fn is_well_formed_llms_txt(body: &str) -> bool {
    let mut first = true;
    let mut has_link = false;
    for line in body.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if first {
            if !line.starts_with("# ") {
                return false;
            }
            first = false;
        }
        if line.contains("](") {
            has_link = true;
        }
    }
    !first && has_link
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

    #[test]
    fn test_robots_specific_group_beats_wildcard() {
        let body = "User-agent: *\nAllow: /\n\nUser-agent: PerplexityBot\nDisallow: /private/\n";
        assert!(!is_path_allowed_by_robots(
            body,
            "PerplexityBot",
            "/private/page"
        ));
        assert!(is_path_allowed_by_robots(body, "PerplexityBot", "/public"));
        assert!(is_path_allowed_by_robots(
            body,
            "OAI-SearchBot",
            "/private/page"
        ));
    }

    #[test]
    fn test_robots_allow_wins_tie_and_longest_rule_wins() {
        // Tie: same length Allow beats Disallow.
        let tie = "User-agent: *\nDisallow: /a\nAllow: /a\n";
        assert!(is_path_allowed_by_robots(tie, "Googlebot", "/a"));
        // Longest rule wins over a shorter Allow.
        let longest = "User-agent: *\nAllow: /private\nDisallow: /private/secret\n";
        assert!(!is_path_allowed_by_robots(
            longest,
            "Googlebot",
            "/private/secret/x"
        ));
        assert!(is_path_allowed_by_robots(
            longest,
            "Googlebot",
            "/private/ok"
        ));
        // Empty Disallow allows everything; `Disallow: /` blocks everything.
        assert!(is_path_allowed_by_robots(
            "User-agent: *\nDisallow:\n",
            "Googlebot",
            "/anything"
        ));
        assert!(!is_path_allowed_by_robots(
            "User-agent: *\nDisallow: /\n",
            "Googlebot",
            "/anything"
        ));
    }

    #[test]
    fn test_robots_multi_agent_group_and_wildcards() {
        let body = "User-agent: OAI-SearchBot\nUser-agent: ChatGPT-User\nDisallow: /drafts/*\n";
        assert!(!is_path_allowed_by_robots(
            body,
            "OAI-SearchBot",
            "/drafts/x"
        ));
        assert!(!is_path_allowed_by_robots(
            body,
            "ChatGPT-User",
            "/drafts/x"
        ));
        assert!(is_path_allowed_by_robots(
            body,
            "PerplexityBot",
            "/drafts/x"
        ));
        // Case-insensitive agent match; `$` anchors the end.
        let anchored = "User-agent: *\nDisallow: /*.pdf$\n";
        assert!(!is_path_allowed_by_robots(anchored, "bot", "/file.pdf"));
        assert!(is_path_allowed_by_robots(anchored, "bot", "/file.pdf/x"));
    }

    #[test]
    fn test_llms_txt_well_formed() {
        assert!(is_well_formed_llms_txt(
            "# My Site\n\n- [Home](https://example.com/)\n- [Guide](https://example.com/guide)\n"
        ));
        assert!(!is_well_formed_llms_txt(
            "My Site\n\n- [Home](https://example.com/)\n"
        ));
        assert!(!is_well_formed_llms_txt("# My Site\n\nNo links here.\n"));
        assert!(!is_well_formed_llms_txt(""));
    }
}
