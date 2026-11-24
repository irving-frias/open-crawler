//! XPath 1.0 subset evaluator over the `scraper` DOM tree.
//!
//! OpenCrawler's custom extraction needs XPath, but a full XPath 1.0 engine
//! would mean the `libxml` crate (bindings to the system libxml2). The CI
//! builds the app on macOS, Ubuntu and Windows with `tauri-action`, and
//! Windows has no system libxml2, so that dependency would break the Windows
//! build — unverifiable from here. This module implements the subset of XPath
//! that covers the expressions SEO users actually write for custom
//! extraction, in pure Rust, with no new dependencies:
//!
//! - Axes: child (`/`) and descendant (`//`).
//! - Node tests: tag name, `*`, `text()`, `@attr`, `@*`.
//! - Predicates: `[@attr='v']`, `[@attr]`, `[n]` (1-based, per parent, as in
//!   XPath), `[contains(@attr,'v')]`, `[contains(text(),'v')]`.
//! - Top-level functions: `count(...)`, `string(...)`.
//!
//! Anything outside the subset (other axes, `|`, `position()`, `last()`,
//! comparisons, arithmetic) is rejected by [`validate`] with a plain error,
//! so the user gets a clear message instead of a silently wrong result.

use ego_tree::NodeRef;
use scraper::{ElementRef, Html, Node};

/// A node produced by an XPath evaluation.
#[derive(Debug, Clone)]
pub enum XNode<'a> {
    Element(ElementRef<'a>),
    Text(String),
    Attr(String),
}

impl<'a> XNode<'a> {
    /// The XPath string-value of the node: an element's full text content,
    /// a text node itself, or an attribute value.
    pub fn string_value(&self) -> String {
        match self {
            XNode::Element(el) => el.text().collect::<String>(),
            XNode::Text(t) => t.clone(),
            XNode::Attr(v) => v.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Axis {
    Child,
    Descendant,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum NodeTest {
    Tag(String),
    AnyElement,
    Text,
    Attr(String),
    AttrAny,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Predicate {
    AttrEq(String, String),
    AttrExists(String),
    Position(usize),
    ContainsAttr(String, String),
    ContainsText(String),
}

#[derive(Debug, Clone)]
struct Step {
    axis: Axis,
    test: NodeTest,
    predicates: Vec<Predicate>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TopFunc {
    Count,
    StringValue,
}

#[derive(Debug)]
struct XPathExpr {
    func: Option<TopFunc>,
    steps: Vec<Step>,
}

/// Parses and validates an expression. `Ok` means the expression is inside
/// the supported subset; the error is a short human-readable message.
pub fn validate(expr: &str) -> Result<(), String> {
    parse(expr).map(|_| ())
}

/// Evaluates a path expression (no top-level function) and returns the
/// matched nodes in document order.
pub fn eval<'a>(doc: &'a Html, expr: &str) -> Result<Vec<XNode<'a>>, String> {
    let parsed = parse(expr)?;
    if parsed.func.is_some() {
        return Err("count() and string() are only for count/text extraction".into());
    }
    eval_steps(doc, &parsed.steps)
}

/// Evaluates `count(path)` — or a plain path — and returns the number of
/// matched nodes.
pub fn eval_count(doc: &Html, expr: &str) -> Result<usize, String> {
    let parsed = parse(expr)?;
    match parsed.func {
        Some(TopFunc::Count) => Ok(eval_steps(doc, &parsed.steps)?.len()),
        Some(TopFunc::StringValue) => Err("string() cannot be used for a count".into()),
        None => Ok(eval_steps(doc, &parsed.steps)?.len()),
    }
}

/// Evaluates `string(path)` — or a plain path — and returns the string-value
/// of the first matched node in document order ("" when nothing matches).
pub fn eval_string(doc: &Html, expr: &str) -> Result<String, String> {
    let parsed = parse(expr)?;
    match parsed.func {
        Some(TopFunc::StringValue) => Ok(eval_steps(doc, &parsed.steps)?
            .into_iter()
            .next()
            .map(|n| n.string_value())
            .unwrap_or_default()),
        Some(TopFunc::Count) => Err("count() cannot be used for text".into()),
        None => Ok(eval_steps(doc, &parsed.steps)?
            .into_iter()
            .next()
            .map(|n| n.string_value())
            .unwrap_or_default()),
    }
}

// ---------------------------------------------------------------------------
// Parser
// ---------------------------------------------------------------------------

struct Parser {
    src: Vec<char>,
    pos: usize,
}

impl Parser {
    fn new(expr: &str) -> Self {
        Parser {
            src: expr.chars().collect(),
            pos: 0,
        }
    }

    fn peek(&self) -> Option<char> {
        self.src.get(self.pos).copied()
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.peek();
        if c.is_some() {
            self.pos += 1;
        }
        c
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(c) if c.is_whitespace()) {
            self.pos += 1;
        }
    }

    fn expect(&mut self, c: char) -> Result<(), String> {
        self.skip_ws();
        if self.bump() == Some(c) {
            Ok(())
        } else {
            Err(format!("expected '{c}' in XPath expression"))
        }
    }

    fn parse_ident(&mut self) -> Result<String, String> {
        self.skip_ws();
        let start = self.pos;
        match self.peek() {
            Some(c) if c.is_ascii_alphabetic() || c == '_' => {
                self.pos += 1;
            }
            _ => return Err("expected a name in XPath expression".into()),
        }
        while matches!(self.peek(), Some(c) if c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.')
        {
            self.pos += 1;
        }
        Ok(self.src[start..self.pos].iter().collect())
    }

    fn parse_string(&mut self) -> Result<String, String> {
        self.skip_ws();
        let quote = match self.peek() {
            Some('\'') => '\'',
            Some('"') => '"',
            _ => return Err("expected a quoted string in XPath expression".into()),
        };
        self.pos += 1;
        let start = self.pos;
        while let Some(c) = self.peek() {
            if c == quote {
                let s: String = self.src[start..self.pos].iter().collect();
                self.pos += 1;
                return Ok(s);
            }
            self.pos += 1;
        }
        Err("unterminated string in XPath expression".into())
    }

    fn parse_number(&mut self) -> Result<usize, String> {
        self.skip_ws();
        let start = self.pos;
        while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
            self.pos += 1;
        }
        if self.pos == start {
            return Err("expected a number in XPath predicate".into());
        }
        let s: String = self.src[start..self.pos].iter().collect();
        let n: usize = s
            .parse()
            .map_err(|_| "number too large in XPath predicate".to_string())?;
        if n == 0 {
            return Err("XPath positions are 1-based; [0] never matches".into());
        }
        Ok(n)
    }
}

fn parse(expr: &str) -> Result<XPathExpr, String> {
    let trimmed = expr.trim();
    if trimmed.is_empty() {
        return Err("empty XPath expression".into());
    }
    let mut p = Parser::new(trimmed);

    // Top-level function: count(...) / string(...).
    let mut func = None;
    if let Some(name) = try_parse_func_name(&mut p)? {
        func = Some(match name.as_str() {
            "count" => TopFunc::Count,
            "string" => TopFunc::StringValue,
            _ => return Err(format!("unsupported XPath function '{name}'")),
        });
        p.expect('(')?;
        let inner = parse_path(&mut p)?;
        p.expect(')')?;
        p.skip_ws();
        if p.peek().is_some() {
            return Err("unexpected trailing characters in XPath expression".into());
        }
        return Ok(XPathExpr { func, steps: inner });
    }

    let steps = parse_path(&mut p)?;
    p.skip_ws();
    if p.peek().is_some() {
        return Err("unexpected trailing characters in XPath expression".into());
    }
    Ok(XPathExpr { func, steps })
}

/// If the parser is at `count(` or `string(`, consumes the name and returns
/// it. Otherwise leaves the parser untouched and returns `None`.
fn try_parse_func_name(p: &mut Parser) -> Result<Option<String>, String> {
    let save = p.pos;
    p.skip_ws();
    if !matches!(p.peek(), Some(c) if c.is_ascii_alphabetic()) {
        p.pos = save;
        return Ok(None);
    }
    let name = p.parse_ident()?;
    let mut q = p.pos;
    while matches!(p.src.get(q), Some(c) if c.is_whitespace()) {
        q += 1;
    }
    if p.src.get(q) == Some(&'(') && (name == "count" || name == "string") {
        Ok(Some(name))
    } else if p.src.get(q) == Some(&'(') {
        // Some other function call: unsupported, say so plainly.
        Err(format!("unsupported XPath function '{name}'"))
    } else {
        p.pos = save;
        Ok(None)
    }
}

fn parse_path(p: &mut Parser) -> Result<Vec<Step>, String> {
    let mut steps = Vec::new();
    // A path starts with '/' or '//' (absolute) or with a bare step
    // (relative, evaluated from the document root).
    let mut axis = Axis::Child;
    p.skip_ws();
    if p.peek() == Some('/') {
        p.bump();
        if p.peek() == Some('/') {
            p.bump();
            axis = Axis::Descendant;
        }
        steps.push(parse_step(p, axis)?);
    } else {
        steps.push(parse_step(p, Axis::Child)?);
    }

    loop {
        p.skip_ws();
        match p.peek() {
            Some('/') => {
                p.bump();
                let ax = if p.peek() == Some('/') {
                    p.bump();
                    Axis::Descendant
                } else {
                    Axis::Child
                };
                steps.push(parse_step(p, ax)?);
            }
            _ => break,
        }
    }

    // text() and @attr only make sense as the last step: nothing below a
    // text node or an attribute can be selected.
    for step in &steps[..steps.len().saturating_sub(1)] {
        if matches!(
            step.test,
            NodeTest::Text | NodeTest::Attr(_) | NodeTest::AttrAny
        ) {
            return Err("text() and @attr must be the last step of an XPath".into());
        }
    }
    Ok(steps)
}

fn parse_step(p: &mut Parser, axis: Axis) -> Result<Step, String> {
    p.skip_ws();
    let test = match p.peek() {
        Some('*') => {
            p.bump();
            NodeTest::AnyElement
        }
        Some('@') => {
            p.bump();
            match p.peek() {
                Some('*') => {
                    p.bump();
                    NodeTest::AttrAny
                }
                _ => NodeTest::Attr(p.parse_ident()?.to_ascii_lowercase()),
            }
        }
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {
            let name = p.parse_ident()?;
            p.skip_ws();
            if name == "text" && p.peek() == Some('(') {
                p.bump();
                p.expect(')')?;
                NodeTest::Text
            } else if p.peek() == Some('(') {
                return Err(format!("unsupported XPath function '{name}'"));
            } else {
                NodeTest::Tag(name.to_ascii_lowercase())
            }
        }
        _ => return Err("expected a step (tag, *, text() or @attr) in XPath".into()),
    };

    let mut predicates = Vec::new();
    loop {
        p.skip_ws();
        if p.peek() != Some('[') {
            break;
        }
        p.bump();
        predicates.push(parse_predicate(p)?);
        p.expect(']')?;
    }

    Ok(Step {
        axis,
        test,
        predicates,
    })
}

fn parse_predicate(p: &mut Parser) -> Result<Predicate, String> {
    p.skip_ws();
    match p.peek() {
        Some('@') => {
            p.bump();
            let attr = p.parse_ident()?.to_ascii_lowercase();
            p.skip_ws();
            if p.peek() == Some('=') {
                p.bump();
                let value = p.parse_string()?;
                Ok(Predicate::AttrEq(attr, value))
            } else {
                Ok(Predicate::AttrExists(attr))
            }
        }
        Some(c) if c.is_ascii_digit() => Ok(Predicate::Position(p.parse_number()?)),
        Some(c) if c.is_ascii_alphabetic() => {
            let name = p.parse_ident()?;
            if name != "contains" {
                return Err(format!("unsupported XPath function '{name}' in predicate"));
            }
            p.expect('(')?;
            p.skip_ws();
            let (is_text, attr) = match p.peek() {
                Some('@') => {
                    p.bump();
                    (false, Some(p.parse_ident()?.to_ascii_lowercase()))
                }
                _ => {
                    let t = p.parse_ident()?;
                    if t != "text" {
                        return Err("contains() needs @attr or text()".into());
                    }
                    p.expect('(')?;
                    p.expect(')')?;
                    (true, None)
                }
            };
            p.expect(',')?;
            let needle = p.parse_string()?;
            p.expect(')')?;
            if is_text {
                Ok(Predicate::ContainsText(needle))
            } else {
                Ok(Predicate::ContainsAttr(attr.unwrap(), needle))
            }
        }
        _ => Err("unsupported XPath predicate".into()),
    }
}

// ---------------------------------------------------------------------------
// Evaluation
// ---------------------------------------------------------------------------

/// One candidate node during evaluation, kept with its parent so the
/// position predicate `[n]` can be applied per parent, as XPath does.
enum Cand<'a> {
    Element(ElementRef<'a>),
    Text(NodeRef<'a, Node>, String),
    Attr(ElementRef<'a>, String, String),
}

impl<'a> Cand<'a> {
    /// The parent node id used to group candidates for `[n]`.
    fn parent_key(&self) -> Option<ego_tree::NodeId> {
        match self {
            Cand::Element(el) => el.parent().map(|n| n.id()),
            Cand::Text(node, _) => node.parent().map(|n| n.id()),
            Cand::Attr(owner, _, _) => Some(owner.id()),
        }
    }

    fn into_xnode(self) -> XNode<'a> {
        match self {
            Cand::Element(el) => XNode::Element(el),
            Cand::Text(_, t) => XNode::Text(t),
            Cand::Attr(_, _, v) => XNode::Attr(v),
        }
    }
}

fn eval_steps<'a>(doc: &'a Html, steps: &[Step]) -> Result<Vec<XNode<'a>>, String> {
    // The context starts at the document node; a leading '/' selects its
    // children, a leading '//' its descendants.
    let mut context: Vec<NodeRef<'a, Node>> = vec![doc.tree.root()];
    let mut last: Vec<Cand<'a>> = Vec::new();

    for step in steps {
        let mut cands: Vec<Cand<'a>> = Vec::new();
        for ctx in &context {
            collect_step(*ctx, step, &mut cands);
        }
        // XPath node-sets are sets: when context nodes nest (e.g. `//div//p`
        // where one div contains another), the same node is reached twice.
        // Drop duplicates, keeping document order.
        dedupe_cands(&mut cands);
        apply_predicates(step, &mut cands);
        // The next step continues from the element candidates only.
        context = cands
            .iter()
            .filter_map(|c| match c {
                Cand::Element(el) => Some(**el),
                _ => None,
            })
            .collect();
        last = cands;
    }

    Ok(last.into_iter().map(|c| c.into_xnode()).collect())
}

fn collect_step<'a>(ctx: NodeRef<'a, Node>, step: &Step, out: &mut Vec<Cand<'a>>) {
    match &step.test {
        NodeTest::Tag(_) | NodeTest::AnyElement => {
            let want = match &step.test {
                NodeTest::Tag(n) => Some(n.as_str()),
                _ => None,
            };
            let elems: Vec<ElementRef<'a>> = match step.axis {
                Axis::Child => ctx
                    .children()
                    .filter_map(ElementRef::wrap)
                    .filter(|el| want.is_none_or(|n| el.value().name() == n))
                    .collect(),
                Axis::Descendant => ctx
                    .descendants()
                    .filter_map(ElementRef::wrap)
                    .filter(|el| want.is_none_or(|n| el.value().name() == n))
                    .collect(),
            };
            out.extend(elems.into_iter().map(Cand::Element));
        }
        NodeTest::Text => {
            let texts: Vec<(NodeRef<'a, Node>, String)> = match step.axis {
                Axis::Child => ctx
                    .children()
                    .filter_map(|n| n.value().as_text().map(|t| (n, t.text.to_string())))
                    .collect(),
                Axis::Descendant => ctx
                    .descendants()
                    .filter_map(|n| n.value().as_text().map(|t| (n, t.text.to_string())))
                    .collect(),
            };
            out.extend(texts.into_iter().map(|(n, t)| Cand::Text(n, t)));
        }
        NodeTest::Attr(_) | NodeTest::AttrAny => {
            let want = match &step.test {
                NodeTest::Attr(n) => Some(n.as_str()),
                _ => None,
            };
            // The attribute axis reads the context element's own attributes;
            // with `//` it reads every descendant element's attributes too.
            let owners: Vec<ElementRef<'a>> = match step.axis {
                Axis::Child => ElementRef::wrap(ctx).into_iter().collect(),
                Axis::Descendant => {
                    let mut v: Vec<ElementRef<'a>> = ElementRef::wrap(ctx).into_iter().collect();
                    v.extend(ctx.descendants().filter_map(ElementRef::wrap));
                    v
                }
            };
            for owner in owners {
                for (attr_name, value) in owner.value().attrs() {
                    if want.is_none_or(|w| attr_name == w) {
                        out.push(Cand::Attr(owner, attr_name.to_string(), value.to_string()));
                    }
                }
            }
        }
    }
}

/// Drops duplicate candidates, keeping the first occurrence in document
/// order. Needed because a descendant step from nested context nodes reaches
/// the same node through more than one context node.
fn dedupe_cands(cands: &mut Vec<Cand<'_>>) {
    use std::collections::HashSet;
    #[derive(Hash, PartialEq, Eq)]
    enum Key {
        Element(ego_tree::NodeId),
        Text(ego_tree::NodeId),
        Attr(ego_tree::NodeId, String),
    }
    let mut seen: HashSet<Key> = HashSet::new();
    cands.retain(|c| {
        let key = match c {
            Cand::Element(el) => Key::Element(el.id()),
            Cand::Text(node, _) => Key::Text(node.id()),
            Cand::Attr(owner, name, _) => Key::Attr(owner.id(), name.clone()),
        };
        seen.insert(key)
    });
}

fn apply_predicates(step: &Step, cands: &mut Vec<Cand<'_>>) {
    for pred in &step.predicates {
        match pred {
            Predicate::AttrEq(attr, value) => {
                cands.retain(|c| match c {
                    Cand::Element(el) => el.attr(attr).is_some_and(|v| v == value),
                    Cand::Attr(_, name, v) => name == attr && v == value,
                    Cand::Text(_, _) => false,
                });
            }
            Predicate::AttrExists(attr) => {
                cands.retain(|c| match c {
                    Cand::Element(el) => el.attr(attr).is_some(),
                    Cand::Attr(_, _, _) => true,
                    Cand::Text(_, _) => false,
                });
            }
            Predicate::Position(n) => {
                // XPath `[n]` is the n-th matching child of each parent, not
                // the n-th of the whole node-set. Count per parent and keep
                // the n-th of each group; candidates are already in document
                // order, so `keep` stays ordered.
                let mut counts: std::collections::HashMap<Option<ego_tree::NodeId>, usize> =
                    std::collections::HashMap::new();
                let mut keep: Vec<usize> = Vec::new();
                for (idx, c) in cands.iter().enumerate() {
                    let count = counts.entry(c.parent_key()).or_insert(0);
                    *count += 1;
                    if *count == *n {
                        keep.push(idx);
                    }
                }
                let old = std::mem::take(cands);
                *cands = old
                    .into_iter()
                    .enumerate()
                    .filter(|(idx, _)| keep.contains(idx))
                    .map(|(_, c)| c)
                    .collect();
            }
            Predicate::ContainsAttr(attr, needle) => {
                cands.retain(|c| match c {
                    Cand::Element(el) => el.attr(attr).is_some_and(|v| v.contains(needle.as_str())),
                    Cand::Attr(_, name, v) => name == attr && v.contains(needle.as_str()),
                    Cand::Text(_, _) => false,
                });
            }
            Predicate::ContainsText(needle) => {
                cands.retain(|c| match c {
                    Cand::Element(el) => el.text().collect::<String>().contains(needle.as_str()),
                    Cand::Text(_, t) => t.contains(needle.as_str()),
                    Cand::Attr(_, _, _) => false,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doc(html: &str) -> Html {
        Html::parse_document(html)
    }

    #[test]
    fn eval_tag_and_descendant() {
        let d = doc("<html><body><h1>Title</h1><p>one</p><p>two</p></body></html>");
        let nodes = eval(&d, "//h1").unwrap();
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].string_value(), "Title");
        let ps = eval(&d, "//p").unwrap();
        assert_eq!(ps.len(), 2);
        let first = eval(&d, "/html/body/p[1]").unwrap();
        assert_eq!(first.len(), 1);
        assert_eq!(first[0].string_value(), "one");
    }

    #[test]
    fn eval_attr_and_text_steps() {
        let d = doc(
            "<html><head><meta name=\"description\" content=\"Desc here\">\
             <link rel=\"canonical\" href=\"https://x.com/c\"></head>\
             <body><h1>Hi</h1></body></html>",
        );
        let meta = eval(&d, "//meta[@name='description']/@content").unwrap();
        assert_eq!(meta.len(), 1);
        assert_eq!(meta[0].string_value(), "Desc here");
        let canon = eval(&d, "//link[@rel=\"canonical\"]/@href").unwrap();
        assert_eq!(canon[0].string_value(), "https://x.com/c");
        let h1_text = eval(&d, "//h1/text()").unwrap();
        assert_eq!(h1_text[0].string_value(), "Hi");
    }

    #[test]
    fn eval_position_is_per_parent() {
        let d = doc("<html><body><div><p>a</p><p>b</p></div><div><p>c</p></div></body></html>");
        // [1] = first p of each parent div.
        let firsts = eval(&d, "//div/p[1]").unwrap();
        assert_eq!(firsts.len(), 2);
        assert_eq!(firsts[0].string_value(), "a");
        assert_eq!(firsts[1].string_value(), "c");
        // (//p)[1] is not in the subset; //p[1] must still be per-parent.
        let all_first = eval(&d, "//p[1]").unwrap();
        assert_eq!(all_first.len(), 2);
    }

    #[test]
    fn eval_contains_and_count_string() {
        let d = doc(
            "<html><body><a href=\"/a\">x</a><a href=\"/b\" class=\"nav item\">y</a>\
             <a href=\"/c\">z</a></body></html>",
        );
        assert_eq!(eval_count(&d, "count(//a)").unwrap(), 3);
        assert_eq!(eval_count(&d, "//a").unwrap(), 3);
        assert_eq!(
            eval_string(&d, "string(//a[@class='nav item'])").unwrap(),
            "y"
        );
        let nav = eval(&d, "//a[contains(@class,'nav')]").unwrap();
        assert_eq!(nav.len(), 1);
        assert_eq!(eval_string(&d, "//a[contains(text(),'z')]").unwrap(), "z");
        assert_eq!(eval_string(&d, "//h9").unwrap(), "");
    }

    #[test]
    fn validate_rejects_out_of_subset() {
        for bad in [
            "//div/parent::span",
            "//a | //b",
            "//p[position()=1]",
            "//p[last()]",
            "//div[@x=1]",
            "//div[0]",
            "//h1/text()/span",
            "count(//a) + 1",
            "",
            "//",
        ] {
            assert!(validate(bad).is_err(), "{bad} should be rejected");
        }
        for good in [
            "//h1",
            "/html/body/h1",
            "//meta[@name='description']/@content",
            "//a[contains(@href,'x')]",
            "//li[2]",
            "count(//img)",
            "string(//title)",
            "//div[@data-x]",
            "//*[@id='main']",
        ] {
            assert!(validate(good).is_ok(), "{good} should be accepted");
        }
    }
}
