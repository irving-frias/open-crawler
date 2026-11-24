//! Custom extraction: user-defined rules that pull values out of every
//! crawled page of a project (Screaming Frog's "custom extraction").
//!
//! A rule is a name, a selector type (CSS selector, XPath subset — see
//! [`crate::crawler::xpath`] — or regex) and a mode: `text` (first match's
//! text), `attribute` (a named attribute of the first match) or `count`
//! (number of matches). Rules live per project in the `custom_extractions`
//! table; values are computed at crawl time (and on re-audit / on demand)
//! and stored per page as a JSON object `{name: string|number|null}` in
//! `crawled_pages.custom_extractions_json` (`null` = no match).

use regex::Regex;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

use crate::crawler::xpath;

/// How the value is selected from the page.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SelectorType {
    Css,
    Xpath,
    Regex,
}

impl SelectorType {
    pub fn from_name(s: &str) -> Option<Self> {
        match s {
            "css" => Some(Self::Css),
            "xpath" => Some(Self::Xpath),
            "regex" => Some(Self::Regex),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Css => "css",
            Self::Xpath => "xpath",
            Self::Regex => "regex",
        }
    }
}

/// What is read from the matched node(s).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExtractionMode {
    Text,
    Attribute,
    Count,
}

impl ExtractionMode {
    pub fn from_name(s: &str) -> Option<Self> {
        match s {
            "text" => Some(Self::Text),
            "attribute" => Some(Self::Attribute),
            "count" => Some(Self::Count),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Attribute => "attribute",
            Self::Count => "count",
        }
    }
}

/// One user-defined extraction rule, scoped to a project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomExtraction {
    pub id: String,
    pub name: String,
    pub selector_type: SelectorType,
    pub selector: String,
    pub mode: ExtractionMode,
    #[serde(default)]
    pub attribute: Option<String>,
    #[serde(default)]
    pub position: i64,
}

/// Checks a rule before it is saved: name and selector present, the selector
/// parses for its type, and the mode fits the type (regex has no attributes).
pub fn validate(rule: &CustomExtraction) -> Result<(), String> {
    if rule.name.trim().is_empty() {
        return Err("extraction name cannot be empty".into());
    }
    if rule.selector.trim().is_empty() {
        return Err("extraction selector cannot be empty".into());
    }
    match rule.selector_type {
        SelectorType::Css => {
            Selector::parse(&rule.selector).map_err(|e| format!("invalid CSS selector: {e:?}"))?;
        }
        SelectorType::Xpath => xpath::validate(&rule.selector)?,
        SelectorType::Regex => {
            Regex::new(&rule.selector).map_err(|e| format!("invalid regex: {e}"))?;
        }
    }
    if rule.mode == ExtractionMode::Attribute {
        match rule.selector_type {
            SelectorType::Regex => {
                return Err("regex extraction has no attributes; use text or count".into());
            }
            _ => {
                if rule
                    .attribute
                    .as_deref()
                    .is_none_or(|a| a.trim().is_empty())
                {
                    return Err("attribute mode needs an attribute name".into());
                }
            }
        }
    }
    Ok(())
}

/// Applies every rule to one page's HTML and returns `{name: value}`.
/// The document is parsed once and shared by the CSS/XPath rules; regex
/// rules run on the raw HTML string.
pub fn extract(
    html: &str,
    rules: &[CustomExtraction],
) -> serde_json::Map<String, serde_json::Value> {
    let mut out = serde_json::Map::new();
    if rules.is_empty() {
        return out;
    }
    let needs_doc = rules
        .iter()
        .any(|r| matches!(r.selector_type, SelectorType::Css | SelectorType::Xpath));
    let doc = needs_doc.then(|| Html::parse_document(html));
    for rule in rules {
        let value = extract_one_value(html, doc.as_ref(), rule).unwrap_or(serde_json::Value::Null);
        out.insert(rule.name.clone(), value);
    }
    out
}

/// Applies a single rule to one page's HTML (used by the "test rule"
/// command). Returns an error instead of `null` when the rule itself fails.
pub fn extract_one(html: &str, rule: &CustomExtraction) -> Result<serde_json::Value, String> {
    validate(rule)?;
    let doc = match rule.selector_type {
        SelectorType::Css | SelectorType::Xpath => Some(Html::parse_document(html)),
        SelectorType::Regex => None,
    };
    extract_one_value(html, doc.as_ref(), rule)
}

fn extract_one_value(
    html: &str,
    doc: Option<&Html>,
    rule: &CustomExtraction,
) -> Result<serde_json::Value, String> {
    match rule.selector_type {
        SelectorType::Css => {
            let doc = doc.ok_or("no document for CSS extraction")?;
            let sel = Selector::parse(&rule.selector)
                .map_err(|e| format!("invalid CSS selector: {e:?}"))?;
            match rule.mode {
                ExtractionMode::Text => Ok(doc
                    .select(&sel)
                    .next()
                    .map(|el| {
                        serde_json::Value::String(el.text().collect::<String>().trim().to_string())
                    })
                    .unwrap_or(serde_json::Value::Null)),
                ExtractionMode::Attribute => {
                    let name = rule.attribute.clone().unwrap_or_default();
                    Ok(doc
                        .select(&sel)
                        .next()
                        .and_then(|el| el.attr(&name))
                        .map(|v| serde_json::Value::String(v.to_string()))
                        .unwrap_or(serde_json::Value::Null))
                }
                ExtractionMode::Count => Ok(serde_json::Value::from(doc.select(&sel).count())),
            }
        }
        SelectorType::Xpath => {
            let doc = doc.ok_or("no document for XPath extraction")?;
            match rule.mode {
                ExtractionMode::Text => {
                    // A top-level string(...) call always yields a string
                    // ("" when nothing matches); a plain path yields null
                    // when nothing matches.
                    if rule.selector.trim_start().starts_with("string(") {
                        Ok(serde_json::Value::String(xpath::eval_string(
                            doc,
                            &rule.selector,
                        )?))
                    } else {
                        let nodes = xpath::eval(doc, &rule.selector)?;
                        Ok(nodes
                            .into_iter()
                            .next()
                            .map(|n| serde_json::Value::String(n.string_value().trim().to_string()))
                            .unwrap_or(serde_json::Value::Null))
                    }
                }
                ExtractionMode::Attribute => {
                    let name = rule.attribute.clone().unwrap_or_default();
                    let nodes = xpath::eval(doc, &rule.selector)?;
                    Ok(nodes
                        .into_iter()
                        .next()
                        .and_then(|n| match n {
                            xpath::XNode::Attr(v) => Some(serde_json::Value::String(v)),
                            xpath::XNode::Element(el) => el
                                .attr(&name)
                                .map(|v| serde_json::Value::String(v.to_string())),
                            xpath::XNode::Text(_) => None,
                        })
                        .unwrap_or(serde_json::Value::Null))
                }
                ExtractionMode::Count => Ok(serde_json::Value::from(xpath::eval_count(
                    doc,
                    &rule.selector,
                )?)),
            }
        }
        SelectorType::Regex => {
            let re = Regex::new(&rule.selector).map_err(|e| format!("invalid regex: {e}"))?;
            match rule.mode {
                ExtractionMode::Text => Ok(re
                    .captures(html)
                    .and_then(|caps| {
                        caps.get(1)
                            .or_else(|| caps.get(0))
                            .map(|m| serde_json::Value::String(m.as_str().to_string()))
                    })
                    .unwrap_or(serde_json::Value::Null)),
                ExtractionMode::Attribute => {
                    Err("regex extraction has no attributes; use text or count".into())
                }
                ExtractionMode::Count => Ok(serde_json::Value::from(re.find_iter(html).count())),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(
        name: &str,
        selector_type: SelectorType,
        selector: &str,
        mode: ExtractionMode,
    ) -> CustomExtraction {
        CustomExtraction {
            id: "r1".into(),
            name: name.into(),
            selector_type,
            selector: selector.into(),
            mode,
            attribute: None,
            position: 0,
        }
    }

    const HTML: &str = r#"<html><head>
        <meta name="description" content="Page desc">
        <title>Test Page</title></head>
        <body><h1>Hello World</h1>
        <p class="price">$19.99</p><p class="price">$9.99</p>
        <a href="/a">one</a><a href="/b">two</a></body></html>"#;

    #[test]
    fn css_text_attribute_count() {
        let text = rule("title", SelectorType::Css, "title", ExtractionMode::Text);
        assert_eq!(
            extract_one(HTML, &text).unwrap(),
            serde_json::Value::String("Test Page".into())
        );
        let mut attr = rule(
            "desc",
            SelectorType::Css,
            "meta[name=description]",
            ExtractionMode::Attribute,
        );
        attr.attribute = Some("content".into());
        assert_eq!(
            extract_one(HTML, &attr).unwrap(),
            serde_json::Value::String("Page desc".into())
        );
        let count = rule("links", SelectorType::Css, "a", ExtractionMode::Count);
        assert_eq!(
            extract_one(HTML, &count).unwrap(),
            serde_json::Value::from(2)
        );
        let missing = rule(
            "nope",
            SelectorType::Css,
            ".does-not-exist",
            ExtractionMode::Text,
        );
        assert_eq!(
            extract_one(HTML, &missing).unwrap(),
            serde_json::Value::Null
        );
    }

    #[test]
    fn xpath_text_attribute_count() {
        let text = rule("h1", SelectorType::Xpath, "//h1", ExtractionMode::Text);
        assert_eq!(
            extract_one(HTML, &text).unwrap(),
            serde_json::Value::String("Hello World".into())
        );
        let mut attr = rule(
            "desc",
            SelectorType::Xpath,
            "//meta[@name='description']",
            ExtractionMode::Attribute,
        );
        attr.attribute = Some("content".into());
        assert_eq!(
            extract_one(HTML, &attr).unwrap(),
            serde_json::Value::String("Page desc".into())
        );
        // An XPath ending in /@attr reads the attribute directly.
        let direct = rule(
            "desc2",
            SelectorType::Xpath,
            "//meta[@name='description']/@content",
            ExtractionMode::Text,
        );
        assert_eq!(
            extract_one(HTML, &direct).unwrap(),
            serde_json::Value::String("Page desc".into())
        );
        let count = rule(
            "prices",
            SelectorType::Xpath,
            "//p[@class='price']",
            ExtractionMode::Count,
        );
        assert_eq!(
            extract_one(HTML, &count).unwrap(),
            serde_json::Value::from(2)
        );
        let missing = rule("nope", SelectorType::Xpath, "//h9", ExtractionMode::Text);
        assert_eq!(
            extract_one(HTML, &missing).unwrap(),
            serde_json::Value::Null
        );
    }

    #[test]
    fn regex_group_or_whole_match_and_count() {
        let price = rule(
            "price",
            SelectorType::Regex,
            r"\$(\d+\.\d+)",
            ExtractionMode::Text,
        );
        assert_eq!(
            extract_one(HTML, &price).unwrap(),
            serde_json::Value::String("19.99".into())
        );
        let no_group = rule(
            "dollar",
            SelectorType::Regex,
            r"\$\d+",
            ExtractionMode::Text,
        );
        assert_eq!(
            extract_one(HTML, &no_group).unwrap(),
            serde_json::Value::String("$19".into())
        );
        let count = rule(
            "prices",
            SelectorType::Regex,
            r"\$\d+\.\d+",
            ExtractionMode::Count,
        );
        assert_eq!(
            extract_one(HTML, &count).unwrap(),
            serde_json::Value::from(2)
        );
    }

    #[test]
    fn validate_rejects_bad_rules() {
        assert!(validate(&rule(
            "x",
            SelectorType::Css,
            "div >>",
            ExtractionMode::Text
        ))
        .is_err());
        assert!(validate(&rule(
            "x",
            SelectorType::Xpath,
            "//div/parent::span",
            ExtractionMode::Text
        ))
        .is_err());
        assert!(validate(&rule("x", SelectorType::Regex, "([", ExtractionMode::Text)).is_err());
        assert!(validate(&rule("", SelectorType::Css, "div", ExtractionMode::Text)).is_err());
        assert!(validate(&rule(
            "x",
            SelectorType::Regex,
            "a",
            ExtractionMode::Attribute
        ))
        .is_err());
        let mut attr = rule("x", SelectorType::Css, "a", ExtractionMode::Attribute);
        assert!(validate(&attr).is_err());
        attr.attribute = Some("href".into());
        assert!(validate(&attr).is_ok());
    }

    #[test]
    fn extract_applies_all_rules_to_one_map() {
        let rules = vec![
            rule("h1", SelectorType::Xpath, "//h1", ExtractionMode::Text),
            rule("links", SelectorType::Css, "a", ExtractionMode::Count),
            rule(
                "price",
                SelectorType::Regex,
                r"\$(\d+\.\d+)",
                ExtractionMode::Text,
            ),
        ];
        let map = extract(HTML, &rules);
        assert_eq!(map["h1"], serde_json::Value::String("Hello World".into()));
        assert_eq!(map["links"], serde_json::Value::from(2));
        assert_eq!(map["price"], serde_json::Value::String("19.99".into()));
    }
}
