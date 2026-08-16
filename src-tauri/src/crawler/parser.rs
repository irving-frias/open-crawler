use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::nesting_table::{can_include, NestingStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HreflangLink {
    pub lang: String,
    pub href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SemanticIssue {
    pub issue_type: String,
    pub severity: String,
    pub element: String,
    pub message: String,
    pub selector: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xpath: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub css_selector: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snippet: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<u32>,
}

#[derive(Debug, Clone, Default)]
pub struct SeoData {
    pub title: Option<String>,
    pub meta_description: Option<String>,
    pub meta_robots: Option<String>,
    pub canonical: Option<String>,
    pub h1: Option<String>,
    pub h2: Vec<String>,
    pub img_alts: Vec<String>,
    pub outgoing_links: Vec<OutgoingLink>,
    pub html_lang: Option<String>,
    pub hreflang_links: Vec<HreflangLink>,
    pub semantic_issues: Vec<SemanticIssue>,
    pub readability_score: Option<f64>,
    pub content_hash: Option<String>,
    pub keywords: Vec<Keyword>,
    pub og_meta: OgMeta,
}

#[derive(Debug, Clone)]
pub struct OutgoingLink {
    pub url: String,
    pub anchor_text: String,
    pub rel_tokens: Vec<String>,
    pub is_follow: bool,
    pub is_sponsored: bool,
    pub is_ugc: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keyword {
    pub keyword: String,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OgMeta {
    pub og_title: Option<String>,
    pub og_description: Option<String>,
    pub og_image: Option<String>,
    pub og_image_alt: Option<String>,
    pub og_type: Option<String>,
    pub og_url: Option<String>,
    pub og_site_name: Option<String>,
    pub og_locale: Option<String>,
    pub twitter_card: Option<String>,
    pub twitter_title: Option<String>,
    pub twitter_description: Option<String>,
    pub twitter_image: Option<String>,
}

impl OgMeta {
    pub fn is_empty(&self) -> bool {
        self.og_title.is_none()
            && self.og_description.is_none()
            && self.og_image.is_none()
            && self.og_image_alt.is_none()
            && self.og_type.is_none()
            && self.og_url.is_none()
            && self.og_site_name.is_none()
            && self.og_locale.is_none()
            && self.twitter_card.is_none()
            && self.twitter_title.is_none()
            && self.twitter_description.is_none()
            && self.twitter_image.is_none()
    }
}

/// Compute XPath for a scraper ElementRef
/// Build an absolute XPath path from /html root to the element.
/// Uses `ElementRef::wrap` to walk the DOM tree (same pattern as `compute_css_selector`).
fn build_absolute_xpath(el: &scraper::ElementRef) -> String {
    let tag = el.value().name().to_string();

    if let Some(id) = el.value().id() {
        return format!("/html/body//*[@id='{}']", id);
    }

    let position = el
        .parent()
        .map(|parent| {
            parent
                .children()
                .filter_map(|c| c.value().as_element())
                .filter(|e| e.name() == el.value().name())
                .take_while(|e| *e != el.value())
                .count()
                + 1
        })
        .unwrap_or(1);

    let current = if position > 1 {
        format!("{}[{}]", tag, position)
    } else {
        tag
    };

    match el.parent() {
        Some(parent) if parent.value().is_element() => match scraper::ElementRef::wrap(parent) {
            Some(pe) => {
                let parent_path = build_absolute_xpath(&pe);
                format!("{}/{}", parent_path, current)
            }
            None => current,
        },
        _ => format!("/html/body/{}", current),
    }
}

fn collect_upward(el: &scraper::ElementRef, out: &mut Vec<String>) {
    if let Some(id) = el.value().id() {
        out.push(format!("//*[@id='{}']", id));
        return;
    }

    let tag = el.value().name().to_string();

    let position = el
        .parent()
        .map(|parent| {
            parent
                .children()
                .filter_map(|c| c.value().as_element())
                .filter(|e| e.name() == el.value().name())
                .take_while(|e| *e != el.value())
                .count()
                + 1
        })
        .unwrap_or(1);

    let step = if position > 1 {
        format!("{}[{}]", tag, position)
    } else {
        tag
    };

    out.push(step);

    if let Some(parent) = el.parent().and_then(scraper::ElementRef::wrap) {
        collect_upward(&parent, out);
    }
}

fn build_accessible_xpath(el: &scraper::ElementRef) -> String {
    let mut parts: Vec<String> = Vec::new();
    collect_upward(el, &mut parts);

    if !parts.is_empty() && parts.last().map(|s| s.starts_with("//*[@id='")) == Some(true) {
        parts.reverse();
        if parts.len() > 4 {
            parts.truncate(4);
        }
        parts.join("/")
    } else {
        build_absolute_xpath(el)
    }
}

pub fn compute_xpath(element: &scraper::ElementRef) -> String {
    build_accessible_xpath(element)
}

/// Full absolute XPath: always /html/body/...
pub fn compute_xpath_full(element: &scraper::ElementRef) -> String {
    build_absolute_xpath(element)
}

/// Short XPath: //tag[@id='x'] if has id, otherwise relative path from parent
pub fn compute_xpath_short(element: &scraper::ElementRef) -> String {
    let tag = element.value().name().to_string();

    if let Some(id) = element.value().id() {
        return format!("//*[@id='{}']", id);
    }

    let position = element
        .parent()
        .map(|parent| {
            parent
                .children()
                .filter_map(|c| c.value().as_element())
                .filter(|e| e.name() == element.value().name())
                .take_while(|e| *e != element.value())
                .count()
                + 1
        })
        .unwrap_or(1);

    let current = if position > 1 {
        format!("{}[{}]", tag, position)
    } else {
        tag.clone()
    };

    if tag == "html" || tag == "body" || tag == "head" {
        format!("/{}", current)
    } else {
        format!("//{}/{}", tag, current)
    }
}

/// Compute a unique CSS selector for a scraper ElementRef, building full path from html
pub fn compute_css_selector(element: &scraper::ElementRef) -> String {
    fn build_path(el: &scraper::ElementRef, depth: usize) -> String {
        let tag = el.value().name().to_string();

        // If has id, that's unique enough — stop here
        if let Some(id) = el.value().id() {
            return format!("{}#{}", tag, id);
        }

        // Count siblings with same tag for nth-of-type
        let position = el
            .parent()
            .map(|parent| {
                parent
                    .children()
                    .filter_map(|c| c.value().as_element())
                    .filter(|e| e.name() == el.value().name())
                    .take_while(|e| *e != el.value())
                    .count()
                    + 1
            })
            .unwrap_or(1);

        let classes: Vec<&str> = el.value().classes().collect();

        let current = if !classes.is_empty() {
            let class_str = classes.join(".");
            if position > 1 {
                format!("{}.{}:nth-of-type({})", tag, class_str, position)
            } else {
                format!("{}.{}", tag, class_str)
            }
        } else if position > 1 {
            format!("{}:nth-of-type({})", tag, position)
        } else {
            tag.clone()
        };

        // Walk up to parent element
        match el.parent() {
            Some(parent) if parent.value().is_element() => {
                match scraper::ElementRef::wrap(parent) {
                    Some(pe) => {
                        let parent_path = build_path(&pe, depth + 1);
                        if depth == 0 {
                            current
                        } else {
                            format!("{} > {}", parent_path, current)
                        }
                    }
                    None => current,
                }
            }
            _ => {
                if depth == 0 {
                    format!("html > {}", current)
                } else {
                    current
                }
            }
        }
    }

    let mut path = build_path(element, 0);
    // Ensure it starts with html
    if !path.starts_with("html") {
        path = format!("html > body > {}", path);
    }
    path
}

/// Extract a text snippet from an element (first 200 chars)
pub fn compute_snippet(element: &scraper::ElementRef) -> String {
    let html = element.html();
    if html.len() > 200 {
        format!("{}...", super::truncate_bytes(&html, 200))
    } else {
        html
    }
}

/// Build a SemanticIssue with optional element context (xpath, css_selector, snippet)
pub fn issue(
    issue_type: &str,
    severity: &str,
    element: &str,
    message: &str,
    selector: Option<String>,
    element_ref: Option<scraper::ElementRef>,
) -> SemanticIssue {
    let (xpath, css_selector, snippet) = match element_ref {
        Some(el) => (
            Some(compute_xpath(&el)),
            Some(compute_css_selector(&el)),
            Some(compute_snippet(&el)),
        ),
        None => (None, None, None),
    };

    SemanticIssue {
        issue_type: issue_type.to_string(),
        severity: severity.to_string(),
        element: element.to_string(),
        message: message.to_string(),
        selector,
        xpath,
        css_selector,
        snippet,
        line: None,
        column: None,
    }
}

#[derive(Debug, Clone)]
pub struct SeoParser;

/// Cap the number of per-element issues emitted per issue type so one page cannot
/// flood the results (e.g. a gallery with 500 images without alt).
const MAX_ELEMENT_ISSUES_PER_TYPE: usize = 25;

impl Default for SeoParser {
    fn default() -> Self {
        Self::new()
    }
}

impl SeoParser {
    pub fn new() -> Self {
        Self
    }

    pub fn parse(&self, html: &str, base_url: &Url) -> (SeoData, Vec<String>) {
        let document = Html::parse_document(html);

        let title = self.extract_title(&document);
        let meta_description = self.extract_meta_description(&document);
        let meta_robots = self.extract_meta_robots(&document);
        let canonical = self.extract_canonical(&document);
        let h1 = self.extract_h1(&document);
        let h2 = self.extract_h2(&document);
        let img_alts = self.extract_img_alts(&document);
        let outgoing_links = self.extract_links(&document, base_url);
        let html_lang = self.extract_html_lang(&document);
        let hreflang_links = self.extract_hreflang(&document, base_url);
        let semantic_issues = self.analyze_semantics(&document);
        let visible_text = self.extract_visible_text(&document);
        let readability_score = self.compute_readability(&visible_text);
        let content_hash = self.compute_content_hash(&visible_text);
        let keywords = self.extract_keywords(&visible_text);
        let og_meta = self.extract_og_meta(&document);

        let outgoing_urls: Vec<String> = outgoing_links.iter().map(|l| l.url.clone()).collect();

        (
            SeoData {
                title,
                meta_description,
                meta_robots,
                canonical,
                h1,
                h2,
                img_alts,
                outgoing_links,
                html_lang,
                hreflang_links,
                semantic_issues,
                readability_score,
                content_hash,
                keywords,
                og_meta,
            },
            outgoing_urls,
        )
    }

    // ==================== SEMANTIC ANALYSIS ====================

    pub fn analyze_semantics(&self, document: &Html) -> Vec<SemanticIssue> {
        let mut issues = Vec::new();

        // 1. Missing <html lang>
        if self.extract_html_lang(document).is_none() {
            issues.push(SemanticIssue {
                issue_type: "missing_html_lang".to_string(),
                severity: "error".to_string(),
                element: "<html>".to_string(),
                message: "Missing lang attribute on <html> element".to_string(),
                selector: Some("html".to_string()),
                ..Default::default()
            });
        }

        // 2. Missing <title>
        if self.extract_title(document).is_none() {
            issues.push(SemanticIssue {
                issue_type: "missing_title".to_string(),
                severity: "error".to_string(),
                element: "<title>".to_string(),
                message: "Missing <title> tag".to_string(),
                selector: Some("title".to_string()),
                ..Default::default()
            });
        }

        // 3. Missing meta description
        if self.extract_meta_description(document).is_none() {
            issues.push(SemanticIssue {
                issue_type: "missing_meta_description".to_string(),
                severity: "warning".to_string(),
                element: "<meta>".to_string(),
                message: "Missing meta description".to_string(),
                selector: Some(r#"meta[name="description"]"#.to_string()),
                ..Default::default()
            });
        }

        // 4. Missing canonical
        if self.extract_canonical(document).is_none() {
            issues.push(SemanticIssue {
                issue_type: "missing_canonical".to_string(),
                severity: "warning".to_string(),
                element: "<link>".to_string(),
                message: "Missing canonical link".to_string(),
                selector: Some(r#"link[rel="canonical"]"#.to_string()),
                ..Default::default()
            });
        }

        // 5. Missing <main>
        if !self.has_element(document, "main") {
            issues.push(SemanticIssue {
                issue_type: "missing_main".to_string(),
                severity: "warning".to_string(),
                element: "<main>".to_string(),
                message: "Page has no <main> element".to_string(),
                selector: Some("main".to_string()),
                ..Default::default()
            });
        }

        // 6. Missing <header>
        if !self.has_element(document, "header") {
            issues.push(SemanticIssue {
                issue_type: "missing_header".to_string(),
                severity: "info".to_string(),
                element: "<header>".to_string(),
                message: "Page has no <header> element".to_string(),
                selector: Some("header".to_string()),
                ..Default::default()
            });
        }

        // 7. Missing <footer>
        if !self.has_element(document, "footer") {
            issues.push(SemanticIssue {
                issue_type: "missing_footer".to_string(),
                severity: "info".to_string(),
                element: "<footer>".to_string(),
                message: "Page has no <footer> element".to_string(),
                selector: Some("footer".to_string()),
                ..Default::default()
            });
        }

        // 8. Missing <nav> (only if links exist)
        let link_count = self.count_elements(document, "a[href]");
        if link_count > 0 && !self.has_element(document, "nav") {
            issues.push(SemanticIssue {
                issue_type: "missing_nav".to_string(),
                severity: "info".to_string(),
                element: "<nav>".to_string(),
                message: "Page has links but no <nav> element".to_string(),
                selector: Some("nav".to_string()),
                ..Default::default()
            });
        }

        // 9. Missing <h1>
        if self.extract_h1(document).is_none() {
            issues.push(SemanticIssue {
                issue_type: "missing_h1".to_string(),
                severity: "warning".to_string(),
                element: "<h1>".to_string(),
                message: "Missing <h1> tag".to_string(),
                selector: Some("h1".to_string()),
                ..Default::default()
            });
        }

        // 10. Multiple <h1> (one issue per extra <h1>)
        let h1_count = self.count_elements(document, "h1");
        if h1_count > 1 {
            let h1s = self.iter_elements(document, "h1");
            for h1 in h1s.into_iter().skip(1).take(MAX_ELEMENT_ISSUES_PER_TYPE) {
                issues.push(issue(
                    "multiple_h1",
                    "warning",
                    "<h1>",
                    &format!(
                        "Page has {} <h1> elements; only one is recommended",
                        h1_count
                    ),
                    Some("h1".to_string()),
                    Some(h1),
                ));
            }
        }

        // 11. Heading hierarchy skip
        if let Some(skip_issue) = self.check_heading_hierarchy(document) {
            issues.push(skip_issue);
        }

        // 12. <img> without alt (one issue per image)
        let imgs_selector = r#"img:not([alt]), img[alt=""]"#;
        let img_no_alt_count = self.count_elements(document, imgs_selector);
        if img_no_alt_count > 0 {
            for img in self
                .iter_elements(document, imgs_selector)
                .into_iter()
                .take(MAX_ELEMENT_ISSUES_PER_TYPE)
            {
                issues.push(issue(
                    "img_no_alt",
                    "error",
                    "<img>",
                    "Image is missing the alt attribute",
                    Some(imgs_selector.to_string()),
                    Some(img),
                ));
            }
        }

        // 12b. <img> without explicit width/height (one issue per image).
        // These cause layout shift (CLS) and are surfaced by the
        // `img_dimensions` / `image_optimization` SEO checks.
        let imgs_dims_selector = r#"img:not([width]), img:not([height])"#;
        let img_no_dims_count = self.count_elements(document, imgs_dims_selector);
        if img_no_dims_count > 0 {
            for img in self
                .iter_elements(document, imgs_dims_selector)
                .into_iter()
                .take(MAX_ELEMENT_ISSUES_PER_TYPE)
            {
                issues.push(issue(
                    "img_no_dimensions",
                    "warning",
                    "<img>",
                    "Image is missing explicit width and height",
                    Some(imgs_dims_selector.to_string()),
                    Some(img),
                ));
            }
        }

        // 13. <input>/<textarea>/<select> without id (one issue per element)
        let no_id_selector = "input:not([id]), textarea:not([id]), select:not([id])";
        let inputs_no_id = self.count_elements(document, no_id_selector);
        if inputs_no_id > 0 {
            for el in self
                .iter_elements(document, no_id_selector)
                .into_iter()
                .take(MAX_ELEMENT_ISSUES_PER_TYPE)
            {
                issues.push(issue(
                    "input_no_id",
                    "warning",
                    &format!("<{}>", el.value().name()),
                    "Form element is missing the id attribute",
                    Some(no_id_selector.to_string()),
                    Some(el),
                ));
            }
        }

        // 14. Form controls without an associated <label> (one issue per control)
        let inputs_no_label = self.inputs_without_label(document);
        if !inputs_no_label.is_empty() {
            for el in inputs_no_label
                .into_iter()
                .take(MAX_ELEMENT_ISSUES_PER_TYPE)
            {
                issues.push(issue(
                    "input_no_label",
                    "error",
                    &format!("<{}>", el.value().name()),
                    "Form control has no associated <label>",
                    Some("input".to_string()),
                    Some(el),
                ));
            }
        }

        // 15. Empty link text (one issue per link)
        let empty_links = self.empty_links(document);
        if !empty_links.is_empty() {
            for link in empty_links.into_iter().take(MAX_ELEMENT_ISSUES_PER_TYPE) {
                issues.push(issue(
                    "empty_link_text",
                    "warning",
                    "<a>",
                    "Link has no text and no aria-label",
                    Some("a[href]".to_string()),
                    Some(link),
                ));
            }
        }

        // 16. Missing ARIA on form controls (one issue per control)
        let missing_aria = self.form_controls_without_aria(document);
        if !missing_aria.is_empty() {
            for el in missing_aria.into_iter().take(MAX_ELEMENT_ISSUES_PER_TYPE) {
                issues.push(issue(
                    "missing_aria",
                    "warning",
                    &format!("<{}>", el.value().name()),
                    "Form control is missing aria-label or aria-labelledby",
                    Some("select, textarea".to_string()),
                    Some(el),
                ));
            }
        }

        // 17. Invalid element nesting (flow inside phrasing)
        let nesting_issues = self.check_element_nesting(document);
        issues.extend(nesting_issues);

        // 18. Multiple <main> elements (one issue per extra <main>)
        let main_count = self.count_elements(document, "main");
        if main_count > 1 {
            let mains = self.iter_elements(document, "main");
            for main in mains.into_iter().skip(1).take(MAX_ELEMENT_ISSUES_PER_TYPE) {
                issues.push(issue(
                    "multiple_main",
                    "error",
                    "<main>",
                    &format!(
                        "Page has {} <main> elements; only one is recommended",
                        main_count
                    ),
                    Some("main".to_string()),
                    Some(main),
                ));
            }
        }

        // 19. Missing skip link (only meaningful when a <main> landmark exists)
        if self.count_elements(document, "main") > 0 && !self.skip_link_present(document) {
            issues.push(SemanticIssue {
                issue_type: "skip_link_missing".to_string(),
                severity: "info".to_string(),
                element: "<a>".to_string(),
                message: "Page has no skip link pointing to the main content".to_string(),
                selector: Some(r##"a[href^="#"]"##.to_string()),
                ..Default::default()
            });
        }

        // 20. <section> without an accessible name (not a usable landmark)
        let sections = self.sections_without_accessible_name(document);
        if !sections.is_empty() {
            for el in sections.into_iter().take(MAX_ELEMENT_ISSUES_PER_TYPE) {
                issues.push(issue(
                    "section_no_accessible_name",
                    "warning",
                    "<section>",
                    "<section> has no accessible name",
                    Some("section".to_string()),
                    Some(el),
                ));
            }
        }

        // 21. <form> without an accessible name
        let forms = self.forms_without_accessible_name(document);
        if !forms.is_empty() {
            for el in forms.into_iter().take(MAX_ELEMENT_ISSUES_PER_TYPE) {
                issues.push(issue(
                    "form_no_accessible_name",
                    "warning",
                    "<form>",
                    "<form> has no accessible name",
                    Some("form".to_string()),
                    Some(el),
                ));
            }
        }

        // 22. <button> without an accessible name
        let buttons = self.buttons_without_name(document);
        if !buttons.is_empty() {
            for el in buttons.into_iter().take(MAX_ELEMENT_ISSUES_PER_TYPE) {
                issues.push(issue(
                    "button_no_name",
                    "warning",
                    "<button>",
                    "<button> has no text and no accessible name",
                    Some("button".to_string()),
                    Some(el),
                ));
            }
        }

        // 23. Duplicate id attributes (one issue per duplicated element)
        let duplicates = self.duplicate_ids(document);
        if !duplicates.is_empty() {
            for (id, el) in duplicates.into_iter().take(MAX_ELEMENT_ISSUES_PER_TYPE) {
                issues.push(issue(
                    "duplicate_id",
                    "error",
                    &format!("<{} id=\"...\">", el.value().name()),
                    &format!("Duplicate id attribute \"{}\"", id),
                    Some("[id]".to_string()),
                    Some(el),
                ));
            }
        }

        // 24. Navigation without an aria-current indicator
        if self.count_elements(document, "nav") > 0 && !self.nav_has_aria_current(document) {
            issues.push(SemanticIssue {
                issue_type: "aria_current_nav".to_string(),
                severity: "info".to_string(),
                element: "<nav>".to_string(),
                message: "Navigation has no aria-current indicator for the active page".to_string(),
                selector: Some("nav".to_string()),
                ..Default::default()
            });
        }

        // 25. Substantial <main> content without an <article> (citable content)
        if self.count_elements(document, "article") == 0 && self.main_has_substantial_text(document)
        {
            issues.push(SemanticIssue {
                issue_type: "article_missing".to_string(),
                severity: "warning".to_string(),
                element: "<article>".to_string(),
                message: "Page has substantial content but no <article> element".to_string(),
                selector: Some("article".to_string()),
                ..Default::default()
            });
        }

        // 26. <time> without datetime (freshness signal for AI/AEO extraction)
        let times_no_datetime = self.iter_elements(document, "time:not([datetime])");
        for el in times_no_datetime
            .into_iter()
            .take(MAX_ELEMENT_ISSUES_PER_TYPE)
        {
            issues.push(issue(
                "time_without_datetime",
                "warning",
                "<time>",
                "<time> is missing the datetime attribute",
                Some("time".to_string()),
                Some(el),
            ));
        }

        // 27. <figure> without <figcaption>
        for el in self
            .figures_without_caption(document)
            .into_iter()
            .take(MAX_ELEMENT_ISSUES_PER_TYPE)
        {
            issues.push(issue(
                "figure_without_caption",
                "info",
                "<figure>",
                "<figure> has no <figcaption>",
                Some("figure".to_string()),
                Some(el),
            ));
        }

        // 28. <table> without <th> header cells
        for el in self
            .tables_without_headers(document)
            .into_iter()
            .take(MAX_ELEMENT_ISSUES_PER_TYPE)
        {
            issues.push(issue(
                "table_without_headers",
                "error",
                "<table>",
                "<table> has no <th> header cells",
                Some("table".to_string()),
                Some(el),
            ));
        }

        // 29. <table> without <caption>
        for el in self
            .tables_without_caption(document)
            .into_iter()
            .take(MAX_ELEMENT_ISSUES_PER_TYPE)
        {
            issues.push(issue(
                "table_without_caption",
                "info",
                "<table>",
                "<table> has no <caption>",
                Some("table".to_string()),
                Some(el),
            ));
        }

        // 30. <blockquote> without attribution (cite or named source)
        for el in self
            .blockquotes_without_attribution(document)
            .into_iter()
            .take(MAX_ELEMENT_ISSUES_PER_TYPE)
        {
            issues.push(issue(
                "blockquote_without_attribution",
                "warning",
                "<blockquote>",
                "<blockquote> has no attribution (cite or author)",
                Some("blockquote".to_string()),
                Some(el),
            ));
        }

        // 31. Substantial text directly inside a <div> instead of <p>
        for el in self
            .divs_with_direct_text(document, 20)
            .into_iter()
            .take(MAX_ELEMENT_ISSUES_PER_TYPE)
        {
            issues.push(issue(
                "text_in_div_instead_of_p",
                "warning",
                "<div>",
                "<div> contains substantial text; use <p> for body copy",
                Some("div".to_string()),
                Some(el),
            ));
        }

        // 32. <iframe> without title
        for el in self
            .iter_elements(document, "iframe:not([title])")
            .into_iter()
            .take(MAX_ELEMENT_ISSUES_PER_TYPE)
        {
            issues.push(issue(
                "iframe_without_title",
                "warning",
                "<iframe>",
                "<iframe> is missing the title attribute",
                Some("iframe".to_string()),
                Some(el),
            ));
        }

        // 33. <video> without controls or <track>
        for el in self
            .videos_without_controls(document)
            .into_iter()
            .take(MAX_ELEMENT_ISSUES_PER_TYPE)
        {
            issues.push(issue(
                "video_without_track_or_controls",
                "warning",
                "<video>",
                "<video> has no controls attribute and no <track>",
                Some("video".to_string()),
                Some(el),
            ));
        }

        // 34. Empty headings (h1-h6): break the heading outline and the
        //     passage-extraction signals used by search and AI tools.
        let empty_heading_tags = ["h1", "h2", "h3", "h4", "h5", "h6"];
        for el in self
            .empty_elements(document, &empty_heading_tags)
            .into_iter()
            .take(MAX_ELEMENT_ISSUES_PER_TYPE)
        {
            issues.push(issue(
                "empty_heading",
                "warning",
                &format!("<{}>", el.value().name()),
                &format!("<{}> has no content", el.value().name()),
                Some(empty_heading_tags.join(", ")),
                Some(el),
            ));
        }

        // 35. Empty <p>: dead body-copy markup that wastes layout space.
        for el in self
            .empty_elements(document, &["p"])
            .into_iter()
            .take(MAX_ELEMENT_ISSUES_PER_TYPE)
        {
            issues.push(issue(
                "empty_paragraph",
                "warning",
                "<p>",
                "<p> has no content",
                Some("p".to_string()),
                Some(el),
            ));
        }

        // 36. Empty content tags (div/span/li/section/...): dead markup usually
        //     left by broken templates or failed hydration.
        const EMPTY_CONTENT_TAGS: &[&str] = &[
            "div",
            "span",
            "li",
            "section",
            "article",
            "blockquote",
            "label",
            "td",
            "th",
            "header",
            "footer",
            "nav",
            "main",
            "aside",
            "figure",
            "caption",
            "summary",
        ];
        for el in self
            .empty_elements(document, EMPTY_CONTENT_TAGS)
            .into_iter()
            .take(MAX_ELEMENT_ISSUES_PER_TYPE)
        {
            issues.push(issue(
                "empty_content_tag",
                "info",
                &format!("<{}>", el.value().name()),
                &format!("<{}> has no content", el.value().name()),
                Some(EMPTY_CONTENT_TAGS.join(", ")),
                Some(el),
            ));
        }

        issues
    }

    /// Elements matching any of `tags` that contain neither element children nor
    /// non-whitespace text. Elements carrying identity/styling/hiding attributes
    /// (id, class, style, hidden, aria-hidden) are skipped because they are
    /// plausibly intentional (JS mount points, icon fonts, CSS spacers).
    fn empty_elements<'a>(
        &self,
        document: &'a Html,
        tags: &[&str],
    ) -> Vec<scraper::ElementRef<'a>> {
        let Ok(selector) = Selector::parse(&tags.join(", ")) else {
            return Vec::new();
        };
        document
            .select(&selector)
            .filter(|el| {
                let value = el.value();
                if value.id().is_some()
                    || value.attr("class").is_some()
                    || value.attr("style").is_some()
                    || value.attr("hidden").is_some()
                    || value.attr("aria-hidden").is_some()
                {
                    return false;
                }
                el.children().all(|child| match child.value() {
                    scraper::node::Node::Text(t) => t.trim().is_empty(),
                    _ => false,
                })
            })
            .collect()
    }

    fn main_has_substantial_text(&self, document: &Html) -> bool {
        let selector = match Selector::parse("main") {
            Ok(s) => s,
            Err(_) => return false,
        };
        let Some(main) = document.select(&selector).next() else {
            return false;
        };
        let mut parts: Vec<String> = Vec::new();
        collect_visible_text(&main, &mut parts);
        parts.join(" ").split_whitespace().count() >= 50
    }

    fn figures_without_caption<'a>(&self, document: &'a Html) -> Vec<scraper::ElementRef<'a>> {
        let caption = match Selector::parse("figcaption") {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };
        self.iter_elements(document, "figure")
            .into_iter()
            .filter(|el| el.select(&caption).next().is_none())
            .collect()
    }

    fn tables_without_headers<'a>(&self, document: &'a Html) -> Vec<scraper::ElementRef<'a>> {
        let th = match Selector::parse("th") {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };
        self.iter_elements(document, "table")
            .into_iter()
            .filter(|el| el.select(&th).next().is_none())
            .collect()
    }

    fn tables_without_caption<'a>(&self, document: &'a Html) -> Vec<scraper::ElementRef<'a>> {
        let caption = match Selector::parse("caption") {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };
        self.iter_elements(document, "table")
            .into_iter()
            .filter(|el| el.select(&caption).next().is_none())
            .collect()
    }

    fn blockquotes_without_attribution<'a>(
        &self,
        document: &'a Html,
    ) -> Vec<scraper::ElementRef<'a>> {
        let inner = match Selector::parse("footer, figcaption, cite") {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };
        self.iter_elements(document, "blockquote")
            .into_iter()
            .filter(|el| el.value().attr("cite").is_none() && el.select(&inner).next().is_none())
            .collect()
    }

    fn videos_without_controls<'a>(&self, document: &'a Html) -> Vec<scraper::ElementRef<'a>> {
        let track = match Selector::parse("track") {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };
        self.iter_elements(document, "video")
            .into_iter()
            .filter(|el| {
                el.value().attr("controls").is_none() && el.select(&track).next().is_none()
            })
            .collect()
    }

    fn divs_with_direct_text<'a>(
        &self,
        document: &'a Html,
        min_words: usize,
    ) -> Vec<scraper::ElementRef<'a>> {
        self.iter_elements(document, "div")
            .into_iter()
            .filter(|el| self.direct_text_words(*el) >= min_words)
            .collect()
    }

    fn direct_text_words(&self, el: scraper::ElementRef) -> usize {
        el.children()
            .filter_map(|child| match child.value() {
                scraper::node::Node::Text(t) => Some(t.split_whitespace().count()),
                _ => None,
            })
            .sum()
    }

    fn has_element(&self, document: &Html, selector_str: &str) -> bool {
        let selector = match Selector::parse(selector_str) {
            Ok(s) => s,
            Err(_) => return false,
        };
        document.select(&selector).next().is_some()
    }

    fn count_elements(&self, document: &Html, selector_str: &str) -> usize {
        let selector = match Selector::parse(selector_str) {
            Ok(s) => s,
            Err(_) => return 0,
        };
        document.select(&selector).count()
    }

    fn iter_elements<'a>(
        &self,
        document: &'a Html,
        selector_str: &str,
    ) -> Vec<scraper::ElementRef<'a>> {
        match Selector::parse(selector_str) {
            Ok(s) => document.select(&s).collect(),
            Err(_) => Vec::new(),
        }
    }

    fn check_heading_hierarchy(&self, document: &Html) -> Option<SemanticIssue> {
        let selectors: Vec<(&str, u8)> = vec![
            ("h1", 1),
            ("h2", 2),
            ("h3", 3),
            ("h4", 4),
            ("h5", 5),
            ("h6", 6),
        ];

        let mut last_level: Option<u8> = None;

        for (sel_str, level) in selectors {
            let selector = match Selector::parse(sel_str) {
                Ok(s) => s,
                Err(_) => continue,
            };
            let count = document.select(&selector).count();
            if count > 0 {
                if let Some(prev) = last_level {
                    if level > prev + 1 {
                        return Some(SemanticIssue {
                            issue_type: "heading_skip".to_string(),
                            severity: "info".to_string(),
                            element: format!("<h{}>", level),
                            message: format!("Heading level skips from h{} to h{}", prev, level),
                            selector: Some(sel_str.to_string()),
                            ..Default::default()
                        });
                    }
                }
                last_level = Some(level);
            }
        }

        None
    }

    fn inputs_without_label<'a>(&self, document: &'a Html) -> Vec<scraper::ElementRef<'a>> {
        let selector = match Selector::parse(
            "input:not([type='hidden']):not([type='submit']):not([type='button'])",
        ) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        // Rule (equivalent of //input[(@id and not(//label[@for=current()/@id])) or
        // (not(@id) and not(parent::label))]): an input is valid only if it has an id
        // with an associated <label for="id">, or it has no id but its direct parent
        // is a <label>.
        let mut result = Vec::new();
        for input in document.select(&selector) {
            match input.value().attr("id") {
                Some(input_id) => {
                    let label_selector_str = format!("label[for='{}']", input_id);
                    let has_label = Selector::parse(&label_selector_str)
                        .ok()
                        .map(|s| document.select(&s).next().is_some())
                        .unwrap_or(false);
                    if has_label {
                        continue;
                    }
                }
                None => {
                    if self.is_direct_child_of_label(input) {
                        continue;
                    }
                }
            }
            result.push(input);
        }

        result
    }

    fn is_direct_child_of_label(&self, input: scraper::ElementRef) -> bool {
        input
            .parent()
            .and_then(scraper::ElementRef::wrap)
            .map(|parent| parent.value().name() == "label")
            .unwrap_or(false)
    }

    fn empty_links<'a>(&self, document: &'a Html) -> Vec<scraper::ElementRef<'a>> {
        let selector = match Selector::parse("a[href]") {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        let mut result = Vec::new();
        for link in document.select(&selector) {
            let has_aria_label = link.value().attr("aria-label").is_some();
            let has_aria_labelledby = link.value().attr("aria-labelledby").is_some();
            let has_title = link.value().attr("title").is_some();
            let text = link.text().collect::<Vec<_>>().join("").trim().to_string();
            let has_img_alt = {
                if let Ok(img_selector) = Selector::parse("img[alt]") {
                    link.select(&img_selector).next().is_some()
                } else {
                    false
                }
            };

            if text.is_empty()
                && !has_aria_label
                && !has_aria_labelledby
                && !has_title
                && !has_img_alt
            {
                result.push(link);
            }
        }

        result
    }

    fn form_controls_without_aria<'a>(&self, document: &'a Html) -> Vec<scraper::ElementRef<'a>> {
        let selector = match Selector::parse("select, textarea") {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        let mut result = Vec::new();
        for el in document.select(&selector) {
            let has_aria_label = el.value().attr("aria-label").is_some();
            let has_aria_labelledby = el.value().attr("aria-labelledby").is_some();
            let has_title = el.value().attr("title").is_some();
            let id = el.value().attr("id");

            if has_aria_label || has_aria_labelledby || has_title {
                continue;
            }

            if let Some(el_id) = id {
                let has_label = {
                    let label_selector_str = format!("label[for='{}']", el_id);
                    Selector::parse(&label_selector_str)
                        .ok()
                        .map(|s| document.select(&s).next().is_some())
                        .unwrap_or(false)
                };
                if has_label {
                    continue;
                }
            }

            result.push(el);
        }

        result
    }

    fn skip_link_present(&self, document: &Html) -> bool {
        let selector = match Selector::parse("a[href^='#']") {
            Ok(s) => s,
            Err(_) => return false,
        };
        for link in document.select(&selector) {
            let text = link.text().collect::<Vec<_>>().join(" ").to_lowercase();
            let label = link.value().attr("aria-label").unwrap_or("").to_lowercase();
            let class = link.value().attr("class").unwrap_or("").to_lowercase();
            if text.contains("skip") || label.contains("skip") || class.contains("skip") {
                return true;
            }
        }
        false
    }

    fn sections_without_accessible_name<'a>(
        &self,
        document: &'a Html,
    ) -> Vec<scraper::ElementRef<'a>> {
        let selector = match Selector::parse("section") {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };
        let heading = Selector::parse("h1, h2, h3, h4, h5, h6").unwrap();
        let mut result = Vec::new();
        for section in document.select(&selector) {
            let has_aria = section.value().attr("aria-label").is_some()
                || section.value().attr("aria-labelledby").is_some();
            let has_heading = section.select(&heading).next().is_some();
            if !has_aria && !has_heading {
                result.push(section);
            }
        }
        result
    }

    fn forms_without_accessible_name<'a>(
        &self,
        document: &'a Html,
    ) -> Vec<scraper::ElementRef<'a>> {
        let selector = match Selector::parse("form") {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };
        let mut result = Vec::new();
        for form in document.select(&selector) {
            let has_aria = form.value().attr("aria-label").is_some()
                || form.value().attr("aria-labelledby").is_some()
                || form.value().attr("title").is_some();
            if !has_aria {
                result.push(form);
            }
        }
        result
    }

    fn buttons_without_name<'a>(&self, document: &'a Html) -> Vec<scraper::ElementRef<'a>> {
        let selector = match Selector::parse("button") {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };
        let mut result = Vec::new();
        for button in document.select(&selector) {
            let has_aria = button.value().attr("aria-label").is_some()
                || button.value().attr("aria-labelledby").is_some()
                || button.value().attr("title").is_some();
            let text = button
                .text()
                .collect::<Vec<_>>()
                .join("")
                .trim()
                .to_string();
            if !has_aria && text.is_empty() {
                result.push(button);
            }
        }
        result
    }

    fn duplicate_ids<'a>(&self, document: &'a Html) -> Vec<(String, scraper::ElementRef<'a>)> {
        let selector = match Selector::parse("[id]") {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };
        let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut result = Vec::new();
        for el in document.select(&selector) {
            let id = el.value().attr("id").unwrap_or("");
            if id.is_empty() {
                continue;
            }
            if !seen.insert(id.to_string()) {
                result.push((id.to_string(), el));
            }
        }
        result
    }

    fn nav_has_aria_current(&self, document: &Html) -> bool {
        self.count_elements(document, "[aria-current]") > 0
    }

    /// Check for invalid element nesting using the static caninclude table.
    /// Reports "cant" as error, "doubt" as info (context-dependent nesting).
    fn check_element_nesting(&self, document: &Html) -> Vec<SemanticIssue> {
        let mut issues = Vec::new();
        let mut seen_pairs: std::collections::HashSet<(String, String)> =
            std::collections::HashSet::new();

        let all_selector = Selector::parse("*").unwrap();
        for element in document.select(&all_selector) {
            let child_tag = element.value().name();

            if let Some(parent_ref) = self.find_parent_element(element) {
                let parent_tag = parent_ref.value().name();
                let pair = (child_tag.to_string(), parent_tag.to_string());

                if seen_pairs.contains(&pair) {
                    continue;
                }

                match can_include(parent_tag, child_tag) {
                    Some(NestingStatus::Cant) => {
                        seen_pairs.insert(pair.clone());
                        let selector = compute_css_selector(&element);
                        let xpath = compute_xpath(&element);
                        issues.push(SemanticIssue {
                            issue_type: "invalid_nesting".to_string(),
                            severity: "error".to_string(),
                            element: format!("<{}> in <{}>", child_tag, parent_tag),
                            message: format!(
                                "<{}> cannot be a child of <{}>",
                                child_tag, parent_tag
                            ),
                            selector: Some(selector),
                            xpath: Some(xpath),
                            ..Default::default()
                        });
                    }
                    Some(NestingStatus::Doubt) => {
                        seen_pairs.insert(pair.clone());
                        let selector = compute_css_selector(&element);
                        let xpath = compute_xpath(&element);
                        issues.push(SemanticIssue {
                            issue_type: "context_nesting".to_string(),
                            severity: "info".to_string(),
                            element: format!("<{}> in <{}>", child_tag, parent_tag),
                            message: format!(
                                "<{}> nesting in <{}> depends on context",
                                child_tag, parent_tag
                            ),
                            selector: Some(selector),
                            xpath: Some(xpath),
                            ..Default::default()
                        });
                    }
                    _ => {} // Can or unknown
                }
            }
        }

        issues
    }

    /// Find the nearest ancestor element (not text nodes or comments)
    fn find_parent_element<'a>(
        &self,
        element: scraper::ElementRef<'a>,
    ) -> Option<scraper::ElementRef<'a>> {
        let mut current = element.parent();
        while let Some(node) = current {
            if let Some(el) = scraper::ElementRef::wrap(node) {
                return Some(el);
            }
            current = node.parent();
        }
        None
    }

    // ==================== SEO EXTRACTION ====================

    fn extract_title(&self, document: &Html) -> Option<String> {
        let selector = Selector::parse("title").ok()?;
        document
            .select(&selector)
            .next()
            .map(|el| el.text().collect::<Vec<_>>().join("").trim().to_string())
            .filter(|s| !s.is_empty())
    }

    fn extract_meta_description(&self, document: &Html) -> Option<String> {
        let selector = Selector::parse(r#"meta[name="description"]"#).ok()?;
        document
            .select(&selector)
            .next()
            .and_then(|el| el.value().attr("content"))
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }

    fn extract_meta_robots(&self, document: &Html) -> Option<String> {
        let selector = Selector::parse(r#"meta[name="robots"]"#).ok()?;
        document
            .select(&selector)
            .next()
            .and_then(|el| el.value().attr("content"))
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }

    fn extract_canonical(&self, document: &Html) -> Option<String> {
        let selector = Selector::parse(r#"link[rel="canonical"]"#).ok()?;
        document
            .select(&selector)
            .next()
            .and_then(|el| el.value().attr("href"))
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }

    fn extract_h1(&self, document: &Html) -> Option<String> {
        let selector = Selector::parse("h1").ok()?;
        document
            .select(&selector)
            .next()
            .map(|el| el.text().collect::<Vec<_>>().join("").trim().to_string())
            .filter(|s| !s.is_empty())
    }

    fn extract_h2(&self, document: &Html) -> Vec<String> {
        let selector = match Selector::parse("h2") {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };
        document
            .select(&selector)
            .filter_map(|el| {
                let text = el.text().collect::<Vec<_>>().join("").trim().to_string();
                if text.is_empty() {
                    None
                } else {
                    Some(text)
                }
            })
            .collect()
    }

    fn extract_img_alts(&self, document: &Html) -> Vec<String> {
        let selector = match Selector::parse("img") {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };
        document
            .select(&selector)
            .filter_map(|el| el.value().attr("alt"))
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    fn extract_links(&self, document: &Html, base_url: &Url) -> Vec<OutgoingLink> {
        let selector = match Selector::parse("a[href]") {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        document
            .select(&selector)
            .filter_map(|el| {
                let href = el.value().attr("href")?;
                let anchor_text = el.text().collect::<Vec<_>>().join("").trim().to_string();

                let absolute_url = if href.starts_with("http://") || href.starts_with("https://") {
                    href.to_string()
                } else {
                    base_url.join(href).ok()?.to_string()
                };

                let rel = el.value().attr("rel").unwrap_or("");
                let rel_tokens: Vec<String> = rel
                    .split_whitespace()
                    .map(|t| t.to_ascii_lowercase())
                    .collect();
                let is_follow = !rel_tokens.iter().any(|t| t == "nofollow");
                let is_sponsored = rel_tokens.iter().any(|t| t == "sponsored");
                let is_ugc = rel_tokens.iter().any(|t| t == "ugc");

                Some(OutgoingLink {
                    url: absolute_url,
                    anchor_text,
                    rel_tokens,
                    is_follow,
                    is_sponsored,
                    is_ugc,
                })
            })
            .collect()
    }

    fn extract_html_lang(&self, document: &Html) -> Option<String> {
        let selector = Selector::parse("html").ok()?;
        document
            .select(&selector)
            .next()
            .and_then(|el| el.value().attr("lang"))
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }

    fn extract_hreflang(&self, document: &Html, base_url: &Url) -> Vec<HreflangLink> {
        let selector = match Selector::parse(r#"link[rel="alternate"][hreflang]"#) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        document
            .select(&selector)
            .filter_map(|el| {
                let lang = el.value().attr("hreflang")?.to_string();
                let href = el.value().attr("href")?;

                let absolute_url = if href.starts_with("http://") || href.starts_with("https://") {
                    href.to_string()
                } else {
                    base_url.join(href).ok()?.to_string()
                };

                Some(HreflangLink {
                    lang,
                    href: absolute_url,
                })
            })
            .collect()
    }

    // ==================== READABILITY, KEYWORDS, SIMHASH, SOCIAL META ====================

    /// Extract the visible text of the page (skips script/style/noscript/template/svg
    /// and hidden or aria-hidden elements).
    fn extract_visible_text(&self, document: &Html) -> String {
        let body_selector = match Selector::parse("body") {
            Ok(s) => s,
            Err(_) => return String::new(),
        };
        let Some(body) = document.select(&body_selector).next() else {
            return String::new();
        };
        let mut parts: Vec<String> = Vec::new();
        collect_visible_text(&body, &mut parts);
        parts.join(" ")
    }

    /// Flesch Reading Ease (0-100). `None` when there is not enough text.
    fn compute_readability(&self, text: &str) -> Option<f64> {
        let text = text.trim();
        if text.is_empty() {
            return None;
        }

        let words: Vec<&str> = text.split_whitespace().collect();
        if words.len() < 30 {
            return None;
        }

        let sentences = count_sentences(text);
        if sentences == 0 {
            return None;
        }

        let syllables: usize = words.iter().map(|w| count_syllables(w)).sum();
        let asl = words.len() as f64 / sentences as f64;
        let asw = syllables as f64 / words.len() as f64;
        let score = 206.835 - (1.015 * asl) - (84.6 * asw);

        Some(score.clamp(0.0, 100.0))
    }

    /// Simhash fingerprint of the visible text, hex-encoded.
    fn compute_content_hash(&self, text: &str) -> Option<String> {
        if text.trim().is_empty() {
            return None;
        }
        Some(format!("{:x}", simhash::simhash(text)))
    }

    /// Top keywords (word frequency), filtered by stopwords, capped at 20.
    fn extract_keywords(&self, text: &str) -> Vec<Keyword> {
        let mut counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

        for raw in text.split_whitespace() {
            let word: String = raw
                .chars()
                .filter(|c| c.is_alphabetic() || *c == '\'')
                .collect::<String>()
                .to_lowercase();
            if word.len() < 4 {
                continue;
            }
            if is_stopword(&word) {
                continue;
            }
            *counts.entry(word).or_insert(0) += 1;
        }

        let mut keywords: Vec<Keyword> = counts
            .into_iter()
            .map(|(keyword, count)| Keyword { keyword, count })
            .collect();
        keywords.sort_by(|a, b| b.count.cmp(&a.count).then(a.keyword.cmp(&b.keyword)));
        keywords.truncate(20);
        keywords
    }

    /// Extract Open Graph and Twitter Card meta tags.
    fn extract_og_meta(&self, document: &Html) -> OgMeta {
        let mut meta = OgMeta::default();

        let prop_selector = match Selector::parse(r#"meta[property^="og:"], meta[name^="og:"]"#) {
            Ok(s) => s,
            Err(_) => return meta,
        };

        for el in document.select(&prop_selector) {
            let key = el
                .value()
                .attr("property")
                .or_else(|| el.value().attr("name"))
                .unwrap_or("")
                .to_string();
            let content = el.value().attr("content").unwrap_or("").trim().to_string();
            if content.is_empty() {
                continue;
            }
            match key.as_str() {
                "og:title" => meta.og_title = Some(content),
                "og:description" => meta.og_description = Some(content),
                "og:image" => meta.og_image = Some(content),
                "og:image:alt" => meta.og_image_alt = Some(content),
                "og:type" => meta.og_type = Some(content),
                "og:url" => meta.og_url = Some(content),
                "og:site_name" => meta.og_site_name = Some(content),
                "og:locale" => meta.og_locale = Some(content),
                _ => {}
            }
        }

        let twitter_selector = match Selector::parse(r#"meta[name^="twitter:"]"#) {
            Ok(s) => s,
            Err(_) => return meta,
        };

        for el in document.select(&twitter_selector) {
            let key = el.value().attr("name").unwrap_or("").to_string();
            let content = el.value().attr("content").unwrap_or("").trim().to_string();
            if content.is_empty() {
                continue;
            }
            match key.as_str() {
                "twitter:card" => meta.twitter_card = Some(content),
                "twitter:title" => meta.twitter_title = Some(content),
                "twitter:description" => meta.twitter_description = Some(content),
                "twitter:image" => meta.twitter_image = Some(content),
                _ => {}
            }
        }

        meta
    }
}

fn collect_visible_text(el: &scraper::ElementRef, out: &mut Vec<String>) {
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
                    collect_visible_text(&child_el, out);
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
    count.max(1)
}

fn count_syllables(word: &str) -> usize {
    let lower = word.to_lowercase();
    let mut count = 0usize;
    let mut prev_vowel = false;

    for c in lower.chars() {
        let is_vowel = matches!(
            c,
            'a' | 'e' | 'i' | 'o' | 'u' | 'á' | 'é' | 'í' | 'ó' | 'ú' | 'ü' | 'y'
        );
        if is_vowel && !prev_vowel {
            count += 1;
        }
        prev_vowel = is_vowel;
    }

    if count == 0 {
        return 1;
    }

    // Silent final 'e' (English heuristic, e.g. "home")
    if lower.ends_with('e') && !lower.ends_with("le") {
        count -= 1;
    }

    count.max(1)
}

fn is_stopword(word: &str) -> bool {
    const STOPWORDS: &[&str] = &[
        // English
        "the", "and", "for", "with", "that", "this", "from", "have", "are", "was", "were", "been",
        "will", "would", "could", "should", "shall", "might", "must", "can", "may", "their",
        "there", "these", "those", "them", "they", "than", "then", "which", "what", "when",
        "where", "while", "who", "whom", "whose", "why", "how", "your", "yours", "you", "our",
        "ours", "his", "her", "hers", "its", "not", "but", "also", "into", "about", "after",
        "before", "between", "over", "under", "again", "once", "out", "down", "off", "very",
        "just", "because", "every", "some", "such", "only", "other", "each", "both", "more",
        "most", "few", "same", "here", "upon", "through", "during", "above", "below", "does",
        "did", "doing", "has", "had", "having", "get", "got", "one", "two", "three", "first",
        "second", "next", "last", "any", "another", "those", "being", "been", "come", "go", "goes",
        "going", "am", "is", // Spanish
        "como", "para", "por", "con", "los", "las", "una", "uno", "unas", "unos", "del", "que",
        "cual", "quien", "este", "esta", "esto", "ese", "esa", "eso", "aquel", "aquella", "ello",
        "ella", "ellos", "ellas", "usted", "ustedes", "nosotros", "nosotras", "vosotros",
        "vosotras", "sido", "ser", "estos", "estas", "entre", "sobre", "hacia", "desde", "hasta",
        "durante", "contra", "mediante", "tambien", "pero", "aunque", "porque", "sino", "sino",
        "si", "no", "ni", "ya", "mas", "menos", "todo", "toda", "todos", "todas", "nada", "algo",
        "alguien", "nadie", "muy", "tan", "tanto", "cuando", "donde", "el", "me", "te", "se", "lo",
        "la", "le", "les", "mi", "mis", "tu", "tus", "su", "sus", "nuestro", "nuestra", "vuestro",
        "vuestra", "estan", "estas", "son", "era", "fue", "sera",
    ];
    STOPWORDS.contains(&word)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_semantics_clean_page() {
        let parser = SeoParser::new();
        let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <title>Test Page</title>
    <meta name="description" content="Test description">
    <link rel="canonical" href="https://example.com">
</head>
<body>
    <header>Header</header>
    <nav><a href="/page1">Link1</a></nav>
    <main>
        <h1>Main Title</h1>
        <h2>Section</h2>
        <img src="test.jpg" alt="Test image">
        <a href="/page2">Link2</a>
        <label for="name">Name</label>
        <input type="text" id="name">
    </main>
    <footer>Footer</footer>
</body>
</html>"#;
        let document = Html::parse_document(html);
        let issues = parser.analyze_semantics(&document);

        // Should have no errors, maybe some info
        let errors: Vec<_> = issues.iter().filter(|i| i.severity == "error").collect();
        assert!(
            errors.is_empty(),
            "Clean page should have no errors: {:?}",
            errors
        );
    }

    #[test]
    fn test_analyze_semantics_missing_everything() {
        let parser = SeoParser::new();
        let html = r#"<!DOCTYPE html>
<html>
<head></head>
<body>
    <div>
        <img src="test.jpg">
        <input type="text">
        <a href="/page"></a>
    </div>
</body>
</html>"#;
        let document = Html::parse_document(html);
        let issues = parser.analyze_semantics(&document);

        let error_types: Vec<&str> = issues
            .iter()
            .filter(|i| i.severity == "error")
            .map(|i| i.issue_type.as_str())
            .collect();
        assert!(error_types.contains(&"missing_html_lang"));
        assert!(error_types.contains(&"missing_title"));
        assert!(error_types.contains(&"img_no_alt"));
        assert!(error_types.contains(&"input_no_label"));
    }

    #[test]
    fn test_input_label_rule_and_img_alt_rule() {
        let parser = SeoParser::new();
        let html = r#"<!DOCTYPE html>
<html lang="en">
<head><title>Test</title></head>
<body>
    <main>
        <label for="a">A</label>
        <input type="text" id="a">
        <input type="text" id="b">
        <label>B<input type="text"></label>
        <label><span><input type="text"></span></label>
        <label for="c"><input type="text" id="c2"></label>
        <input type="text">
        <img src="x.jpg">
        <img src="y.jpg" alt="">
        <img src="z.jpg" alt="ok">
    </main>
</body>
</html>"#;
        let document = Html::parse_document(html);
        let issues = parser.analyze_semantics(&document);

        let img_issues: Vec<_> = issues
            .iter()
            .filter(|i| i.issue_type == "img_no_alt")
            .collect();
        assert_eq!(
            img_issues.len(),
            2,
            "expected 2 imgs without alt, got {:#?}",
            img_issues
        );
        for issue in img_issues {
            assert!(
                issue.xpath.is_some(),
                "img_no_alt issue should carry an xpath"
            );
            assert!(
                issue.css_selector.is_some(),
                "img_no_alt issue should carry a css_selector"
            );
            assert!(
                issue.snippet.is_some(),
                "img_no_alt issue should carry a snippet"
            );
        }

        let dims_issues: Vec<_> = issues
            .iter()
            .filter(|i| i.issue_type == "img_no_dimensions")
            .collect();
        assert_eq!(
            dims_issues.len(),
            3,
            "expected 3 imgs without dimensions, got {:#?}",
            dims_issues
        );
        for issue in &dims_issues {
            assert!(
                issue.xpath.is_some(),
                "img_no_dimensions issue should carry an xpath"
            );
            assert!(
                issue.css_selector.is_some(),
                "img_no_dimensions issue should carry a css_selector"
            );
            assert!(
                issue.snippet.is_some(),
                "img_no_dimensions issue should carry a snippet"
            );
        }

        let label_issues: Vec<_> = issues
            .iter()
            .filter(|i| i.issue_type == "input_no_label")
            .collect();
        assert_eq!(
            label_issues.len(),
            4,
            "expected 4 invalid inputs, got {:#?}",
            label_issues
        );
        for issue in label_issues {
            assert!(
                issue.xpath.is_some(),
                "input_no_label issue should carry an xpath"
            );
            assert!(
                issue.css_selector.is_some(),
                "input_no_label issue should carry a css_selector"
            );
            assert!(
                issue.snippet.is_some(),
                "input_no_label issue should carry a snippet"
            );
        }
    }

    #[test]
    fn test_invalid_nesting_div_in_span() {
        let parser = SeoParser::new();
        let html = r#"<!DOCTYPE html>
<html lang="en">
<head><title>Test</title></head>
<body>
    <span>
        <div>This is wrong</div>
    </span>
</body>
</html>"#;
        let document = Html::parse_document(html);
        let issues = parser.analyze_semantics(&document);

        let nesting_issues: Vec<_> = issues
            .iter()
            .filter(|i| i.issue_type == "invalid_nesting")
            .collect();
        assert!(!nesting_issues.is_empty(), "Should detect div inside span");
        assert!(nesting_issues
            .iter()
            .any(|i| i.message.contains("<div>") && i.message.contains("<span>")));
    }

    #[test]
    fn test_invalid_nesting_xpath_is_clean_single_id() {
        let parser = SeoParser::new();
        let html = r#"<!DOCTYPE html>
<html lang="en">
<head><title>Test</title></head>
<body>
    <div id="menu-items">
        <span>
            <div>This is wrong</div>
        </span>
    </div>
</body>
</html>"#;
        let document = Html::parse_document(html);
        let issues = parser.analyze_semantics(&document);

        let nesting_issues: Vec<_> = issues
            .iter()
            .filter(|i| i.issue_type == "invalid_nesting")
            .collect();
        assert!(!nesting_issues.is_empty(), "Should detect div inside span");
        for issue in nesting_issues {
            let xpath = issue
                .xpath
                .as_deref()
                .expect("nesting issue should have xpath");
            assert_eq!(
                xpath.matches("[@id='").count(),
                1,
                "xpath has duplicate id selector: {xpath}"
            );
            assert!(
                !xpath.contains("']'"),
                "xpath has mangled id value: {xpath}"
            );
            assert!(xpath.contains("menu-items"), "xpath lost id value: {xpath}");
        }
    }

    #[test]
    fn test_compute_xpath_clean_single_id_path() {
        let parser = SeoParser::new();
        let html = r#"<!DOCTYPE html>
<html lang="en">
<head><title>Test</title></head>
<body>
    <ul id="menu-items">
        <li>1</li>
        <li>2</li>
        <li>3</li>
        <li>4</li>
        <li><a href="/x"><span>Link</span></a></li>
    </ul>
</body>
</html>"#;
        let document = Html::parse_document(html);
        let selector = Selector::parse("ul#menu-items li:nth-of-type(5) a span").unwrap();
        let span = document.select(&selector).next().expect("span not found");
        let xpath = compute_xpath(&span);
        assert_eq!(
            xpath, "//*[@id='menu-items']/li[5]/a/span",
            "unexpected xpath: {xpath}"
        );

        let issues = parser.analyze_semantics(&document);
        let nesting_issues: Vec<_> = issues
            .iter()
            .filter(|i| i.issue_type == "invalid_nesting" || i.issue_type == "context_nesting")
            .collect();
        for issue in nesting_issues {
            if let Some(xpath) = issue.xpath.as_deref() {
                assert_eq!(
                    xpath.matches("[@id='").count(),
                    1,
                    "xpath has duplicate id selector: {xpath}"
                );
                assert!(
                    !xpath.contains("']'"),
                    "xpath has mangled id value: {xpath}"
                );
            }
        }
    }

    #[test]
    fn test_invalid_nesting_p_in_a() {
        let parser = SeoParser::new();
        let html = r#"<!DOCTYPE html>
<html lang="en">
<head><title>Test</title></head>
<body>
    <a href="/page">
        <p>Wrong nesting</p>
    </a>
</body>
</html>"#;
        let document = Html::parse_document(html);
        let issues = parser.analyze_semantics(&document);

        let nesting_issues: Vec<_> = issues
            .iter()
            .filter(|i| i.issue_type == "context_nesting")
            .collect();
        assert!(
            !nesting_issues.is_empty(),
            "Should detect p inside a as context-dependent"
        );
    }

    #[test]
    fn test_invalid_nesting_table_in_code() {
        let parser = SeoParser::new();
        let html = r#"<!DOCTYPE html>
<html lang="en">
<head><title>Test</title></head>
<body>
    <code>
        <table><tr><td>Wrong</td></tr></table>
    </code>
</body>
</html>"#;
        let document = Html::parse_document(html);
        let issues = parser.analyze_semantics(&document);

        let nesting_issues: Vec<_> = issues
            .iter()
            .filter(|i| i.issue_type == "invalid_nesting")
            .collect();
        assert!(
            !nesting_issues.is_empty(),
            "Should detect table inside code"
        );
    }

    #[test]
    fn test_valid_nesting_div_in_body() {
        let parser = SeoParser::new();
        let html = r#"<!DOCTYPE html>
<html lang="en">
<head><title>Test</title></head>
<body>
    <div>
        <div>This is fine</div>
    </div>
</body>
</html>"#;
        let document = Html::parse_document(html);
        let issues = parser.analyze_semantics(&document);

        let nesting_issues: Vec<_> = issues
            .iter()
            .filter(|i| i.issue_type == "invalid_nesting")
            .collect();
        assert!(nesting_issues.is_empty(), "div inside div should be valid");
    }

    #[test]
    fn test_valid_nesting_span_in_div() {
        let parser = SeoParser::new();
        let html = r#"<!DOCTYPE html>
<html lang="en">
<head><title>Test</title></head>
<body>
    <div>
        <span>This is fine</span>
    </div>
</body>
</html>"#;
        let document = Html::parse_document(html);
        let issues = parser.analyze_semantics(&document);

        let nesting_issues: Vec<_> = issues
            .iter()
            .filter(|i| i.issue_type == "invalid_nesting")
            .collect();
        assert!(nesting_issues.is_empty(), "span inside div should be valid");
    }

    #[test]
    fn test_valid_nesting_li_in_ul() {
        let parser = SeoParser::new();
        let html = r#"<!DOCTYPE html>
<html lang="en">
<head><title>Test</title></head>
<body>
    <ul>
        <li>Item 1</li>
        <li>Item 2</li>
    </ul>
</body>
</html>"#;
        let document = Html::parse_document(html);
        let issues = parser.analyze_semantics(&document);

        let nesting_issues: Vec<_> = issues
            .iter()
            .filter(|i| i.issue_type == "invalid_nesting")
            .collect();
        assert!(nesting_issues.is_empty(), "li inside ul should be valid");
    }

    #[test]
    fn test_nesting_table_covers_common_cases() {
        use crate::nesting_table::{can_include, NestingStatus};

        // Invalid combinations should return Cant
        assert_eq!(can_include("span", "div"), Some(NestingStatus::Cant));
        assert_eq!(can_include("span", "table"), Some(NestingStatus::Cant));
        assert_eq!(can_include("code", "ul"), Some(NestingStatus::Cant));
        assert_eq!(can_include("label", "section"), Some(NestingStatus::Cant));

        // Valid combinations should return Can
        assert_eq!(can_include("div", "div"), Some(NestingStatus::Can));
        assert_eq!(can_include("div", "span"), Some(NestingStatus::Can));
        assert_eq!(can_include("ul", "li"), Some(NestingStatus::Can));
        assert_eq!(can_include("tr", "td"), Some(NestingStatus::Can));
        assert_eq!(can_include("table", "tr"), Some(NestingStatus::Can));
    }

    #[test]
    fn test_readability_and_keywords() {
        let parser = SeoParser::new();
        let mut body = String::from("<p>");
        for i in 0..20 {
            body.push_str(&format!(
                "The quick brown fox jumps over the lazy dog near the riverbank number {}.",
                i
            ));
        }
        body.push_str("</p>");
        let html = format!(
            "<!DOCTYPE html><html lang='en'><head><title>Test</title></head><body>{}</body></html>",
            body
        );
        let url = Url::parse("https://example.com").unwrap();
        let (data, _) = parser.parse(&html, &url);

        let score = data.readability_score.expect("should compute readability");
        assert!((0.0..=100.0).contains(&score));
        assert!(!data.keywords.is_empty());
        assert!(data.keywords.iter().any(|k| k.keyword == "riverbank"));
        assert!(data.content_hash.is_some());
    }

    #[test]
    fn test_content_hash_stable_for_similar_text() {
        let parser = SeoParser::new();
        let h1 = parser.compute_content_hash("lorem ipsum dolor sit amet consectetur adipiscing");
        let h2 = parser.compute_content_hash("lorem ipsum dolor sit amet consectetur adipiscing");
        let h3 = parser.compute_content_hash("totally different content goes here instead");
        assert_eq!(h1, h2);
        assert_ne!(h1, h3);
    }

    #[test]
    fn test_visible_text_skips_scripts() {
        let parser = SeoParser::new();
        let html = r#"<!DOCTYPE html><html lang="en"><head><title>T</title>
            <script>var secret = "should not appear";</script>
            <style>.hidden{color:red}</style>
        </head><body>
            <p>Visible paragraph text</p>
            <div aria-hidden="true">Hidden from screen readers</div>
            <noscript>noscript fallback</noscript>
        </body></html>"#;
        let url = Url::parse("https://example.com").unwrap();
        let (data, _) = parser.parse(html, &url);
        let text = parser.extract_visible_text(&parser_doc(html));
        assert!(text.contains("Visible paragraph text"));
        assert!(!text.contains("should not appear"));
        assert!(!text.contains("Hidden from screen readers"));
        assert!(!text.contains("noscript fallback"));
        assert!(data.content_hash.is_some());
    }

    fn parser_doc(html: &str) -> Html {
        Html::parse_document(html)
    }

    #[test]
    fn test_og_meta_extraction() {
        let parser = SeoParser::new();
        let html = r#"<!DOCTYPE html><html lang="en"><head><title>T</title>
            <meta property="og:title" content="My Awesome Page">
            <meta property="og:description" content="A great description">
            <meta property="og:image" content="https://example.com/img.png">
            <meta property="og:type" content="article">
            <meta name="twitter:card" content="summary_large_image">
            <meta name="twitter:title" content="Tweet Title">
        </head><body><h1>Hello</h1></body></html>"#;
        let url = Url::parse("https://example.com").unwrap();
        let (data, _) = parser.parse(html, &url);

        assert_eq!(data.og_meta.og_title.as_deref(), Some("My Awesome Page"));
        assert_eq!(
            data.og_meta.og_description.as_deref(),
            Some("A great description")
        );
        assert_eq!(
            data.og_meta.og_image.as_deref(),
            Some("https://example.com/img.png")
        );
        assert_eq!(data.og_meta.og_type.as_deref(), Some("article"));
        assert_eq!(
            data.og_meta.twitter_card.as_deref(),
            Some("summary_large_image")
        );
        assert_eq!(data.og_meta.twitter_title.as_deref(), Some("Tweet Title"));
        assert!(!data.og_meta.is_empty());
    }

    #[test]
    fn test_multiple_main_detected() {
        let parser = SeoParser::new();
        let html = r#"<!DOCTYPE html>
<html lang="en">
<head><title>T</title></head>
<body>
    <main>One</main>
    <main>Two</main>
</body>
</html>"#;
        let issues = parser.analyze_semantics(&parser_doc(html));
        let main_issues: Vec<_> = issues
            .iter()
            .filter(|i| i.issue_type == "multiple_main")
            .collect();
        assert_eq!(main_issues.len(), 1);
        assert_eq!(main_issues[0].severity, "error");
    }

    #[test]
    fn test_skip_link_detection() {
        let parser = SeoParser::new();
        let with_skip = r##"<!DOCTYPE html>
<html lang="en">
<head><title>T</title></head>
<body>
    <a href="#main">Skip to content</a>
    <main><h1>Title</h1></main>
</body>
</html>"##;
        let issues = parser.analyze_semantics(&parser_doc(with_skip));
        assert!(
            !issues.iter().any(|i| i.issue_type == "skip_link_missing"),
            "skip link present, no issue expected"
        );

        let without_skip = r#"<!DOCTYPE html>
<html lang="en">
<head><title>T</title></head>
<body>
    <main><h1>Title</h1></main>
</body>
</html>"#;
        let issues = parser.analyze_semantics(&parser_doc(without_skip));
        assert!(
            issues.iter().any(|i| i.issue_type == "skip_link_missing"),
            "missing skip link should be reported"
        );
    }

    #[test]
    fn test_extract_links_captures_rel_tokens_and_flags() {
        let parser = SeoParser::new();
        let html = r#"<!DOCTYPE html>
<html lang="en">
<head><title>T</title></head>
<body>
    <a href="https://x.com/next" rel="nofollow ugc">User link</a>
    <a href="https://x.com/ads" rel="sponsored">Promoted</a>
    <a href="https://x.com/plain">Plain</a>
    <a href="https://y.com/ext">External</a>
</body>
</html>"#;
        let doc = parser_doc(html);
        let links = parser.extract_links(&doc, &Url::parse("https://x.com/start").unwrap());

        let by_url: std::collections::HashMap<&str, &OutgoingLink> =
            links.iter().map(|l| (l.url.as_str(), l)).collect();

        let user = by_url["https://x.com/next"];
        assert_eq!(user.rel_tokens, vec!["nofollow", "ugc"]);
        assert!(!user.is_follow);
        assert!(!user.is_sponsored);
        assert!(user.is_ugc);

        let ads = by_url["https://x.com/ads"];
        assert_eq!(ads.rel_tokens, vec!["sponsored"]);
        assert!(ads.is_follow);
        assert!(ads.is_sponsored);
        assert!(!ads.is_ugc);

        let plain = by_url["https://x.com/plain"];
        assert!(plain.rel_tokens.is_empty());
        assert!(plain.is_follow);
        assert!(!plain.is_sponsored);
        assert!(!plain.is_ugc);
    }

    #[test]
    fn test_section_requires_accessible_name() {
        let parser = SeoParser::new();
        let unnamed = r#"<!DOCTYPE html>
<html lang="en">
<head><title>T</title></head>
<body>
    <main><h1>Title</h1><section>content</section></main>
</body>
</html>"#;
        let issues = parser.analyze_semantics(&parser_doc(unnamed));
        assert!(
            issues
                .iter()
                .any(|i| i.issue_type == "section_no_accessible_name"),
            "unnamed section should be reported"
        );

        let named = r#"<!DOCTYPE html>
<html lang="en">
<head><title>T</title></head>
<body>
    <main><h1>Title</h1><section aria-label="Pricing"><h2>Plans</h2></section></main>
</body>
</html>"#;
        let issues = parser.analyze_semantics(&parser_doc(named));
        assert!(
            !issues
                .iter()
                .any(|i| i.issue_type == "section_no_accessible_name"),
            "named section should pass"
        );
    }

    #[test]
    fn test_form_requires_accessible_name() {
        let parser = SeoParser::new();
        let unnamed = r#"<!DOCTYPE html>
<html lang="en">
<head><title>T</title></head>
<body>
    <form><input type="text" id="a"><label for="a">A</label></form>
</body>
</html>"#;
        let issues = parser.analyze_semantics(&parser_doc(unnamed));
        assert!(
            issues
                .iter()
                .any(|i| i.issue_type == "form_no_accessible_name"),
            "unnamed form should be reported"
        );

        let named = r#"<!DOCTYPE html>
<html lang="en">
<head><title>T</title></head>
<body>
    <form aria-label="Contact"><input type="text" id="a"><label for="a">A</label></form>
</body>
</html>"#;
        let issues = parser.analyze_semantics(&parser_doc(named));
        assert!(
            !issues
                .iter()
                .any(|i| i.issue_type == "form_no_accessible_name"),
            "named form should pass"
        );
    }

    #[test]
    fn test_button_requires_accessible_name() {
        let parser = SeoParser::new();
        let unnamed = r#"<!DOCTYPE html>
<html lang="en">
<head><title>T</title></head>
<body>
    <button></button>
</body>
</html>"#;
        let issues = parser.analyze_semantics(&parser_doc(unnamed));
        assert!(
            issues.iter().any(|i| i.issue_type == "button_no_name"),
            "empty button should be reported"
        );

        let icon_button = r#"<!DOCTYPE html>
<html lang="en">
<head><title>T</title></head>
<body>
    <button aria-label="Close"><svg></svg></button>
</body>
</html>"#;
        let issues = parser.analyze_semantics(&parser_doc(icon_button));
        assert!(
            !issues.iter().any(|i| i.issue_type == "button_no_name"),
            "aria-label button should pass"
        );
    }

    #[test]
    fn test_duplicate_ids_detected() {
        let parser = SeoParser::new();
        let html = r#"<!DOCTYPE html>
<html lang="en">
<head><title>T</title></head>
<body>
    <div id="dup">a</div>
    <div id="dup">b</div>
    <div id="unique">c</div>
</body>
</html>"#;
        let issues = parser.analyze_semantics(&parser_doc(html));
        let dups: Vec<_> = issues
            .iter()
            .filter(|i| i.issue_type == "duplicate_id")
            .collect();
        assert_eq!(dups.len(), 1, "one duplicate occurrence beyond the first");
        assert!(dups[0].message.contains("dup"), "message names the id");
        assert_eq!(dups[0].severity, "error");
    }

    #[test]
    fn test_aria_current_nav() {
        let parser = SeoParser::new();
        let with_current = r#"<!DOCTYPE html>
<html lang="en">
<head><title>T</title></head>
<body>
    <nav><a href="/" aria-current="page">Home</a><a href="/about">About</a></nav>
    <main><h1>Title</h1></main>
</body>
</html>"#;
        let issues = parser.analyze_semantics(&parser_doc(with_current));
        assert!(
            !issues.iter().any(|i| i.issue_type == "aria_current_nav"),
            "aria-current present, no issue expected"
        );

        let without_current = r#"<!DOCTYPE html>
<html lang="en">
<head><title>T</title></head>
<body>
    <nav><a href="/">Home</a><a href="/about">About</a></nav>
    <main><h1>Title</h1></main>
</body>
</html>"#;
        let issues = parser.analyze_semantics(&parser_doc(without_current));
        assert!(
            issues.iter().any(|i| i.issue_type == "aria_current_nav"),
            "nav without aria-current should be reported"
        );
    }

    #[test]
    fn test_article_missing_for_substantial_main() {
        let parser = SeoParser::new();
        let html = r#"<!DOCTYPE html>
<html lang="en">
<head><title>T</title></head>
<body>
    <main>
        <h1>Title</h1>
        <p>This is the first paragraph of a long article with enough words to be considered substantial, meaningful content that deserves to be wrapped in an article element for better extraction.</p>
        <p>This second paragraph adds even more detail and keeps expanding the body of text so that the total word count crosses the fifty word threshold used by the analysis.</p>
        <p>A third paragraph rounds out the content, making the main section clearly substantial and self-contained.</p>
    </main>
</body>
</html>"#;
        let issues = parser.analyze_semantics(&parser_doc(html));
        assert!(
            issues.iter().any(|i| i.issue_type == "article_missing"),
            "substantial main without article should be reported"
        );

        let with_article = r#"<!DOCTYPE html>
<html lang="en">
<head><title>T</title></head>
<body>
    <main>
        <article>
            <h1>Title</h1>
            <p>This is the first paragraph of a long article with enough words to be considered substantial, meaningful content that deserves to be wrapped in an article element for better extraction.</p>
            <p>This second paragraph adds even more detail and keeps expanding the body of text so that the total word count crosses the fifty word threshold used by the analysis.</p>
            <p>A third paragraph rounds out the content, making the main section clearly substantial and self-contained.</p>
        </article>
    </main>
</body>
</html>"#;
        let issues = parser.analyze_semantics(&parser_doc(with_article));
        assert!(
            !issues.iter().any(|i| i.issue_type == "article_missing"),
            "article present, no issue expected"
        );
    }

    #[test]
    fn test_time_without_datetime() {
        let parser = SeoParser::new();
        let html = r#"<!DOCTYPE html>
<html lang="en">
<head><title>T</title></head>
<body>
    <main>
        <time datetime="2026-08-11">Aug 11, 2026</time>
        <time>Yesterday</time>
        <time>Today</time>
    </main>
</body>
</html>"#;
        let issues = parser.analyze_semantics(&parser_doc(html));
        let times: Vec<_> = issues
            .iter()
            .filter(|i| i.issue_type == "time_without_datetime")
            .collect();
        assert_eq!(times.len(), 2, "two <time> without datetime expected");
        assert!(times.iter().all(|i| i.xpath.is_some()));
    }

    #[test]
    fn test_figure_and_table_checks() {
        let parser = SeoParser::new();
        let html = r#"<!DOCTYPE html>
<html lang="en">
<head><title>T</title></head>
<body>
    <main>
        <figure><img src="a.png" alt="a"><figcaption>Caption A</figcaption></figure>
        <figure><img src="b.png" alt="b"></figure>
        <table>
            <caption>Planets</caption>
            <tr><th>Name</th><th>Radius</th></tr>
            <tr><td>Earth</td><td>6371</td></tr>
        </table>
        <table>
            <tr><td>No headers</td></tr>
        </table>
    </main>
</body>
</html>"#;
        let issues = parser.analyze_semantics(&parser_doc(html));

        let figs: Vec<_> = issues
            .iter()
            .filter(|i| i.issue_type == "figure_without_caption")
            .collect();
        assert_eq!(figs.len(), 1, "one figure without caption expected");

        let no_th: Vec<_> = issues
            .iter()
            .filter(|i| i.issue_type == "table_without_headers")
            .collect();
        assert_eq!(no_th.len(), 1, "one table without th expected");
        assert_eq!(no_th[0].severity, "error");

        let no_cap: Vec<_> = issues
            .iter()
            .filter(|i| i.issue_type == "table_without_caption")
            .collect();
        assert_eq!(no_cap.len(), 1, "one table without caption expected");
    }

    #[test]
    fn test_blockquote_attribution_and_media_accessibility() {
        let parser = SeoParser::new();
        let html = r#"<!DOCTYPE html>
<html lang="en">
<head><title>T</title></head>
<body>
    <main>
        <blockquote cite="https://example.com/source">Cited quote</blockquote>
        <blockquote>Unattributed quote</blockquote>
        <iframe src="https://example.com"></iframe>
        <iframe src="https://example.com" title="Map"></iframe>
        <video src="v.mp4"></video>
        <video src="v2.mp4" controls></video>
    </main>
</body>
</html>"#;
        let issues = parser.analyze_semantics(&parser_doc(html));

        let quotes: Vec<_> = issues
            .iter()
            .filter(|i| i.issue_type == "blockquote_without_attribution")
            .collect();
        assert_eq!(quotes.len(), 1, "one unattributed blockquote expected");

        let iframes: Vec<_> = issues
            .iter()
            .filter(|i| i.issue_type == "iframe_without_title")
            .collect();
        assert_eq!(iframes.len(), 1, "one iframe without title expected");

        let videos: Vec<_> = issues
            .iter()
            .filter(|i| i.issue_type == "video_without_track_or_controls")
            .collect();
        assert_eq!(videos.len(), 1, "one video without controls expected");
    }

    #[test]
    fn test_text_in_div_instead_of_p() {
        let parser = SeoParser::new();
        let html = r#"<!DOCTYPE html>
<html lang="en">
<head><title>T</title></head>
<body>
    <main>
        <div>This is a long block of text sitting directly inside a div element instead of a paragraph tag, and it keeps going until there are well over twenty words present to trigger the warning.</div>
        <div><p>A short child paragraph is fine.</p></div>
        <span>Short</span>
    </main>
</body>
</html>"#;
        let issues = parser.analyze_semantics(&parser_doc(html));
        let divs: Vec<_> = issues
            .iter()
            .filter(|i| i.issue_type == "text_in_div_instead_of_p")
            .collect();
        assert_eq!(divs.len(), 1, "one text-heavy div expected");
        assert!(divs[0].xpath.is_some());
    }

    #[test]
    fn test_empty_headings_and_paragraphs() {
        let parser = SeoParser::new();
        let html = r#"<!DOCTYPE html>
<html lang="en">
<head><title>T</title></head>
<body>
    <main>
        <h1>Real heading</h1>
        <h2></h2>
        <h3>   </h3>
        <p></p>
        <p> </p>
        <p>Real paragraph</p>
    </main>
</body>
</html>"#;
        let issues = parser.analyze_semantics(&parser_doc(html));

        let headings: Vec<_> = issues
            .iter()
            .filter(|i| i.issue_type == "empty_heading")
            .collect();
        assert_eq!(headings.len(), 2, "two empty headings expected");
        assert!(headings.iter().all(|i| i.severity == "warning"));
        assert!(headings.iter().all(|i| i.xpath.is_some()));

        let paragraphs: Vec<_> = issues
            .iter()
            .filter(|i| i.issue_type == "empty_paragraph")
            .collect();
        assert_eq!(paragraphs.len(), 2, "two empty paragraphs expected");
        assert!(paragraphs.iter().all(|i| i.severity == "warning"));
    }

    #[test]
    fn test_empty_content_tags_detected() {
        let parser = SeoParser::new();
        let html = r#"<!DOCTYPE html>
<html lang="en">
<head><title>T</title></head>
<body>
    <main>
        <div></div>
        <ul><li>One</li><li></li></ul>
        <section></section>
        <span></span>
    </main>
</body>
</html>"#;
        let issues = parser.analyze_semantics(&parser_doc(html));
        let empties: Vec<_> = issues
            .iter()
            .filter(|i| i.issue_type == "empty_content_tag")
            .collect();
        assert_eq!(empties.len(), 4, "four empty content tags expected");
        assert!(empties.iter().all(|i| i.severity == "info"));
        assert!(empties.iter().any(|i| i.element == "<li>"));
    }

    #[test]
    fn test_empty_elements_skips_intentional_and_void() {
        let parser = SeoParser::new();
        let html = r#"<!DOCTYPE html>
<html lang="en">
<head><title>T</title></head>
<body>
    <main>
        <div id="app"></div>
        <span class="icon"></span>
        <div style="height: 1px"></div>
        <div aria-hidden="true"></div>
        <img src="x.jpg">
        <input type="text">
        <div><p>Has a child element</p></div>
    </main>
</body>
</html>"#;
        let issues = parser.analyze_semantics(&parser_doc(html));
        let empties: Vec<_> = issues
            .iter()
            .filter(|i| {
                matches!(
                    i.issue_type.as_str(),
                    "empty_heading" | "empty_paragraph" | "empty_content_tag"
                )
            })
            .collect();
        assert!(
            empties.is_empty(),
            "no empty-tag issues expected, got {:?}",
            empties
        );
    }
}
