use scraper::{Html, Selector};
use url::Url;

use crate::crawler::parser::{SemanticIssue, SeoData};

use super::audit::{AuditContext, CheckResult};

/// Extra page signals extracted from the raw HTML (a second, light DOM pass).
#[derive(Debug, Clone, Default)]
pub struct PageExtras {
    pub json_ld_types: Vec<String>,
    pub json_ld_raw: Vec<serde_json::Value>,
    pub has_json_ld: bool,
    pub viewport: bool,
    pub favicon: bool,
    pub charset: bool,
    pub doctype: bool,
    pub img_total: usize,
    pub img_with_dimensions: usize,
    pub img_srcset: usize,
    pub img_lazy: usize,
    pub preconnect_or_preload: bool,
    pub p_count: usize,
    pub word_count: usize,
    pub sentence_count: usize,
    pub question_headings: usize,
    pub url_has_underscore: bool,
    pub table_total: usize,
    pub table_with_headers: usize,
    pub table_with_caption: usize,
    pub figure_total: usize,
    pub figure_with_caption: usize,
    pub details_summary: usize,
    pub video_total: usize,
    pub video_accessible: usize,
    pub iframe_total: usize,
    pub iframe_with_title: usize,
    pub input_total: usize,
    pub autocomplete_inputs: usize,
    pub text_bytes: usize,
    pub html_bytes: usize,
    pub div_with_direct_text: usize,
    pub scripts_render_blocking: usize,
    pub internal_count: usize,
    pub external_count: usize,
    pub meta_robots: Option<String>,
    pub answer_section_words: usize,
    pub time_datetime_count: usize,
    pub privacy_link_count: usize,
    pub consent_banner: bool,
}

impl PageExtras {
    #[allow(clippy::field_reassign_with_default)]
    pub fn extract(html: &str, url: &str) -> Self {
        let document = Html::parse_document(html);
        let mut extras = PageExtras::default();

        // Doctype
        extras.doctype = html
            .trim_start()
            .to_ascii_lowercase()
            .starts_with("<!doctype");

        // Charset / viewport
        extras.charset = has_selector(&document, "meta[charset]");
        extras.viewport = has_selector(&document, r#"meta[name="viewport"]"#);

        // Favicon
        extras.favicon = has_selector(&document, r#"link[rel~="icon"], link[rel="shortcut icon"]"#);

        // Resource hints
        extras.preconnect_or_preload =
            has_selector(&document, r#"link[rel="preconnect"], link[rel="preload"]"#);

        // Images with explicit dimensions / srcset / lazy loading
        if let Ok(sel) = Selector::parse("img") {
            let imgs: Vec<_> = document.select(&sel).collect();
            extras.img_total = imgs.len();
            extras.img_with_dimensions = imgs
                .iter()
                .filter(|el| {
                    el.value().attr("width").is_some() && el.value().attr("height").is_some()
                })
                .count();
            extras.img_srcset = imgs
                .iter()
                .filter(|el| el.value().attr("srcset").is_some())
                .count();
            extras.img_lazy = imgs
                .iter()
                .filter(|el| {
                    let loading = el.value().attr("loading").unwrap_or("");
                    loading == "lazy" || loading == "eager"
                })
                .count();
        }

        // Paragraph count
        extras.p_count = count_selector(&document, "p");

        // Tables
        extras.table_total = count_selector(&document, "table");
        extras.table_with_headers = count_selector(&document, "table:has(th)");
        extras.table_with_caption = count_selector(&document, "table:has(caption)");

        // Figures
        extras.figure_total = count_selector(&document, "figure");
        extras.figure_with_caption = count_selector(&document, "figure:has(figcaption)");

        // details/summary (FAQ-like accordions)
        extras.details_summary = count_selector(&document, "details");

        // Video / iframe accessibility
        extras.video_total = count_selector(&document, "video");
        extras.video_accessible = count_selector(&document, "video[controls], video:has(track)");
        extras.iframe_total = count_selector(&document, "iframe");
        extras.iframe_with_title = count_selector(&document, "iframe[title]");

        // Form inputs with an autocomplete hint
        if let Ok(sel) = Selector::parse("input:not([type='hidden'])") {
            let inputs: Vec<_> = document.select(&sel).collect();
            extras.input_total = inputs.len();
            let with_autocomplete = inputs
                .iter()
                .filter(|el| el.value().attr("autocomplete").is_some())
                .count();
            extras.autocomplete_inputs = if inputs.is_empty() {
                0
            } else {
                (with_autocomplete * 100) / inputs.len()
            };
        }

        // Render-blocking scripts (external scripts without async/defer, excluding type=module)
        extras.scripts_render_blocking = count_selector(
            &document,
            r#"script[src]:not([async]):not([defer]):not([type="module"]):not([type="application/ld+json"])"#,
        );

        // Divs holding substantial direct text (body copy in <div> instead of <p>)
        if let Ok(sel) = Selector::parse("div") {
            extras.div_with_direct_text = document
                .select(&sel)
                .filter(|el| direct_text_words(el) >= 20)
                .count();
        }

        // JSON-LD structured data (raw values kept for validation)
        if let Ok(sel) = Selector::parse(r#"script[type="application/ld+json"]"#) {
            for script in document.select(&sel) {
                let text = script.text().collect::<Vec<_>>().join("");
                if let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) {
                    extras.has_json_ld = true;
                    extras.json_ld_raw.push(value.clone());
                    collect_json_ld_types(&value, &mut extras.json_ld_types);
                }
            }
        }

        // Question-style headings (SXO / AEO signal)
        if let Ok(sel) = Selector::parse("h2, h3") {
            extras.question_headings = document
                .select(&sel)
                .filter(|el| {
                    let text = el.text().collect::<Vec<_>>().join("").trim().to_lowercase();
                    text.split_whitespace()
                        .next()
                        .map(|w| QUESTION_WORDS.contains(&w))
                        .unwrap_or(false)
                })
                .count();
        }

        // Visible text stats
        let visible = collect_visible_text(&document);
        extras.word_count = visible.split_whitespace().count();
        extras.sentence_count = count_sentences(&visible).max(1);
        extras.text_bytes = visible.len();
        extras.html_bytes = html.len();

        // Direct answer section: words in the first paragraph block right after the H1
        extras.answer_section_words = answer_section_words(&document);

        // <time datetime> count (freshness signal)
        extras.time_datetime_count = count_selector(&document, "time[datetime]");

        // Compliance: links pointing to privacy / cookies / terms pages
        if let Ok(sel) = Selector::parse("a[href]") {
            extras.privacy_link_count = document
                .select(&sel)
                .filter(|el| {
                    let href = el.value().attr("href").unwrap_or("").to_ascii_lowercase();
                    let text = el.text().collect::<Vec<_>>().join("").to_ascii_lowercase();
                    ["privacy", "cookies", "cookie", "gdpr", "terms", "legal", "datenschutz"]
                        .iter()
                        .any(|k| href.contains(k) || text.contains(k))
                })
                .count();
        }

        // Compliance: cookie-consent banner / CMP markers
        let lower_html = html.to_ascii_lowercase();
        extras.consent_banner = CONSENT_MARKERS
            .iter()
            .any(|m| lower_html.contains(m));

        // URL heuristics
        if let Ok(parsed) = Url::parse(url) {
            extras.url_has_underscore = parsed.path().contains('_');
        }

        // robots meta (also surfaced in PageExtras for directive checks)
        if let Ok(sel) = Selector::parse(r#"meta[name="robots"]"#) {
            extras.meta_robots = document
                .select(&sel)
                .next()
                .and_then(|el| el.value().attr("content").map(|s| s.to_string()));
        }

        // Internal / external link counts
        if let Ok(sel) = Selector::parse("a[href]") {
            let origin = Url::parse(url).ok();
            for el in document.select(&sel) {
                let Some(href) = el.value().attr("href") else {
                    continue;
                };
                if !href.starts_with("http://") && !href.starts_with("https://") {
                    continue;
                }
                let Ok(parsed) = Url::parse(href) else {
                    continue;
                };
                let same_host = match &origin {
                    Some(o) => parsed.host_str() == o.host_str(),
                    None => false,
                };
                if same_host {
                    extras.internal_count += 1;
                } else if parsed.host_str().is_some() {
                    extras.external_count += 1;
                }
            }
        }

        extras
    }
}

const QUESTION_WORDS: &[&str] = &[
    "what", "how", "why", "who", "where", "when", "which", "can", "could", "does", "do", "is",
    "are", "should",
];

/// Substrings that typically identify a cookie-consent banner or a consent
/// management platform (CMP) in the raw HTML.
const CONSENT_MARKERS: &[&str] = &[
    "cookie-banner",
    "cookie_banner",
    "cookie-consent",
    "cookie_consent",
    "cookie-notice",
    "cc-banner",
    "onetrust",
    "iubenda",
    "cookiebot",
    "didomi",
    "consent-manager",
    "accept cookies",
    "accept all cookies",
    "accept the use of cookies",
    "cookie settings",
    "manage consent",
];

fn has_selector(document: &Html, selector_str: &str) -> bool {
    Selector::parse(selector_str)
        .ok()
        .map(|s| document.select(&s).next().is_some())
        .unwrap_or(false)
}

fn count_selector(document: &Html, selector_str: &str) -> usize {
    Selector::parse(selector_str)
        .map(|s| document.select(&s).count())
        .unwrap_or(0)
}

/// Counts words in the direct text children of an element (ignoring descendants).
fn direct_text_words(el: &scraper::ElementRef) -> usize {
    el.children()
        .filter_map(|child| match child.value() {
            scraper::node::Node::Text(t) => Some(t.split_whitespace().count()),
            _ => None,
        })
        .sum()
}

/// Word count of the paragraph block that immediately follows the first <h1>.
/// Used by the direct-answer (AEO) check: a 40-100 word passage right after the
/// H1 is a strong signal the page opens with a scannable, extractable answer.
fn answer_section_words(document: &Html) -> usize {
    let Ok(h1_sel) = Selector::parse("h1") else {
        return 0;
    };
    let Some(h1) = document.select(&h1_sel).next() else {
        return 0;
    };
    let mut words = 0usize;
    for sibling in h1.next_siblings() {
        let Some(el) = sibling.value().as_element() else {
            continue;
        };
        if matches!(
            el.name(),
            "h2" | "h3" | "h4" | "script" | "style" | "nav" | "footer"
        ) {
            break;
        }
        if let Some(child_el) = scraper::ElementRef::wrap(sibling) {
            let mut parts: Vec<String> = Vec::new();
            collect_text(&child_el, &mut parts);
            words += parts.join(" ").split_whitespace().count();
        }
        if words >= 200 {
            break;
        }
    }
    words
}

/// Formats a huge internal:external ratio (no external links) as a readable value.
fn ratio_display(ratio: f64) -> String {
    if ratio == f64::MAX {
        "∞".to_string()
    } else if ratio >= 100.0 {
        format!("{ratio:.0}")
    } else {
        format!("{ratio:.1}")
    }
}

/// Every JSON-LD block must carry @context and @type to be valid structured data.
fn all_json_ld_valid(blocks: &[serde_json::Value]) -> bool {
    if blocks.is_empty() {
        return true;
    }
    blocks.iter().all(|v| json_ld_block_valid(v, true))
}

fn json_ld_block_valid(value: &serde_json::Value, top_level: bool) -> bool {
    match value {
        serde_json::Value::Array(items) => items.iter().all(|i| json_ld_block_valid(i, top_level)),
        serde_json::Value::Object(map) => {
            // Top-level blocks declare @context; nested entity objects inherit it.
            if top_level && !map.contains_key("@context") {
                return false;
            }
            let has_type = match map.get("@type") {
                Some(serde_json::Value::String(s)) => !s.is_empty(),
                Some(serde_json::Value::Array(l)) => !l.is_empty(),
                _ => false,
            };
            if !has_type {
                return false;
            }
            let mut ok = true;
            for (key, child) in map {
                if key == "@context" || key == "@type" {
                    continue;
                }
                if key == "@graph" && !json_ld_block_valid(child, false) {
                    ok = false;
                }
                if is_entity_value(child) && !json_ld_block_valid(child, false) {
                    ok = false;
                }
            }
            ok
        }
        _ => false,
    }
}

/// Whether a value looks like a nested schema.org entity (has @type) vs a scalar.
fn is_entity_value(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::Object(map) => map.contains_key("@type") || map.contains_key("@id"),
        serde_json::Value::Array(items) => items
            .iter()
            .any(|i| matches!(i, serde_json::Value::Object(m) if m.contains_key("@type"))),
        _ => false,
    }
}

/// Fraction (0-1) of required schema properties that are present across blocks.
fn schema_completeness_score(blocks: &[serde_json::Value]) -> f64 {
    if blocks.is_empty() {
        return 0.0;
    }
    let mut present = 0usize;
    let mut required = 0usize;
    for block in blocks {
        collect_required(block, &mut present, &mut required);
    }
    if required == 0 {
        return 0.0;
    }
    present as f64 / required as f64
}

fn collect_required(value: &serde_json::Value, present: &mut usize, required: &mut usize) {
    let (entities, required_keys) = match value {
        serde_json::Value::Object(map) => {
            let ty = map.get("@type").and_then(|t| t.as_str()).unwrap_or("");
            let keys: &[&str] = match ty {
                "Article" | "NewsArticle" | "BlogPosting" => {
                    &["headline", "datePublished", "author"]
                }
                "Person" => &["name"],
                "Organization" => &["name"],
                "WebSite" => &["name"],
                "FAQPage" => &["mainEntity"],
                "Product" => &["name"],
                "BreadcrumbList" => &["itemListElement"],
                "Recipe" => &["name", "recipeIngredient", "recipeInstructions"],
                _ => &[],
            };
            let children: Vec<_> = map
                .iter()
                .filter(|(k, _)| !k.starts_with('@'))
                .map(|(_, v)| v)
                .collect();
            (children, keys)
        }
        serde_json::Value::Array(items) => {
            let children: Vec<_> = items.iter().collect();
            (children, &[] as &[&str])
        }
        _ => (Vec::new(), &[] as &[&str]),
    };

    *required += required_keys.len();
    for key in required_keys {
        let found = match value {
            serde_json::Value::Object(map) => map.get(*key).map(|v| !v.is_null()).unwrap_or(false),
            _ => false,
        };
        if found {
            *present += 1;
        }
    }

    for child in entities {
        collect_required(child, present, required);
    }
}

/// Whether any JSON-LD block declares an author entity.
fn json_ld_has_author(blocks: &[serde_json::Value]) -> bool {
    blocks.iter().any(block_has_author)
}

fn block_has_author(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::Array(items) => items.iter().any(block_has_author),
        serde_json::Value::Object(map) => {
            if map.contains_key("author") {
                return true;
            }
            map.values().any(block_has_author)
        }
        _ => false,
    }
}

/// Whether any JSON-LD block exposes freshness dates.
fn json_ld_has_freshness(blocks: &[serde_json::Value]) -> bool {
    blocks.iter().any(block_has_freshness)
}

fn block_has_freshness(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::Array(items) => items.iter().any(block_has_freshness),
        serde_json::Value::Object(map) => {
            if map.contains_key("datePublished") || map.contains_key("dateModified") {
                return true;
            }
            map.values().any(block_has_freshness)
        }
        _ => false,
    }
}

/// Whether any JSON-LD block declares a privacy policy or data policies.
fn json_ld_has_privacy(blocks: &[serde_json::Value]) -> bool {
    blocks.iter().any(block_has_privacy)
}

fn block_has_privacy(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::Array(items) => items.iter().any(block_has_privacy),
        serde_json::Value::Object(map) => {
            if map.contains_key("privacyPolicy") || map.contains_key("policies") {
                return true;
            }
            map.values().any(block_has_privacy)
        }
        _ => false,
    }
}

fn collect_json_ld_types(value: &serde_json::Value, out: &mut Vec<String>) {
    match value {
        serde_json::Value::Array(items) => {
            for item in items {
                collect_json_ld_types(item, out);
            }
        }
        serde_json::Value::Object(map) => {
            if let Some(types) = map.get("@type") {
                match types {
                    serde_json::Value::String(t) => out.push(t.clone()),
                    serde_json::Value::Array(list) => {
                        for t in list {
                            if let serde_json::Value::String(s) = t {
                                out.push(s.clone());
                            }
                        }
                    }
                    _ => {}
                }
            }
            for (key, child) in map {
                if key == "@graph" {
                    collect_json_ld_types(child, out);
                }
            }
        }
        _ => {}
    }
}

fn collect_visible_text(document: &Html) -> String {
    let Ok(body_sel) = Selector::parse("body") else {
        return String::new();
    };
    let Some(body) = document.select(&body_sel).next() else {
        return String::new();
    };
    let mut parts: Vec<String> = Vec::new();
    collect_text(&body, &mut parts);
    parts.join(" ")
}

fn collect_text(el: &scraper::ElementRef, out: &mut Vec<String>) {
    let name = el.value().name();
    if matches!(name, "script" | "style" | "noscript" | "template" | "svg") {
        return;
    }
    if el.value().attr("hidden").is_some() || el.value().attr("aria-hidden").is_some() {
        return;
    }
    for child in el.children() {
        match child.value() {
            scraper::node::Node::Text(t) => {
                let text = t.trim();
                if !text.is_empty() {
                    out.push(text.to_string());
                }
            }
            scraper::node::Node::Element(_) => {
                if let Some(child_el) = scraper::ElementRef::wrap(child) {
                    collect_text(&child_el, out);
                }
            }
            _ => {}
        }
    }
}

fn count_sentences(text: &str) -> usize {
    let mut count = 0usize;
    let mut prev_end = false;
    for c in text.chars() {
        if matches!(c, '.' | '!' | '?') {
            if !prev_end {
                count += 1;
                prev_end = true;
            }
        } else {
            prev_end = false;
        }
    }
    count
}

#[allow(clippy::too_many_arguments)]
fn check(
    id: &str,
    category: &str,
    severity: &str,
    passed: bool,
    weight: f64,
    message: impl Into<String>,
    guidance: impl Into<String>,
    evidence: Option<String>,
) -> CheckResult {
    CheckResult {
        id: id.to_string(),
        category: category.to_string(),
        severity: severity.to_string(),
        passed,
        weight,
        message: message.into(),
        guidance: guidance.into(),
        evidence,
        examples: Vec::new(),
    }
}

fn has_issue(seo: &SeoData, types: &[&str]) -> bool {
    seo.semantic_issues
        .iter()
        .any(|i| types.contains(&i.issue_type.as_str()))
}

fn token_set(s: &str) -> std::collections::HashSet<String> {
    s.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() > 2)
        .map(|w| w.to_string())
        .collect()
}

/// Runs every audit check and returns the flattened result list.
#[allow(clippy::too_many_lines)]
pub fn run_all(seo: &SeoData, extras: &PageExtras, ctx: &AuditContext) -> Vec<CheckResult> {
    let mut out: Vec<CheckResult> = Vec::new();

    // ==================== META & CONTENT ====================
    let title_present = seo.title.is_some();
    let title_len = seo.title.as_deref().map(|t| t.chars().count()).unwrap_or(0);
    out.push(check(
        "title_present",
        "meta",
        "error",
        title_present,
        3.0,
        if title_present {
            "Page has a title"
        } else {
            "Page has no <title>"
        },
        "Add a unique, descriptive <title> tag to every page.",
        None,
    ));
    out.push(check(
        "title_length",
        "meta",
        "warning",
        (30..=65).contains(&title_len),
        2.0,
        format!("Title length: {title_len} chars (target 30-65)"),
        "Keep titles between 30 and 65 characters so they render fully in search results.",
        Some(title_len.to_string()),
    ));

    let desc_present = seo.meta_description.is_some();
    let desc_len = seo
        .meta_description
        .as_deref()
        .map(|d| d.chars().count())
        .unwrap_or(0);
    out.push(check(
        "meta_description_present",
        "meta",
        "warning",
        desc_present,
        2.0,
        if desc_present {
            "Page has a meta description"
        } else {
            "Page has no meta description"
        },
        "Write a unique 50-160 character meta description for each page.",
        None,
    ));
    out.push(check(
        "meta_description_length",
        "meta",
        "warning",
        (50..=160).contains(&desc_len),
        2.0,
        format!("Meta description length: {desc_len} chars (target 50-160)"),
        "Adjust the meta description to between 50 and 160 characters.",
        Some(desc_len.to_string()),
    ));

    let h1_present = seo.h1.is_some();
    out.push(check(
        "h1_present",
        "meta",
        "warning",
        h1_present,
        2.0,
        if h1_present {
            "Page has one H1"
        } else {
            "Page has no <h1>"
        },
        "Add exactly one <h1> summarizing the page's main topic.",
        None,
    ));
    out.push(check(
        "h1_count",
        "meta",
        "warning",
        !has_issue(seo, &["multiple_h1"]),
        2.0,
        "Exactly one <h1> on the page",
        "Use a single <h1>; additional headings should use h2-h6.",
        None,
    ));
    out.push(check(
        "heading_hierarchy",
        "meta",
        "info",
        !has_issue(seo, &["heading_skip"]),
        1.0,
        "Heading hierarchy has no skipped levels",
        "Do not skip heading levels (h1 → h3). Use a logical outline.",
        None,
    ));

    let h1_title_overlap = match (&seo.title, &seo.h1) {
        (Some(t), Some(h)) => !token_set(t).is_disjoint(&token_set(h)),
        _ => false,
    };
    out.push(check(
        "h1_title_match",
        "meta",
        "info",
        h1_title_overlap,
        1.0,
        "H1 and title share topic keywords",
        "Align the <title> and <h1> so both clearly describe the page topic.",
        None,
    ));

    out.push(check(
        "word_count",
        "meta",
        "warning",
        extras.word_count >= 300,
        2.0,
        format!("Word count: {} (recommended ≥ 300)", extras.word_count),
        "Expand thin content to at least 300 words for better ranking.",
        Some(extras.word_count.to_string()),
    ));

    let top_keyword_density = seo
        .keywords
        .first()
        .map(|k| k.count as f64 / extras.word_count.max(1) as f64 * 100.0)
        .unwrap_or(0.0);
    out.push(check(
        "keyword_density",
        "meta",
        "info",
        (0.5..=5.0).contains(&top_keyword_density),
        1.0,
        format!("Top keyword density: {top_keyword_density:.1}% (target 0.5-5%)"),
        "Avoid keyword stuffing; use the main keyword naturally a few times.",
        Some(format!("{top_keyword_density:.1}%")),
    ));

    let origin = Url::parse(&ctx.url).ok().and_then(|u| {
        u.host_str()
            .map(|h| (u.scheme().to_string(), h.to_string()))
    });
    let mut has_internal = false;
    let mut has_outbound = false;
    for link in &seo.outgoing_links {
        if !link.url.starts_with("http://") && !link.url.starts_with("https://") {
            continue;
        }
        if let Ok(parsed) = Url::parse(&link.url) {
            let same = match &origin {
                Some((scheme, host)) => {
                    parsed.scheme() == scheme && parsed.host_str() == Some(host)
                }
                None => false,
            };
            if same {
                has_internal = true;
            } else if parsed.host_str().is_some() {
                has_outbound = true;
            }
        }
    }
    out.push(check(
        "internal_links",
        "meta",
        "info",
        has_internal,
        1.0,
        "Page contains at least one internal link",
        "Link to other pages of your site so crawlers can discover content.",
        None,
    ));
    out.push(check(
        "outbound_links",
        "meta",
        "info",
        has_outbound,
        1.0,
        "Page contains at least one outbound link",
        "Cite related external sources to add context and authority.",
        None,
    ));

    // Internal:external ratio (10:1 recommended)
    let ie_ratio = if extras.external_count > 0 {
        extras.internal_count as f64 / extras.external_count as f64
    } else if extras.internal_count > 0 {
        f64::MAX
    } else {
        0.0
    };
    out.push(check(
        "internal_external_ratio",
        "meta",
        "info",
        extras.internal_count > 0 && ie_ratio >= 10.0,
        1.0,
        format!(
            "Internal/external link ratio: {}:1 (target ≥ 10:1)",
            ratio_display(ie_ratio)
        ),
        "Link mostly within your own site; cite external sources sparingly.",
        Some(format!(
            "{} internal / {} external",
            extras.internal_count, extras.external_count
        )),
    ));

    // Meta description topic match (title/description share keywords)
    let desc_topic_match = match (&seo.title, &seo.meta_description) {
        (Some(t), Some(d)) => !token_set(t).is_disjoint(&token_set(d)),
        _ => false,
    };
    out.push(check(
        "meta_description_topic_match",
        "meta",
        "info",
        desc_topic_match,
        1.0,
        "Meta description shares topic keywords with the title",
        "Make the meta description summarize the same topic as the title and content.",
        None,
    ));

    // ==================== TECHNICAL & MOBILE ====================
    let https = ctx.url.starts_with("https://");
    out.push(check(
        "https_used",
        "technical",
        "warning",
        https,
        2.0,
        if https {
            "Served over HTTPS"
        } else {
            "Not served over HTTPS"
        },
        "Serve the site over HTTPS with a valid certificate.",
        None,
    ));
    out.push(check(
        "status_ok",
        "technical",
        "error",
        ctx.status_code < 400,
        3.0,
        format!("HTTP status {}", ctx.status_code),
        "Fix the page so it returns a 200 status instead of an error.",
        Some(ctx.status_code.to_string()),
    ));
    out.push(check(
        "viewport",
        "technical",
        "warning",
        extras.viewport,
        2.0,
        if extras.viewport {
            "Viewport meta tag present"
        } else {
            "No viewport meta tag"
        },
        "Add <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">.",
        None,
    ));
    out.push(check(
        "favicon",
        "technical",
        "info",
        extras.favicon,
        1.0,
        if extras.favicon {
            "Favicon present"
        } else {
            "No favicon declared"
        },
        "Add a favicon via <link rel=\"icon\">.",
        None,
    ));
    out.push(check(
        "charset",
        "technical",
        "info",
        extras.charset,
        1.0,
        if extras.charset {
            "Charset declared"
        } else {
            "No charset declared"
        },
        "Declare <meta charset=\"utf-8\"> in the <head>.",
        None,
    ));
    out.push(check(
        "doctype",
        "technical",
        "info",
        extras.doctype,
        1.0,
        if extras.doctype {
            "HTML5 doctype present"
        } else {
            "Missing HTML5 doctype"
        },
        "Start the document with <!DOCTYPE html>.",
        None,
    ));
    out.push(check(
        "canonical_present",
        "technical",
        "warning",
        seo.canonical.is_some(),
        2.0,
        if seo.canonical.is_some() {
            "Canonical tag present"
        } else {
            "No canonical tag"
        },
        "Add <link rel=\"canonical\"> pointing to the page's preferred URL.",
        None,
    ));
    // Canonical self-reference (recommended pattern for canonical pages)
    let canonical_self = seo.canonical.as_deref().map(|c| {
        let trimmed = c.trim_end_matches('#');
        trimmed == ctx.url.trim_end_matches('#')
            || trimmed.trim_end_matches('/') == ctx.url.trim_end_matches('/')
    });
    out.push(check(
        "canonical_self_reference",
        "technical",
        "info",
        canonical_self.unwrap_or(false),
        1.0,
        match canonical_self {
            Some(true) => "Canonical points to the page itself".to_string(),
            Some(false) => "Canonical points to a different URL".to_string(),
            None => "No canonical to evaluate".to_string(),
        },
        "The canonical tag of a page that should rank should reference itself.",
        seo.canonical.clone(),
    ));
    let robots = extras.meta_robots.as_deref().or(seo.meta_robots.as_deref());
    let robots_nofollow = robots.map(|r| r.contains("nofollow")).unwrap_or(false);
    let robots_nosnippet = robots.map(|r| r.contains("nosnippet")).unwrap_or(false);
    out.push(check(
        "meta_robots_directives",
        "technical",
        "warning",
        !robots_nofollow && !robots_nosnippet,
        2.0,
        match (robots_nofollow, robots_nosnippet) {
            (true, true) => "robots meta blocks follow and snippet".to_string(),
            (true, false) => "robots meta sets nofollow".to_string(),
            (false, true) => "robots meta sets nosnippet".to_string(),
            (false, false) => "robots meta allows follow and snippet".to_string(),
        },
        "Remove nofollow/nosnippet from the robots meta unless the page must not be followed or shown in snippets.",
        robots.map(|r| r.to_string()),
    ));
    let noindex = seo
        .meta_robots
        .as_deref()
        .map(|r| r.contains("noindex"))
        .unwrap_or(false);
    out.push(check(
        "indexable",
        "technical",
        "warning",
        !noindex,
        2.0,
        if noindex {
            "Page is marked noindex"
        } else {
            "Page is indexable"
        },
        "Remove the noindex directive if this page should appear in search results.",
        None,
    ));
    out.push(check(
        "html_lang",
        "technical",
        "warning",
        seo.html_lang.is_some(),
        2.0,
        if seo.html_lang.is_some() {
            "HTML lang attribute present"
        } else {
            "Missing HTML lang attribute"
        },
        "Set the lang attribute on <html> (e.g. lang=\"en\").",
        None,
    ));
    let url_len = ctx.url.len();
    out.push(check(
        "url_length",
        "technical",
        "info",
        url_len <= 100,
        1.0,
        format!("URL length: {url_len} chars (≤ 100 recommended)"),
        "Shorten long, deeply nested URLs.",
        Some(url_len.to_string()),
    ));
    out.push(check(
        "url_underscores",
        "technical",
        "info",
        !extras.url_has_underscore,
        1.0,
        "URL path contains no underscores",
        "Use hyphens instead of underscores in URL paths.",
        None,
    ));

    // ==================== SOCIAL & OPEN GRAPH ====================
    let og = &seo.og_meta;
    out.push(check(
        "og_title",
        "social",
        "warning",
        og.og_title.is_some(),
        2.0,
        if og.og_title.is_some() {
            "og:title present"
        } else {
            "Missing og:title"
        },
        "Add <meta property=\"og:title\"> matching the page title.",
        None,
    ));
    out.push(check(
        "og_description",
        "social",
        "warning",
        og.og_description.is_some(),
        2.0,
        if og.og_description.is_some() {
            "og:description present"
        } else {
            "Missing og:description"
        },
        "Add a concise <meta property=\"og:description\">.",
        None,
    ));
    out.push(check(
        "og_image",
        "social",
        "warning",
        og.og_image.is_some(),
        2.0,
        if og.og_image.is_some() {
            "og:image present"
        } else {
            "Missing og:image"
        },
        "Add an <meta property=\"og:image\"> (1200×630 recommended).",
        None,
    ));
    out.push(check(
        "og_image_alt",
        "social",
        "info",
        og.og_image_alt.is_some(),
        1.0,
        if og.og_image_alt.is_some() {
            "og:image:alt present"
        } else {
            "Missing og:image:alt"
        },
        "Describe the Open Graph image with og:image:alt.",
        None,
    ));
    out.push(check(
        "og_url",
        "social",
        "info",
        og.og_url.is_some(),
        1.0,
        if og.og_url.is_some() {
            "og:url present"
        } else {
            "Missing og:url"
        },
        "Add og:url pointing to the canonical page URL.",
        None,
    ));
    out.push(check(
        "og_type",
        "social",
        "info",
        og.og_type.is_some(),
        1.0,
        if og.og_type.is_some() {
            "og:type present"
        } else {
            "Missing og:type"
        },
        "Add <meta property=\"og:type\"> (e.g. website or article).",
        None,
    ));
    out.push(check(
        "og_site_name",
        "social",
        "info",
        og.og_site_name.is_some(),
        1.0,
        if og.og_site_name.is_some() {
            "og:site_name present"
        } else {
            "Missing og:site_name"
        },
        "Add <meta property=\"og:site_name\"> with your brand name.",
        None,
    ));
    out.push(check(
        "twitter_card",
        "social",
        "warning",
        og.twitter_card.is_some(),
        2.0,
        if og.twitter_card.is_some() {
            "twitter:card present"
        } else {
            "Missing twitter:card"
        },
        "Add <meta name=\"twitter:card\" content=\"summary_large_image\">.",
        None,
    ));
    out.push(check(
        "twitter_title",
        "social",
        "info",
        og.twitter_title.is_some(),
        1.0,
        if og.twitter_title.is_some() {
            "twitter:title present"
        } else {
            "Missing twitter:title"
        },
        "Add a twitter:title meta tag.",
        None,
    ));
    out.push(check(
        "twitter_description",
        "social",
        "info",
        og.twitter_description.is_some(),
        1.0,
        if og.twitter_description.is_some() {
            "twitter:description present"
        } else {
            "Missing twitter:description"
        },
        "Add a twitter:description meta tag.",
        None,
    ));
    out.push(check(
        "twitter_image",
        "social",
        "info",
        og.twitter_image.is_some(),
        1.0,
        if og.twitter_image.is_some() {
            "twitter:image present"
        } else {
            "Missing twitter:image"
        },
        "Add a twitter:image meta tag.",
        None,
    ));

    // ==================== ACCESSIBILITY ====================
    out.push(check(
        "img_alt",
        "accessibility",
        "error",
        !has_issue(seo, &["img_no_alt"]),
        3.0,
        "All images have alt text",
        "Add descriptive alt attributes to every <img>.",
        None,
    ));
    out.push(check(
        "img_dimensions",
        "accessibility",
        "warning",
        extras.img_total == 0 || extras.img_with_dimensions == extras.img_total,
        2.0,
        format!(
            "Images with explicit dimensions: {}/{}",
            extras.img_with_dimensions, extras.img_total
        ),
        "Specify width/height on images to avoid layout shift.",
        Some(format!(
            "{}/{}",
            extras.img_with_dimensions, extras.img_total
        )),
    ));
    out.push(check(
        "form_labels",
        "accessibility",
        "error",
        !has_issue(seo, &["input_no_label"]),
        3.0,
        "All form inputs have labels",
        "Associate every input with a <label> via the for/id attributes.",
        None,
    ));
    out.push(check(
        "input_ids",
        "accessibility",
        "warning",
        !has_issue(seo, &["input_no_id"]),
        2.0,
        "Form controls have id attributes",
        "Give each input/select/textarea an id for label association.",
        None,
    ));
    out.push(check(
        "aria_controls",
        "accessibility",
        "warning",
        !has_issue(seo, &["missing_aria"]),
        2.0,
        "Form controls are accessible by ARIA",
        "Add aria-label or aria-labelledby to form controls lacking labels.",
        None,
    ));
    out.push(check(
        "empty_link_text",
        "accessibility",
        "warning",
        !has_issue(seo, &["empty_link_text"]),
        2.0,
        "All links have accessible text",
        "Give every link visible text or an aria-label.",
        None,
    ));
    out.push(check(
        "main_landmark",
        "accessibility",
        "warning",
        !has_issue(seo, &["missing_main"]),
        2.0,
        "Page has a <main> landmark",
        "Wrap the primary content in <main>.",
        None,
    ));
    out.push(check(
        "header_landmark",
        "accessibility",
        "info",
        !has_issue(seo, &["missing_header"]),
        1.0,
        "Page has a <header> landmark",
        "Add a <header> element for the page banner.",
        None,
    ));
    out.push(check(
        "footer_landmark",
        "accessibility",
        "info",
        !has_issue(seo, &["missing_footer"]),
        1.0,
        "Page has a <footer> landmark",
        "Add a <footer> element.",
        None,
    ));
    out.push(check(
        "nav_landmark",
        "accessibility",
        "info",
        !has_issue(seo, &["missing_nav"]),
        1.0,
        "Page has a <nav> landmark",
        "Wrap navigation links in a <nav> element.",
        None,
    ));
    out.push(check(
        "nesting_valid",
        "accessibility",
        "warning",
        !has_issue(seo, &["invalid_nesting", "context_nesting"]),
        2.0,
        "Element nesting follows HTML rules",
        "Fix invalid element nesting flagged by the semantic analysis.",
        None,
    ));
    out.push(check(
        "table_headers",
        "semantic_html",
        "error",
        extras.table_total == 0 || extras.table_with_headers == extras.table_total,
        2.0,
        format!(
            "Tables with header cells: {}/{}",
            extras.table_with_headers, extras.table_total
        ),
        "Mark header row and/or column cells with <th> so data cells have context.",
        Some(format!(
            "{}/{}",
            extras.table_with_headers, extras.table_total
        )),
    ));
    out.push(check(
        "table_captions",
        "semantic_html",
        "info",
        extras.table_total == 0 || extras.table_with_caption == extras.table_total,
        1.0,
        format!(
            "Tables with captions: {}/{}",
            extras.table_with_caption, extras.table_total
        ),
        "Add a <caption> to each table describing what it contains.",
        Some(format!(
            "{}/{}",
            extras.table_with_caption, extras.table_total
        )),
    ));
    out.push(check(
        "figure_captions",
        "semantic_html",
        "info",
        extras.figure_total == 0 || extras.figure_with_caption == extras.figure_total,
        1.0,
        format!(
            "Figures with captions: {}/{}",
            extras.figure_with_caption, extras.figure_total
        ),
        "Add a <figcaption> inside each <figure> to explain the visual.",
        Some(format!(
            "{}/{}",
            extras.figure_with_caption, extras.figure_total
        )),
    ));
    out.push(check(
        "iframe_titles",
        "semantic_html",
        "warning",
        extras.iframe_total == 0 || extras.iframe_with_title == extras.iframe_total,
        2.0,
        format!(
            "Iframes with title: {}/{}",
            extras.iframe_with_title, extras.iframe_total
        ),
        "Give every <iframe> a title describing the embedded content.",
        Some(format!(
            "{}/{}",
            extras.iframe_with_title, extras.iframe_total
        )),
    ));
    out.push(check(
        "video_accessible",
        "semantic_html",
        "warning",
        extras.video_total == 0 || extras.video_accessible == extras.video_total,
        2.0,
        format!(
            "Videos with controls/captions: {}/{}",
            extras.video_accessible, extras.video_total
        ),
        "Add controls (or a <track>) to each <video> so users can pause and caption content.",
        Some(format!(
            "{}/{}",
            extras.video_accessible, extras.video_total
        )),
    ));
    out.push(check(
        "autocomplete_inputs",
        "accessibility",
        "info",
        extras.input_total == 0 || extras.autocomplete_inputs >= 50,
        0.5,
        format!(
            "Form inputs with autocomplete: {}%",
            extras.autocomplete_inputs
        ),
        "Add an autocomplete attribute (e.g. email, name) to common form fields to speed up completion.",
        Some(format!("{}%", extras.autocomplete_inputs)),
    ));

    // ==================== PERFORMANCE ====================
    let size_kb = ctx.size_bytes as f64 / 1024.0;
    let ratio = if extras.html_bytes > 0 {
        extras.text_bytes as f64 / extras.html_bytes as f64
    } else {
        0.0
    };
    out.push(check(
        "content_to_html_ratio",
        "performance",
        "info",
        ratio >= 0.08,
        1.0,
        format!("Content-to-HTML ratio: {:.1}% (target ≥ 8%)", ratio * 100.0),
        "Reduce non-content markup (inline styles, wrappers) so text dominates the HTML.",
        Some(format!("{:.1}%", ratio * 100.0)),
    ));
    out.push(check(
        "page_weight",
        "performance",
        "warning",
        size_kb < 1536.0,
        2.0,
        format!("Page weight: {:.0} KB (limit ~1.5 MB)", size_kb),
        "Reduce page weight by minifying HTML, CSS and inline assets.",
        Some(format!("{:.0} KB", size_kb)),
    ));
    out.push(check(
        "load_time",
        "performance",
        "warning",
        ctx.load_time_ms < 2500,
        2.0,
        format!("Server load time: {} ms", ctx.load_time_ms),
        "Improve server response time (target under 2.5 s).",
        Some(format!("{} ms", ctx.load_time_ms)),
    ));
    out.push(check(
        "image_optimization",
        "performance",
        "warning",
        extras.img_total == 0 || extras.img_with_dimensions == extras.img_total,
        2.0,
        "Images declare dimensions for lazy layout",
        "Add width/height to images so browsers can reserve space.",
        None,
    ));
    out.push(check(
        "img_srcset",
        "performance",
        "info",
        extras.img_total == 0 || extras.img_srcset == extras.img_total,
        1.0,
        format!(
            "Images with srcset: {}/{}",
            extras.img_srcset, extras.img_total
        ),
        "Use srcset to serve responsive sizes for different viewports.",
        Some(format!("{}/{}", extras.img_srcset, extras.img_total)),
    ));
    out.push(check(
        "lazy_loading",
        "performance",
        "info",
        extras.img_total == 0 || extras.img_lazy >= extras.img_total.saturating_sub(1),
        1.0,
        format!(
            "Images with explicit loading: {}/{}",
            extras.img_lazy, extras.img_total
        ),
        "Declare loading=\"lazy\" (or eager) on images below the fold.",
        Some(format!("{}/{}", extras.img_lazy, extras.img_total)),
    ));
    out.push(check(
        "render_blocking_scripts",
        "performance",
        "warning",
        extras.scripts_render_blocking == 0,
        2.0,
        format!(
            "Render-blocking scripts: {}",
            extras.scripts_render_blocking
        ),
        "Load external scripts with async or defer so they do not block rendering.",
        Some(extras.scripts_render_blocking.to_string()),
    ));
    out.push(check(
        "resource_hints",
        "performance",
        "info",
        extras.preconnect_or_preload,
        1.0,
        "Resource hints (preconnect/preload) present",
        "Preconnect to critical origins and preload key resources.",
        None,
    ));

    // Optional PageSpeed (PSI) score merges into the performance category.
    if let Some(psi) = ctx.pagespeed_score {
        out.push(check(
            "pagespeed",
            "performance",
            "warning",
            psi >= 0.5,
            3.0,
            format!("PageSpeed (Lighthouse) performance: {:.0}/100", psi * 100.0),
            "Follow the Lighthouse performance audits to improve this score.",
            Some(format!("{:.0}/100", psi * 100.0)),
        ));
    }

    // ==================== AI READABILITY ====================
    let readability = seo.readability_score;
    out.push(check(
        "readability_score",
        "ai_readability",
        "warning",
        readability.map(|r| r >= 50.0).unwrap_or(false),
        2.0,
        match readability {
            Some(r) => format!("Flesch reading ease: {r:.0}/100 (≥ 50 recommended)"),
            None => "Not enough text to measure readability".to_string(),
        },
        "Simplify language and shorten sentences to raise readability.",
        readability.map(|r| format!("{r:.0}/100")),
    ));
    let fk_grade = readability.map(|r| (206.835 - r) / 7.2).unwrap_or(0.0);
    out.push(check(
        "flesch_kincaid_grade",
        "ai_readability",
        "info",
        fk_grade <= 12.0,
        1.0,
        format!("Flesch-Kincaid grade: {fk_grade:.1} (≤ 12 recommended)"),
        "Target a reading grade level most of your audience can read.",
        Some(format!("{fk_grade:.1}")),
    ));
    let avg_sentence = extras.word_count as f64 / extras.sentence_count.max(1) as f64;
    out.push(check(
        "sentence_length",
        "ai_readability",
        "info",
        avg_sentence <= 25.0,
        1.0,
        format!("Average sentence length: {avg_sentence:.1} words (≤ 25 recommended)"),
        "Break long sentences into shorter, single-idea sentences.",
        Some(format!("{avg_sentence:.1}")),
    ));
    out.push(check(
        "paragraph_structure",
        "ai_readability",
        "info",
        extras.p_count >= 3,
        1.0,
        format!("Paragraph count: {} (≥ 3 recommended)", extras.p_count),
        "Structure the content into short paragraphs and subheadings.",
        Some(extras.p_count.to_string()),
    ));
    let semantic_html = !has_issue(
        seo,
        &[
            "missing_main",
            "missing_header",
            "missing_footer",
            "missing_nav",
        ],
    );
    out.push(check(
        "semantic_html",
        "semantic_html",
        "info",
        semantic_html,
        1.0,
        "Semantic landmarks present (main/header/footer/nav)",
        "Use semantic HTML so AI and assistive tools understand the page structure.",
        None,
    ));
    out.push(check(
        "content_present",
        "ai_readability",
        "info",
        extras.word_count > 50,
        1.0,
        "Page has enough text to extract meaning",
        "Add substantive text content to the page.",
        None,
    ));

    // ==================== SXO / AEO / AIO ====================
    out.push(check(
        "structured_data",
        "sxo",
        "info",
        extras.has_json_ld,
        1.0,
        if extras.has_json_ld {
            "JSON-LD structured data present"
        } else {
            "No JSON-LD structured data"
        },
        "Add JSON-LD structured data describing the page's content.",
        None,
    ));
    out.push(check(
        "faq_schema",
        "sxo",
        "info",
        extras.json_ld_types.iter().any(|t| t == "FAQPage"),
        1.0,
        "FAQPage schema present",
        "Mark up questions and answers with FAQPage schema.",
        None,
    ));
    out.push(check(
        "howto_schema",
        "sxo",
        "info",
        extras.json_ld_types.iter().any(|t| t == "HowTo"),
        1.0,
        "HowTo schema present",
        "Mark up step-by-step instructions with HowTo schema.",
        None,
    ));
    out.push(check(
        "breadcrumb_schema",
        "sxo",
        "info",
        extras.json_ld_types.iter().any(|t| t == "BreadcrumbList"),
        1.0,
        "BreadcrumbList schema present",
        "Add BreadcrumbList schema for navigation paths.",
        None,
    ));
    out.push(check(
        "article_schema",
        "sxo",
        "info",
        extras
            .json_ld_types
            .iter()
            .any(|t| matches!(t.as_str(), "Article" | "NewsArticle" | "BlogPosting")),
        1.0,
        "Article schema present",
        "Mark up articles with Article/NewsArticle/BlogPosting schema.",
        None,
    ));
    out.push(check(
        "organization_schema",
        "sxo",
        "info",
        extras
            .json_ld_types
            .iter()
            .any(|t| matches!(t.as_str(), "Organization" | "Person" | "WebSite")),
        1.0,
        "Organization/WebSite schema present",
        "Add Organization or WebSite schema to define entity and brand.",
        None,
    ));
    out.push(check(
        "question_headings",
        "sxo",
        "info",
        extras.question_headings > 0,
        1.0,
        format!(
            "Question-style headings: {} (≥ 1 recommended)",
            extras.question_headings
        ),
        "Use headings phrased as questions users actually ask (e.g. 'How does…').",
        Some(extras.question_headings.to_string()),
    ));
    out.push(check(
        "direct_answer",
        "sxo",
        "info",
        extras.word_count >= 25,
        1.0,
        "Page can answer the query in its opening text",
        "Lead with a direct, concise answer to the main question.",
        None,
    ));
    out.push(check(
        "direct_answer_section",
        "sxo",
        "info",
        (40..=100).contains(&extras.answer_section_words),
        1.0,
        format!(
            "Answer-style section after H1: {} words (ideal 40-100)",
            extras.answer_section_words
        ),
        "Open with a focused 40-100 word passage that answers the main question directly.",
        Some(extras.answer_section_words.to_string()),
    ));
    out.push(check(
        "json_ld_valid",
        "sxo",
        "warning",
        !extras.has_json_ld || all_json_ld_valid(&extras.json_ld_raw),
        2.0,
        if extras.json_ld_raw.is_empty() {
            "No JSON-LD to validate".to_string()
        } else if all_json_ld_valid(&extras.json_ld_raw) {
            "JSON-LD blocks are structurally valid".to_string()
        } else {
            "At least one JSON-LD block is missing @context or @type".to_string()
        },
        "Every JSON-LD block needs @context and @type to be interpretable by Google and AI systems.",
        None,
    ));
    out.push(check(
        "schema_completeness",
        "sxo",
        "info",
        schema_completeness_score(&extras.json_ld_raw) >= 0.6,
        1.0,
        format!(
            "Schema completeness: {:.0}% of required properties present",
            schema_completeness_score(&extras.json_ld_raw) * 100.0
        ),
        "Add the required properties for each schema type (headline for Article, name for Person, etc.).",
        None,
    ));
    let has_faq_schema = extras.json_ld_types.iter().any(|t| t == "FAQPage");
    let faq_like_markup = extras.details_summary > 0 || extras.question_headings > 0;
    out.push(check(
        "faq_accordion_without_schema",
        "sxo",
        "info",
        !faq_like_markup || has_faq_schema,
        1.0,
        match (faq_like_markup, has_faq_schema) {
            (true, false) => "FAQ-like content without FAQPage schema".to_string(),
            (true, true) => "FAQ-like content with FAQPage schema".to_string(),
            (false, _) => "No FAQ-like content detected".to_string(),
        },
        "Mark up question/answer or accordion content with FAQPage schema so AI assistants can cite it.",
        Some(format!(
            "{} details/summary, {} question headings",
            extras.details_summary, extras.question_headings
        )),
    ));
    let has_article_schema = extras
        .json_ld_types
        .iter()
        .any(|t| matches!(t.as_str(), "Article" | "NewsArticle" | "BlogPosting"));
    let has_author = json_ld_has_author(&extras.json_ld_raw);
    out.push(check(
        "author_schema",
        "sxo",
        "info",
        !has_article_schema || has_author,
        1.0,
        match (has_article_schema, has_author) {
            (true, false) => "Article schema present but no author".to_string(),
            (true, true) => "Article schema declares an author".to_string(),
            (false, _) => "No article schema to check".to_string(),
        },
        "Declare author (Person) in article schema to strengthen E-E-A-T and entity attribution.",
        None,
    ));
    let has_freshness =
        json_ld_has_freshness(&extras.json_ld_raw) || extras.time_datetime_count > 0;
    out.push(check(
        "freshness_dates",
        "sxo",
        "info",
        has_freshness,
        1.0,
        if has_freshness {
            "Publish/modify dates present (schema or <time datetime>)".to_string()
        } else {
            "No freshness dates detected".to_string()
        },
        "Add datePublished/dateModified (or <time datetime>) so search and AI systems can trust content freshness.",
        None,
    ));

    // ==================== SECURITY (response headers) ====================
    let header = |name: &str| {
        ctx.response_headers
            .get(name)
            .map(|v| v.to_ascii_lowercase())
            .unwrap_or_default()
    };
    let has_header = |name: &str| ctx.response_headers.contains_key(name);

    let hsts = has_header("strict-transport-security");
    out.push(check(
        "hsts_header",
        "security",
        "warning",
        hsts,
        2.0,
        if hsts {
            "HTTP Strict Transport Security (HSTS) header present"
        } else {
            "Missing Strict-Transport-Security header"
        },
        "Add Strict-Transport-Security so browsers always use HTTPS for the domain.",
        Some(if hsts {
            "strict-transport-security present".to_string()
        } else {
            "strict-transport-security absent".to_string()
        }),
    ));

    let nosniff = header("x-content-type-options").contains("nosniff");
    out.push(check(
        "x_content_type_options",
        "security",
        "warning",
        nosniff,
        1.5,
        if nosniff {
            "X-Content-Type-Options: nosniff set"
        } else {
            "Missing X-Content-Type-Options: nosniff"
        },
        "Set X-Content-Type-Options: nosniff to stop MIME-sniffing attacks.",
        Some(header("x-content-type-options")),
    ));

    let csp = header("content-security-policy");
    let csp_present = !csp.is_empty();
    let xfo = has_header("x-frame-options") || csp.contains("frame-ancestors");
    out.push(check(
        "x_frame_options",
        "security",
        "warning",
        xfo,
        2.0,
        if xfo {
            "Clickjacking protection present (X-Frame-Options or CSP frame-ancestors)"
        } else {
            "No clickjacking protection (X-Frame-Options / frame-ancestors)"
        },
        "Deny framing with X-Frame-Options or a CSP frame-ancestors directive.",
        None,
    ));

    out.push(check(
        "content_security_policy",
        "security",
        "warning",
        csp_present,
        2.0,
        if csp_present {
            "Content-Security-Policy header present"
        } else {
            "Missing Content-Security-Policy header"
        },
        "Add a Content-Security-Policy header restricting scripts and origins.",
        Some(if csp.is_empty() {
            "content-security-policy absent".to_string()
        } else {
            format!("policy length: {} chars", csp.len())
        }),
    ));

    let referrer = has_header("referrer-policy");
    out.push(check(
        "referrer_policy",
        "security",
        "info",
        referrer,
        1.0,
        if referrer {
            "Referrer-Policy header present"
        } else {
            "Missing Referrer-Policy header"
        },
        "Set Referrer-Policy to control what URL data is shared in the Referer header.",
        None,
    ));

    let permissions = has_header("permissions-policy");
    out.push(check(
        "permissions_policy",
        "security",
        "info",
        permissions,
        1.0,
        if permissions {
            "Permissions-Policy header present"
        } else {
            "Missing Permissions-Policy header"
        },
        "Add Permissions-Policy to restrict access to sensitive browser features.",
        None,
    ));

    // ==================== COMPLIANCE (GDPR / privacy) ====================
    let has_privacy_schema = json_ld_has_privacy(&extras.json_ld_raw);
    let privacy_available = extras.privacy_link_count > 0 || has_privacy_schema;
    out.push(check(
        "privacy_policy_available",
        "compliance",
        "warning",
        privacy_available,
        2.0,
        if privacy_available {
            "Privacy policy reachable (link or schema)".to_string()
        } else {
            "No privacy policy link or schema found".to_string()
        },
        "Link a privacy policy (footer, header or schema.org privacyPolicy) so visitors and regulators can find it.",
        Some(format!(
            "{} privacy/legal links, privacyPolicy in schema: {}",
            extras.privacy_link_count, has_privacy_schema
        )),
    ));

    out.push(check(
        "cookie_consent_banner",
        "compliance",
        "warning",
        extras.consent_banner,
        2.0,
        if extras.consent_banner {
            "Cookie-consent banner or CMP detected"
        } else {
            "No cookie-consent banner / CMP detected"
        },
        "Provide an explicit consent mechanism (banner or CMP) for non-essential cookies where required.",
        None,
    ));

    out.push(check(
        "data_protection_schema",
        "compliance",
        "info",
        has_privacy_schema,
        1.0,
        if has_privacy_schema {
            "Data protection references in structured data"
        } else {
            "No privacyPolicy/policies in structured data"
        },
        "Declare privacyPolicy or policies on Organization/WebPage schema to make data handling machine-readable.",
        None,
    ));

    attach_examples(&mut out, seo);

    out
}

/// Maps a failed check to the semantic issue types that can explain it.
/// Order matters: the first mapped type wins when a check covers several.
const SEMANTIC_CHECK_MAP: &[(&str, &[&str])] = &[
    ("h1_count", &["multiple_h1"]),
    ("heading_hierarchy", &["heading_skip"]),
    ("img_alt", &["img_no_alt"]),
    ("img_dimensions", &["img_no_dimensions"]),
    ("image_optimization", &["img_no_dimensions"]),
    ("form_labels", &["input_no_label"]),
    ("input_ids", &["input_no_id"]),
    ("aria_controls", &["missing_aria"]),
    ("empty_link_text", &["empty_link_text"]),
    ("nesting_valid", &["invalid_nesting", "context_nesting"]),
    ("table_headers", &["table_without_headers"]),
    ("table_captions", &["table_without_caption"]),
    ("figure_captions", &["figure_without_caption"]),
    ("iframe_titles", &["iframe_without_title"]),
    ("video_accessible", &["video_without_track_or_controls"]),
    ("main_landmark", &["missing_main"]),
    ("header_landmark", &["missing_header"]),
    ("footer_landmark", &["missing_footer"]),
    ("nav_landmark", &["missing_nav"]),
    (
        "semantic_html",
        &[
            "missing_main",
            "missing_header",
            "missing_footer",
            "missing_nav",
        ],
    ),
];

/// Attach up to 5 concrete offending elements to failed checks so the UI can
/// point at the exact elements that must be fixed (xpath, css_selector, snippet).
fn attach_examples(out: &mut [CheckResult], seo: &SeoData) {
    for check in out.iter_mut() {
        if check.passed {
            continue;
        }
        let Some(issue_types) = SEMANTIC_CHECK_MAP
            .iter()
            .find(|(id, _)| *id == check.id)
            .map(|(_, types)| *types)
        else {
            continue;
        };
        let mut examples: Vec<SemanticIssue> = seo
            .semantic_issues
            .iter()
            .filter(|i| issue_types.contains(&i.issue_type.as_str()) && i.xpath.is_some())
            .cloned()
            .collect();
        examples.sort_by_key(|e| severity_rank(&e.severity));
        examples.truncate(5);
        check.examples = examples;
    }
}

fn severity_rank(severity: &str) -> u8 {
    match severity {
        "error" => 0,
        "warning" => 1,
        _ => 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(url: &str) -> AuditContext {
        AuditContext {
            url: url.to_string(),
            status_code: 200,
            size_bytes: 4096,
            load_time_ms: 120,
            pagespeed_score: None,
            response_headers: Default::default(),
        }
    }

    fn run(html: &str, url: &str) -> Vec<CheckResult> {
        let extras = PageExtras::extract(html, url);
        let parsed = Url::parse(url).ok();
        let parser = crate::crawler::parser::SeoParser::new();
        let (seo, _) = parser.parse(html, &parsed.expect("valid url"));
        run_all(&seo, &extras, &ctx(url))
    }

    fn run_default(html: &str, url: &str) -> Vec<CheckResult> {
        let extras = PageExtras::extract(html, url);
        run_all(&SeoData::default(), &extras, &ctx(url))
    }

    fn check<'a>(out: &'a [CheckResult], id: &str) -> &'a CheckResult {
        out.iter()
            .find(|c| c.id == id)
            .unwrap_or_else(|| panic!("check {id} missing"))
    }

    #[test]
    fn test_extract_counts_semantic_elements() {
        let html = r#"<!DOCTYPE html>
<html lang="en">
<head><title>T</title></head>
<body>
    <main>
        <h1>Title</h1>
        <p>First paragraph with enough words to make a small direct answer section for extraction.</p>
        <table><caption>Data</caption><tr><th>H</th></tr></table>
        <table><tr><td>x</td></tr></table>
        <figure><img src="a.png" alt="a"><figcaption>Cap</figcaption></figure>
        <figure><img src="b.png" alt="b"></figure>
        <details><summary>Q</summary>Answer</details>
        <video src="v.mp4"></video>
        <iframe src="https://example.com"></iframe>
    </main>
</body>
</html>"#;
        let extras = PageExtras::extract(html, "https://example.com/page");
        assert_eq!(extras.table_total, 2);
        assert_eq!(extras.table_with_headers, 1);
        assert_eq!(extras.table_with_caption, 1);
        assert_eq!(extras.figure_total, 2);
        assert_eq!(extras.figure_with_caption, 1);
        assert_eq!(extras.details_summary, 1);
        assert_eq!(extras.video_total, 1);
        assert_eq!(extras.video_accessible, 0);
        assert_eq!(extras.iframe_total, 1);
        assert_eq!(extras.iframe_with_title, 0);
    }

    #[test]
    fn test_internal_external_ratio_and_direct_answer() {
        let links = (0..12)
            .map(|i| format!("<a href=\"https://example.com/page/{i}\">link {i}</a>"))
            .collect::<String>();
        let html = format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head><title>T</title></head>
<body>
    <main>
        <h1>Title</h1>
        <p>This opening paragraph answers the main question directly in a compact passage of roughly forty to one hundred words so that AI answer engines can extract it quickly.</p>
        <p>A second short paragraph adds a little more context about the topic of this page.</p>
        {links}
        <a href="https://other.com/x">external</a>
    </main>
</body>
</html>"#
        );
        let out = run(&html, "https://example.com/page");
        assert!(check(&out, "internal_external_ratio").passed);
        assert!(check(&out, "direct_answer_section").passed);
    }

    #[test]
    fn test_canonical_self_reference() {
        let html = r#"<!DOCTYPE html>
<html lang="en">
<head><title>T</title><link rel="canonical" href="https://example.com/page"></head>
<body><main><h1>T</h1><p>Content.</p></main></body>
</html>"#;
        let out = run(html, "https://example.com/page");
        assert!(check(&out, "canonical_self_reference").passed);

        let out = run(html, "https://example.com/other");
        assert!(!check(&out, "canonical_self_reference").passed);
    }

    #[test]
    fn test_robots_directives_and_freshness() {
        let html = r#"<!DOCTYPE html>
<html lang="en">
<head><title>T</title><meta name="robots" content="nofollow, nosnippet"></head>
<body><main><h1>T</h1><p>Content.</p></main></body>
</html>"#;
        let out = run(html, "https://example.com/page");
        assert!(!check(&out, "meta_robots_directives").passed);

        let with_time = r#"<!DOCTYPE html>
<html lang="en">
<head><title>T</title></head>
<body><main><h1>T</h1><p>Content.</p><time datetime="2026-08-11">Aug</time></main></body>
</html>"#;
        let out = run(with_time, "https://example.com/page");
        assert!(check(&out, "freshness_dates").passed);
    }

    #[test]
    fn test_json_ld_validation_and_completeness() {
        let valid = r#"{
            "@context": "https://schema.org",
            "@type": "Article",
            "headline": "A headline",
            "datePublished": "2026-08-11",
            "author": {"@type": "Person", "name": "Jane"}
        }"#;
        let invalid = r#"{"name": "no context or type"}"#;

        let html = |ld: &str| {
            format!(
                r#"<!DOCTYPE html><html lang="en"><head><title>T</title><script type="application/ld+json">{ld}</script></head><body><main><h1>T</h1><p>Content.</p></main></body></html>"#
            )
        };

        let out = run(&html(valid), "https://example.com/page");
        assert!(check(&out, "json_ld_valid").passed);
        assert!(check(&out, "schema_completeness").passed);
        assert!(check(&out, "author_schema").passed);

        let out = run(&html(invalid), "https://example.com/page");
        assert!(!check(&out, "json_ld_valid").passed);
        assert!(check(&out, "author_schema").passed);

        // Article without author fails author_schema
        let no_author = r#"{
            "@context": "https://schema.org",
            "@type": "Article",
            "headline": "A headline"
        }"#;
        let out = run(&html(no_author), "https://example.com/page");
        assert!(!check(&out, "author_schema").passed);
    }

    #[test]
    fn test_faq_accordion_without_schema_and_rendering_checks() {
        let html = r#"<!DOCTYPE html>
<html lang="en">
<head><title>T</title></head>
<body>
    <main>
        <h1>T</h1>
        <details><summary>Question?</summary>Answer text.</details>
        <img src="a.png" alt="a" width="10" height="10" srcset="a.png 1x" loading="lazy">
        <script src="/app.js"></script>
    </main>
</body>
</html>"#;
        let out = run(html, "https://example.com/page");
        assert!(!check(&out, "faq_accordion_without_schema").passed);
        assert!(check(&out, "img_srcset").passed);
        assert!(check(&out, "lazy_loading").passed);
        assert!(!check(&out, "render_blocking_scripts").passed);
    }

    #[test]
    fn test_security_headers_checks() {
        let html = "<!DOCTYPE html><html lang=\"en\"><head><title>T</title></head><body></body></html>";
        let extras = PageExtras::extract(html, "https://example.com/page");
        let parser = crate::crawler::parser::SeoParser::new();
        let (seo, _) = parser.parse(html, &Url::parse("https://example.com/page").unwrap());

        let mut headers = std::collections::HashMap::new();
        headers.insert("strict-transport-security".to_string(), "max-age=63072000".to_string());
        headers.insert("x-content-type-options".to_string(), "nosniff".to_string());
        headers.insert("x-frame-options".to_string(), "DENY".to_string());
        headers.insert(
            "content-security-policy".to_string(),
            "default-src 'self'; frame-ancestors 'none'".to_string(),
        );
        let secure = AuditContext {
            url: "https://example.com/page".to_string(),
            status_code: 200,
            size_bytes: 4096,
            load_time_ms: 120,
            pagespeed_score: None,
            response_headers: headers,
        };
        let out = run_all(&seo, &extras, &secure);
        for id in [
            "hsts_header",
            "x_content_type_options",
            "x_frame_options",
            "content_security_policy",
        ] {
            assert!(check(&out, id).passed, "{id} should pass");
        }
        assert!(!check(&out, "referrer_policy").passed);
        assert!(!check(&out, "permissions_policy").passed);

        let bare_ctx = ctx("https://example.com/page");
        let out = run_all(&seo, &extras, &bare_ctx);
        assert!(!check(&out, "hsts_header").passed);
        assert!(!check(&out, "x_frame_options").passed);
        assert!(!check(&out, "content_security_policy").passed);
    }

    #[test]
    fn test_compliance_checks_detect_privacy_and_consent() {
        let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <title>T</title>
    <script type="application/ld+json">{
        "@context": "https://schema.org",
        "@type": "Organization",
        "name": "Acme",
        "privacyPolicy": "https://example.com/privacy"
    }</script>
</head>
<body>
    <footer>
        <a href="/privacy">Privacy policy</a>
        <a href="/terms">Terms</a>
    </footer>
    <div id="cookie-banner"><button>Accept cookies</button></div>
</body>
</html>"#;
        let out = run(html, "https://example.com/page");
        assert!(check(&out, "privacy_policy_available").passed);
        assert!(check(&out, "cookie_consent_banner").passed);
        assert!(check(&out, "data_protection_schema").passed);

        let bare = r#"<!DOCTYPE html><html lang="en"><head><title>T</title></head><body></body></html>"#;
        let out = run(bare, "https://example.com/page");
        assert!(!check(&out, "privacy_policy_available").passed);
        assert!(!check(&out, "cookie_consent_banner").passed);
        assert!(!check(&out, "data_protection_schema").passed);
    }
}
