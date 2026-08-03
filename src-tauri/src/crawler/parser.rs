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

#[derive(Debug, Clone)]
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
    pub is_follow: bool,
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
        Some(parent) if parent.value().is_element() => {
            match scraper::ElementRef::wrap(parent) {
                Some(pe) => {
                    let parent_path = build_absolute_xpath(&pe);
                    format!("{}/{}", parent_path, current)
                }
                None => current,
            }
        }
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
        format!("{}...", &html[..200])
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

        // 10. Multiple <h1>
        let h1_count = self.count_elements(document, "h1");
        if h1_count > 1 {
            issues.push(SemanticIssue {
                issue_type: "multiple_h1".to_string(),
                severity: "warning".to_string(),
                element: "<h1>".to_string(),
                message: format!("Multiple <h1> tags found ({})", h1_count),
                selector: Some("h1".to_string()),
            ..Default::default()
            });
        }

        // 11. Heading hierarchy skip
        if let Some(skip_issue) = self.check_heading_hierarchy(document) {
            issues.push(skip_issue);
        }

        // 12. <img> without alt
        let imgs_no_alt = self.count_elements(document, r#"img:not([alt]), img[alt=""]"#);
        if imgs_no_alt > 0 {
            issues.push(SemanticIssue {
                issue_type: "img_no_alt".to_string(),
                severity: "error".to_string(),
                element: "<img>".to_string(),
                message: format!("{} image(s) missing alt attribute", imgs_no_alt),
                selector: Some(r#"img:not([alt]), img[alt=""]"#.to_string()),
            ..Default::default()
            });
        }

        // 13. <input> without id
        let inputs_no_id = self.count_elements(document, "input:not([id]), textarea:not([id]), select:not([id])");
        if inputs_no_id > 0 {
            issues.push(SemanticIssue {
                issue_type: "input_no_id".to_string(),
                severity: "warning".to_string(),
                element: "<input>".to_string(),
                message: format!("{} form element(s) missing id attribute", inputs_no_id),
                selector: Some("input:not([id])".to_string()),
            ..Default::default()
            });
        }

        // 14. <input> without label
        let inputs_no_label = self.count_inputs_without_label(document);
        if inputs_no_label > 0 {
            issues.push(SemanticIssue {
                issue_type: "input_no_label".to_string(),
                severity: "error".to_string(),
                element: "<input>".to_string(),
                message: format!("{} input(s) without associated <label>", inputs_no_label),
                selector: Some("input".to_string()),
            ..Default::default()
            });
        }

        // 15. Empty link text
        let empty_links = self.count_empty_links(document);
        if empty_links > 0 {
            issues.push(SemanticIssue {
                issue_type: "empty_link_text".to_string(),
                severity: "warning".to_string(),
                element: "<a>".to_string(),
                message: format!("{} link(s) with no text and no aria-label", empty_links),
                selector: Some("a[href]" .to_string()),
            ..Default::default()
            });
        }

        // 16. Missing ARIA on form controls
        let missing_aria = self.count_form_controls_without_aria(document);
        if missing_aria > 0 {
            issues.push(SemanticIssue {
                issue_type: "missing_aria".to_string(),
                severity: "warning".to_string(),
                element: "<select>/<textarea>".to_string(),
                message: format!("{} form control(s) missing aria-label or aria-labelledby", missing_aria),
                selector: Some("select, textarea".to_string()),
            ..Default::default()
            });
        }

        // 17. Invalid element nesting (flow inside phrasing)
        let nesting_issues = self.check_element_nesting(document);
        issues.extend(nesting_issues);

        issues
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

    fn check_heading_hierarchy(&self, document: &Html) -> Option<SemanticIssue> {
        let selectors: Vec<(&str, u8)> = vec![
            ("h1", 1), ("h2", 2), ("h3", 3), ("h4", 4), ("h5", 5), ("h6", 6),
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

    fn count_inputs_without_label(&self, document: &Html) -> usize {
        let selector = match Selector::parse("input:not([type='hidden']):not([type='submit']):not([type='button'])") {
            Ok(s) => s,
            Err(_) => return 0,
        };

        let mut count = 0;
        for input in document.select(&selector) {
            let id = input.value().attr("id");
            let has_aria_label = input.value().attr("aria-label").is_some();
            let has_aria_labelledby = input.value().attr("aria-labelledby").is_some();
            let has_title = input.value().attr("title").is_some();

            if has_aria_label || has_aria_labelledby || has_title {
                continue;
            }

            if let Some(input_id) = id {
                let has_label = {
                    let label_selector_str = format!("label[for='{}']", input_id);
                    Selector::parse(&label_selector_str)
                        .ok()
                        .map(|s| document.select(&s).next().is_some())
                        .unwrap_or(false)
                };
                if has_label {
                    continue;
                }
                // Also check if input is inside a label
                if self.is_inside_label(document, input) {
                    continue;
                }
            }

            count += 1;
        }

        count
    }

    fn is_inside_label(&self, _document: &Html, input: scraper::ElementRef) -> bool {
        // Walk up parents to check if input is nested inside a <label>
        let mut current = input.parent();
        while let Some(parent) = current {
            if let Some(parent_el) = scraper::ElementRef::wrap(parent) {
                if parent_el.value().name() == "label" {
                    return true;
                }
            }
            current = parent.parent();
        }
        false
    }

    fn count_empty_links(&self, document: &Html) -> usize {
        let selector = match Selector::parse("a[href]") {
            Ok(s) => s,
            Err(_) => return 0,
        };

        let mut count = 0;
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

            if text.is_empty() && !has_aria_label && !has_aria_labelledby && !has_title && !has_img_alt {
                count += 1;
            }
        }

        count
    }

    fn count_form_controls_without_aria(&self, document: &Html) -> usize {
        let selector = match Selector::parse("select, textarea") {
            Ok(s) => s,
            Err(_) => return 0,
        };

        let mut count = 0;
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

            count += 1;
        }

        count
    }

    /// Check for invalid element nesting using the static caninclude table.
    /// Reports "cant" as error, "doubt" as warning.
    fn check_element_nesting(&self, document: &Html) -> Vec<SemanticIssue> {
        let mut issues = Vec::new();
        let mut seen_pairs: std::collections::HashSet<(String, String)> = std::collections::HashSet::new();

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
    fn find_parent_element<'a>(&self, element: scraper::ElementRef<'a>) -> Option<scraper::ElementRef<'a>> {
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
                let is_follow = !rel.contains("nofollow");

                Some(OutgoingLink {
                    url: absolute_url,
                    anchor_text,
                    is_follow,
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
            let content = el
                .value()
                .attr("content")
                .unwrap_or("")
                .trim()
                .to_string();
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
                _ => {}
            }
        }

        let twitter_selector = match Selector::parse(r#"meta[name^="twitter:"]"#) {
            Ok(s) => s,
            Err(_) => return meta,
        };

        for el in document.select(&twitter_selector) {
            let key = el.value().attr("name").unwrap_or("").to_string();
            let content = el
                .value()
                .attr("content")
                .unwrap_or("")
                .trim()
                .to_string();
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
        "the", "and", "for", "with", "that", "this", "from", "have", "are", "was", "were",
        "been", "will", "would", "could", "should", "shall", "might", "must", "can", "may",
        "their", "there", "these", "those", "them", "they", "than", "then", "which", "what",
        "when", "where", "while", "who", "whom", "whose", "why", "how", "your", "yours",
        "you", "our", "ours", "his", "her", "hers", "its", "not", "but", "also", "into",
        "about", "after", "before", "between", "over", "under", "again", "once", "out",
        "down", "off", "very", "just", "because", "every", "some", "such", "only", "other",
        "each", "both", "more", "most", "few", "same", "here", "upon", "through", "during",
        "above", "below", "does", "did", "doing", "has", "had", "having", "get", "got",
        "one", "two", "three", "first", "second", "next", "last", "any", "another", "those",
        "being", "been", "come", "go", "goes", "going", "am", "is",
        // Spanish
        "como", "para", "por", "con", "los", "las", "una", "uno", "unas", "unos", "del",
        "que", "cual", "quien", "este", "esta", "esto", "ese", "esa", "eso", "aquel",
        "aquella", "ello", "ella", "ellos", "ellas", "usted", "ustedes", "nosotros",
        "nosotras", "vosotros", "vosotras", "sido", "ser", "estos", "estas", "entre",
        "sobre", "hacia", "desde", "hasta", "durante", "contra", "mediante", "tambien",
        "pero", "aunque", "porque", "sino", "sino", "si", "no", "ni", "ya", "mas",
        "menos", "todo", "toda", "todos", "todas", "nada", "algo", "alguien", "nadie",
        "muy", "tan", "tanto", "cuando", "donde", "el", "me", "te", "se", "lo", "la",
        "le", "les", "mi", "mis", "tu", "tus", "su", "sus", "nuestro", "nuestra",
        "vuestro", "vuestra", "estan", "estas", "son", "era", "fue", "sera",
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
        assert!(errors.is_empty(), "Clean page should have no errors: {:?}", errors);
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

        let error_types: Vec<&str> = issues.iter().filter(|i| i.severity == "error").map(|i| i.issue_type.as_str()).collect();
        assert!(error_types.contains(&"missing_html_lang"));
        assert!(error_types.contains(&"missing_title"));
        assert!(error_types.contains(&"img_no_alt"));
        assert!(error_types.contains(&"input_no_label"));
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

        let nesting_issues: Vec<_> = issues.iter()
            .filter(|i| i.issue_type == "invalid_nesting")
            .collect();
        assert!(!nesting_issues.is_empty(), "Should detect div inside span");
        assert!(nesting_issues.iter().any(|i| i.message.contains("<div>") && i.message.contains("<span>")));
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

        let nesting_issues: Vec<_> = issues.iter()
            .filter(|i| i.issue_type == "invalid_nesting")
            .collect();
        assert!(!nesting_issues.is_empty(), "Should detect div inside span");
        for issue in nesting_issues {
            let xpath = issue.xpath.as_deref().expect("nesting issue should have xpath");
            assert_eq!(xpath.matches("[@id='").count(), 1, "xpath has duplicate id selector: {xpath}");
            assert!(!xpath.contains("']'"), "xpath has mangled id value: {xpath}");
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
        assert_eq!(xpath, "//*[@id='menu-items']/li[5]/a/span", "unexpected xpath: {xpath}");

        let issues = parser.analyze_semantics(&document);
        let nesting_issues: Vec<_> = issues.iter()
            .filter(|i| i.issue_type == "invalid_nesting" || i.issue_type == "context_nesting")
            .collect();
        for issue in nesting_issues {
            if let Some(xpath) = issue.xpath.as_deref() {
                assert_eq!(xpath.matches("[@id='").count(), 1, "xpath has duplicate id selector: {xpath}");
                assert!(!xpath.contains("']'"), "xpath has mangled id value: {xpath}");
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

        let nesting_issues: Vec<_> = issues.iter()
            .filter(|i| i.issue_type == "context_nesting")
            .collect();
        assert!(!nesting_issues.is_empty(), "Should detect p inside a as context-dependent");
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

        let nesting_issues: Vec<_> = issues.iter()
            .filter(|i| i.issue_type == "invalid_nesting")
            .collect();
        assert!(!nesting_issues.is_empty(), "Should detect table inside code");
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

        let nesting_issues: Vec<_> = issues.iter()
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

        let nesting_issues: Vec<_> = issues.iter()
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

        let nesting_issues: Vec<_> = issues.iter()
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
        let html = format!("<!DOCTYPE html><html lang='en'><head><title>Test</title></head><body>{}</body></html>", body);
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
        assert_eq!(data.og_meta.og_image.as_deref(), Some("https://example.com/img.png"));
        assert_eq!(data.og_meta.og_type.as_deref(), Some("article"));
        assert_eq!(
            data.og_meta.twitter_card.as_deref(),
            Some("summary_large_image")
        );
        assert_eq!(data.og_meta.twitter_title.as_deref(), Some("Tweet Title"));
        assert!(!data.og_meta.is_empty());
    }
}
