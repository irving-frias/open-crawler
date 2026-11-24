//! Sitemap XML generation from crawled pages (T4).
//!
//! Pure functions: no DB, no Tauri. The DB query lives in
//! `db::repos::export::get_sitemap_pages`; the command in
//! `features::export::commands::export_sitemap` writes the files.

use crate::seo::site::origin_of;

/// One crawled page, as needed for a `<url>` entry.
#[derive(Debug, Clone, PartialEq)]
pub struct SitemapPage {
    pub url: String,
    /// Crawl date in `YYYY-MM-DD` form (from the RFC3339 crawl timestamp).
    pub lastmod: Option<String>,
    pub depth: u32,
    pub seo_score: Option<f64>,
}

/// Max URLs per sitemap file (sitemaps.org protocol limit).
pub const MAX_URLS_PER_SITEMAP: usize = 50_000;

/// A generated file: its file name (relative to the chosen output path) and
/// its XML content. `name == None` means "write to the chosen path itself".
#[derive(Debug, Clone, PartialEq)]
pub struct SitemapFile {
    pub name: Option<String>,
    pub xml: String,
}

/// Priority for a page: `1.0 - 0.1 * depth` (floor 0.1), nudged by SEO score
/// (`+0.1` when `seo_score >= 80`, `-0.1` when `< 40`), clamped to [0.1, 1.0].
pub fn sitemap_priority(depth: u32, seo_score: Option<f64>) -> f64 {
    let mut p = 1.0 - 0.1 * depth as f64;
    if let Some(score) = seo_score {
        if score >= 80.0 {
            p += 0.1;
        } else if score < 40.0 {
            p -= 0.1;
        }
    }
    let p = p.clamp(0.1, 1.0);
    // Round to one decimal: the value is written with one decimal, and
    // float drift (e.g. 0.8 + 0.1 != 0.9) would otherwise leak into tests.
    (p * 10.0).round() / 10.0
}

fn xml_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(c),
        }
    }
    out
}

/// Build one `<urlset>` document from pages already sorted as they should
/// appear (depth asc, score desc).
pub fn build_sitemap_xml(pages: &[SitemapPage]) -> String {
    let mut xml = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n",
    );
    for page in pages {
        xml.push_str("  <url>\n    <loc>");
        xml.push_str(&xml_escape(&page.url));
        xml.push_str("</loc>\n");
        if let Some(lastmod) = &page.lastmod {
            xml.push_str("    <lastmod>");
            xml.push_str(&xml_escape(lastmod));
            xml.push_str("</lastmod>\n");
        }
        xml.push_str("    <priority>");
        xml.push_str(&format!(
            "{:.1}",
            sitemap_priority(page.depth, page.seo_score)
        ));
        xml.push_str("</priority>\n  </url>\n");
    }
    xml.push_str("</urlset>\n");
    xml
}

/// Split pages into files: one per origin, chunked at 50 000 URLs.
/// Returns `(parts, index)`: `parts` are the sitemap files (with their file
/// names relative to the chosen output directory), and `index` is the
/// `<sitemapindex>` document when there is more than one part (None when a
/// single file covers everything — then the caller writes the part itself to
/// the chosen path).
pub fn partition_sitemap(pages: &[SitemapPage]) -> (Vec<SitemapFile>, Option<String>) {
    // Group by origin, preserving first-seen order; sort each group by depth
    // asc then score desc so the best pages come first.
    let mut groups: Vec<(String, Vec<SitemapPage>)> = Vec::new();
    for page in pages {
        let origin = origin_of(&page.url).unwrap_or_else(|| page.url.clone());
        match groups.iter_mut().find(|(o, _)| *o == origin) {
            Some((_, g)) => g.push(page.clone()),
            None => groups.push((origin, vec![page.clone()])),
        }
    }
    for (_, g) in groups.iter_mut() {
        g.sort_by(|a, b| {
            a.depth.cmp(&b.depth).then(
                b.seo_score
                    .unwrap_or(0.0)
                    .partial_cmp(&a.seo_score.unwrap_or(0.0))
                    .unwrap_or(std::cmp::Ordering::Equal),
            )
        });
    }

    // Chunk each group and name the parts. A single part overall keeps
    // `name == None` (write to the chosen path); otherwise parts are named
    // `sitemap-<host>-<n>.xml` (or `sitemap-<n>.xml` for a single origin).
    let total_parts: usize = groups
        .iter()
        .map(|(_, g)| g.chunks(MAX_URLS_PER_SITEMAP).len())
        .sum();
    let multi_origin = groups.len() > 1;
    let mut parts: Vec<SitemapFile> = Vec::new();
    let mut index_entries: Vec<(String, String)> = Vec::new(); // (origin, file name)
    for (origin, group) in &groups {
        // File-safe host for part names (no port, no scheme, no `:`).
        let host = group
            .first()
            .and_then(|p| url::Url::parse(&p.url).ok())
            .and_then(|u| u.host_str().map(|h| h.to_string()))
            .unwrap_or_else(|| "site".to_string());
        for (i, chunk) in group.chunks(MAX_URLS_PER_SITEMAP).enumerate() {
            let name = if total_parts == 1 {
                None
            } else if multi_origin {
                Some(format!("sitemap-{}-{}.xml", host, i + 1))
            } else {
                Some(format!("sitemap-{}.xml", i + 1))
            };
            let xml = build_sitemap_xml(chunk);
            if let Some(n) = &name {
                index_entries.push((origin.clone(), n.clone()));
            }
            parts.push(SitemapFile { name, xml });
        }
    }

    let index = if total_parts > 1 {
        let mut xml = String::from(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<sitemapindex xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n",
        );
        for (origin, name) in &index_entries {
            xml.push_str("  <sitemap>\n    <loc>");
            xml.push_str(&xml_escape(&format!("{}/{}", origin, name)));
            xml.push_str("</loc>\n  </sitemap>\n");
        }
        xml.push_str("</sitemapindex>\n");
        Some(xml)
    } else {
        None
    };
    (parts, index)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page(url: &str, depth: u32, seo_score: Option<f64>) -> SitemapPage {
        SitemapPage {
            url: url.to_string(),
            lastmod: Some("2026-11-24".to_string()),
            depth,
            seo_score,
        }
    }

    #[test]
    fn priority_from_depth_and_score() {
        assert_eq!(sitemap_priority(0, None), 1.0);
        assert_eq!(sitemap_priority(1, None), 0.9);
        assert_eq!(sitemap_priority(5, None), 0.5);
        assert_eq!(sitemap_priority(20, None), 0.1);
        // Score nudges, clamped to [0.1, 1.0].
        assert_eq!(sitemap_priority(0, Some(95.0)), 1.0);
        assert_eq!(sitemap_priority(1, Some(95.0)), 1.0);
        assert_eq!(sitemap_priority(2, Some(95.0)), 0.9);
        assert_eq!(sitemap_priority(2, Some(20.0)), 0.7);
        assert_eq!(sitemap_priority(9, Some(10.0)), 0.1);
    }

    #[test]
    fn build_xml_escapes_and_formats() {
        let xml = build_sitemap_xml(&[
            page("https://example.com/a?x=1&y=2", 0, Some(90.0)),
            SitemapPage {
                url: "https://example.com/b".to_string(),
                lastmod: None,
                depth: 3,
                seo_score: None,
            },
        ]);
        assert!(xml.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<urlset"));
        assert!(xml.contains("<loc>https://example.com/a?x=1&amp;y=2</loc>"));
        assert!(xml.contains("<lastmod>2026-11-24</lastmod>"));
        assert!(xml.contains("<priority>1.0</priority>"));
        assert!(xml.contains("<priority>0.7</priority>"));
        assert_eq!(
            xml.matches("<lastmod>").count(),
            1,
            "no lastmod without a date"
        );
        assert!(xml.ends_with("</urlset>\n"));
    }

    #[test]
    fn partition_single_file_keeps_chosen_path() {
        let pages = vec![
            page("https://example.com/", 0, None),
            page("https://example.com/a", 1, None),
        ];
        let (parts, index) = partition_sitemap(&pages);
        assert_eq!(parts.len(), 1);
        assert_eq!(parts[0].name, None);
        assert_eq!(index, None);
    }

    #[test]
    fn partition_splits_at_50k_and_builds_index() {
        let mut pages = Vec::new();
        for i in 0..50_001 {
            pages.push(page(&format!("https://example.com/p{i}"), 1, None));
        }
        let (parts, index) = partition_sitemap(&pages);
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].name.as_deref(), Some("sitemap-1.xml"));
        assert_eq!(parts[1].name.as_deref(), Some("sitemap-2.xml"));
        assert_eq!(parts[0].xml.matches("<url>").count(), 50_000);
        assert_eq!(parts[1].xml.matches("<url>").count(), 1);
        let index = index.expect("index expected for multiple parts");
        assert!(index.contains("<loc>https://example.com/sitemap-1.xml</loc>"));
        assert!(index.contains("<loc>https://example.com/sitemap-2.xml</loc>"));
    }

    #[test]
    fn partition_groups_by_origin() {
        let pages = vec![
            page("https://a.com/", 0, None),
            page("https://b.com/", 0, None),
            page("https://a.com/x", 1, None),
        ];
        let (parts, index) = partition_sitemap(&pages);
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].name.as_deref(), Some("sitemap-a.com-1.xml"));
        assert_eq!(parts[1].name.as_deref(), Some("sitemap-b.com-1.xml"));
        let index = index.expect("index expected for multiple origins");
        assert!(index.contains("<loc>https://a.com/sitemap-a.com-1.xml</loc>"));
        assert!(index.contains("<loc>https://b.com/sitemap-b.com-1.xml</loc>"));
    }

    #[test]
    fn partition_sorts_by_depth_then_score() {
        let pages = vec![
            page("https://example.com/deep", 3, Some(99.0)),
            page("https://example.com/home", 0, Some(50.0)),
            page("https://example.com/mid-high", 1, Some(90.0)),
            page("https://example.com/mid-low", 1, Some(30.0)),
        ];
        let (parts, _) = partition_sitemap(&pages);
        let xml = &parts[0].xml;
        let pos = |u: &str| xml.find(u).unwrap();
        assert!(pos("home") < pos("mid-high"));
        assert!(pos("mid-high") < pos("mid-low"));
        assert!(pos("mid-low") < pos("deep"));
    }
}
