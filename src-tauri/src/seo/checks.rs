use scraper::{Html, Selector};
use url::Url;

use crate::crawler::parser::{SeoData, SemanticIssue};

use super::audit::{AuditContext, CheckResult};

/// Extra page signals extracted from the raw HTML (a second, light DOM pass).
#[derive(Debug, Clone, Default)]
pub struct PageExtras {
    pub json_ld_types: Vec<String>,
    pub has_json_ld: bool,
    pub viewport: bool,
    pub favicon: bool,
    pub charset: bool,
    pub doctype: bool,
    pub img_total: usize,
    pub img_with_dimensions: usize,
    pub preconnect_or_preload: bool,
    pub p_count: usize,
    pub word_count: usize,
    pub sentence_count: usize,
    pub question_headings: usize,
    pub url_has_underscore: bool,
}

impl PageExtras {
    #[allow(clippy::field_reassign_with_default)]
    pub fn extract(html: &str, url: &str) -> Self {
        let document = Html::parse_document(html);
        let mut extras = PageExtras::default();

        // Doctype
        extras.doctype = html.trim_start().to_ascii_lowercase().starts_with("<!doctype");

        // Charset / viewport
        extras.charset = has_selector(&document, "meta[charset]");
        extras.viewport = has_selector(&document, r#"meta[name="viewport"]"#);

        // Favicon
        extras.favicon =
            has_selector(&document, r#"link[rel~="icon"], link[rel="shortcut icon"]"#);

        // Resource hints
        extras.preconnect_or_preload =
            has_selector(&document, r#"link[rel="preconnect"], link[rel="preload"]"#);

        // Images with explicit dimensions
        if let Ok(sel) = Selector::parse("img") {
            let imgs: Vec<_> = document.select(&sel).collect();
            extras.img_total = imgs.len();
            extras.img_with_dimensions = imgs
                .iter()
                .filter(|el| {
                    el.value().attr("width").is_some() && el.value().attr("height").is_some()
                })
                .count();
        }

        // Paragraph count
        extras.p_count = count_selector(&document, "p");

        // JSON-LD structured data
        if let Ok(sel) = Selector::parse(r#"script[type="application/ld+json"]"#) {
            for script in document.select(&sel) {
                let text = script.text().collect::<Vec<_>>().join("");
                if let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) {
                    extras.has_json_ld = true;
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

        // URL heuristics
        if let Ok(parsed) = Url::parse(url) {
            extras.url_has_underscore = parsed.path().contains('_');
        }

        extras
    }
}

const QUESTION_WORDS: &[&str] = &[
    "what", "how", "why", "who", "where", "when", "which", "can", "could", "does", "do", "is",
    "are", "should",
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
        if title_present { "Page has a title" } else { "Page has no <title>" },
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
    let desc_len = seo.meta_description.as_deref().map(|d| d.chars().count()).unwrap_or(0);
    out.push(check(
        "meta_description_present",
        "meta",
        "warning",
        desc_present,
        2.0,
        if desc_present { "Page has a meta description" } else { "Page has no meta description" },
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
        if h1_present { "Page has one H1" } else { "Page has no <h1>" },
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

    let origin = Url::parse(&ctx.url)
        .ok()
        .and_then(|u| u.host_str().map(|h| (u.scheme().to_string(), h.to_string())));
    let mut has_internal = false;
    let mut has_outbound = false;
    for link in &seo.outgoing_links {
        if !link.url.starts_with("http://") && !link.url.starts_with("https://") {
            continue;
        }
        if let Ok(parsed) = Url::parse(&link.url) {
            let same = match &origin {
                Some((scheme, host)) => parsed.scheme() == scheme && parsed.host_str() == Some(host),
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

    // ==================== TECHNICAL & MOBILE ====================
    let https = ctx.url.starts_with("https://");
    out.push(check(
        "https_used",
        "technical",
        "warning",
        https,
        2.0,
        if https { "Served over HTTPS" } else { "Not served over HTTPS" },
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
        if extras.viewport { "Viewport meta tag present" } else { "No viewport meta tag" },
        "Add <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">.",
        None,
    ));
    out.push(check(
        "favicon",
        "technical",
        "info",
        extras.favicon,
        1.0,
        if extras.favicon { "Favicon present" } else { "No favicon declared" },
        "Add a favicon via <link rel=\"icon\">.",
        None,
    ));
    out.push(check(
        "charset",
        "technical",
        "info",
        extras.charset,
        1.0,
        if extras.charset { "Charset declared" } else { "No charset declared" },
        "Declare <meta charset=\"utf-8\"> in the <head>.",
        None,
    ));
    out.push(check(
        "doctype",
        "technical",
        "info",
        extras.doctype,
        1.0,
        if extras.doctype { "HTML5 doctype present" } else { "Missing HTML5 doctype" },
        "Start the document with <!DOCTYPE html>.",
        None,
    ));
    out.push(check(
        "canonical_present",
        "technical",
        "warning",
        seo.canonical.is_some(),
        2.0,
        if seo.canonical.is_some() { "Canonical tag present" } else { "No canonical tag" },
        "Add <link rel=\"canonical\"> pointing to the page's preferred URL.",
        None,
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
        if noindex { "Page is marked noindex" } else { "Page is indexable" },
        "Remove the noindex directive if this page should appear in search results.",
        None,
    ));
    out.push(check(
        "html_lang",
        "technical",
        "warning",
        seo.html_lang.is_some(),
        2.0,
        if seo.html_lang.is_some() { "HTML lang attribute present" } else { "Missing HTML lang attribute" },
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
        if og.og_title.is_some() { "og:title present" } else { "Missing og:title" },
        "Add <meta property=\"og:title\"> matching the page title.",
        None,
    ));
    out.push(check(
        "og_description",
        "social",
        "warning",
        og.og_description.is_some(),
        2.0,
        if og.og_description.is_some() { "og:description present" } else { "Missing og:description" },
        "Add a concise <meta property=\"og:description\">.",
        None,
    ));
    out.push(check(
        "og_image",
        "social",
        "warning",
        og.og_image.is_some(),
        2.0,
        if og.og_image.is_some() { "og:image present" } else { "Missing og:image" },
        "Add an <meta property=\"og:image\"> (1200×630 recommended).",
        None,
    ));
    out.push(check(
        "og_image_alt",
        "social",
        "info",
        og.og_image_alt.is_some(),
        1.0,
        if og.og_image_alt.is_some() { "og:image:alt present" } else { "Missing og:image:alt" },
        "Describe the Open Graph image with og:image:alt.",
        None,
    ));
    out.push(check(
        "og_url",
        "social",
        "info",
        og.og_url.is_some(),
        1.0,
        if og.og_url.is_some() { "og:url present" } else { "Missing og:url" },
        "Add og:url pointing to the canonical page URL.",
        None,
    ));
    out.push(check(
        "og_type",
        "social",
        "info",
        og.og_type.is_some(),
        1.0,
        if og.og_type.is_some() { "og:type present" } else { "Missing og:type" },
        "Add <meta property=\"og:type\"> (e.g. website or article).",
        None,
    ));
    out.push(check(
        "og_site_name",
        "social",
        "info",
        og.og_site_name.is_some(),
        1.0,
        if og.og_site_name.is_some() { "og:site_name present" } else { "Missing og:site_name" },
        "Add <meta property=\"og:site_name\"> with your brand name.",
        None,
    ));
    out.push(check(
        "twitter_card",
        "social",
        "warning",
        og.twitter_card.is_some(),
        2.0,
        if og.twitter_card.is_some() { "twitter:card present" } else { "Missing twitter:card" },
        "Add <meta name=\"twitter:card\" content=\"summary_large_image\">.",
        None,
    ));
    out.push(check(
        "twitter_title",
        "social",
        "info",
        og.twitter_title.is_some(),
        1.0,
        if og.twitter_title.is_some() { "twitter:title present" } else { "Missing twitter:title" },
        "Add a twitter:title meta tag.",
        None,
    ));
    out.push(check(
        "twitter_description",
        "social",
        "info",
        og.twitter_description.is_some(),
        1.0,
        if og.twitter_description.is_some() { "twitter:description present" } else { "Missing twitter:description" },
        "Add a twitter:description meta tag.",
        None,
    ));
    out.push(check(
        "twitter_image",
        "social",
        "info",
        og.twitter_image.is_some(),
        1.0,
        if og.twitter_image.is_some() { "twitter:image present" } else { "Missing twitter:image" },
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
        format!("Images with explicit dimensions: {}/{}", extras.img_with_dimensions, extras.img_total),
        "Specify width/height on images to avoid layout shift.",
        Some(format!("{}/{}", extras.img_with_dimensions, extras.img_total)),
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

    // ==================== PERFORMANCE ====================
    let size_kb = ctx.size_bytes as f64 / 1024.0;
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
    let semantic_html =
        !has_issue(seo, &["missing_main", "missing_header", "missing_footer", "missing_nav"]);
    out.push(check(
        "semantic_html",
        "ai_readability",
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
        if extras.has_json_ld { "JSON-LD structured data present" } else { "No JSON-LD structured data" },
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
        format!("Question-style headings: {} (≥ 1 recommended)", extras.question_headings),
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

    attach_examples(&mut out, seo);

    out
}

/// Maps a failed check to the semantic issue types that can explain it.
/// Order matters: the first mapped type wins when a check covers several.
const SEMANTIC_CHECK_MAP: &[(&str, &[&str])] = &[
    ("h1_count", &["multiple_h1"]),
    ("heading_hierarchy", &["heading_skip"]),
    ("img_alt", &["img_no_alt"]),
    ("form_labels", &["input_no_label"]),
    ("input_ids", &["input_no_id"]),
    ("aria_controls", &["missing_aria"]),
    ("empty_link_text", &["empty_link_text"]),
    ("nesting_valid", &["invalid_nesting", "context_nesting"]),
    ("main_landmark", &["missing_main"]),
    ("header_landmark", &["missing_header"]),
    ("footer_landmark", &["missing_footer"]),
    ("nav_landmark", &["missing_nav"]),
    ("semantic_html", &["missing_main", "missing_header", "missing_footer", "missing_nav"]),
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
