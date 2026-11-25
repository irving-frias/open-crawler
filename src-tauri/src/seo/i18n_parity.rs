//! Cross-language i18n parity tests.
//!
//! These tests are the regression guard for the most common mistake in this
//! codebase: adding a check in Rust (or a semantic issue type in the parser)
//! without adding its translation to the frontend. Missing entries do not fail
//! the build on their own — the UI silently falls back to the raw English
//! message or to a humanized id — so they are caught here instead.
//!
//! The frontend dictionary lives in TypeScript/JSON, so the source files are
//! embedded at compile time with `include_str!` and parsed with a tiny
//! line-based scanner. If the shape of those files changes, these tests fail
//! with an explicit "could not locate ..." message: update the scanner, do not
//! delete the test.

use std::collections::{BTreeMap, BTreeSet};

const SEO_CHECKS_TS: &str = include_str!("../../../src/lib/seo-checks.ts");
const I18N_ISSUES_TS: &str = include_str!("../../../src/lib/i18n-issues.ts");
const CHECKS_RS: &str = include_str!("checks.rs");
const PARSER_RS: &str = include_str!("../crawler/parser.rs");
const MESSAGES_EN: &str = include_str!("../../../messages/en.json");
const MESSAGES_ES: &str = include_str!("../../../messages/es.json");

/// Drops the `#[cfg(test)]` module so fixtures and assertions are not scanned.
fn without_tests(src: &str) -> &str {
    src.split("#[cfg(test)]").next().unwrap_or(src)
}

/// Collects the string literals passed as the first argument of every
/// `func("literal", ...)` call, ignoring longer identifiers (`has_issue(`) and
/// calls whose first argument is not a literal (`check(&out, ...)`).
fn scan_first_arg(src: &str, func: &str) -> BTreeSet<String> {
    let needle = format!("{func}(");
    let bytes = src.as_bytes();
    let mut out = BTreeSet::new();
    for (idx, _) in src.match_indices(&needle) {
        if idx > 0 {
            let prev = bytes[idx - 1] as char;
            if prev.is_alphanumeric() || prev == '_' {
                continue;
            }
        }
        let mut i = idx + needle.len();
        while i < bytes.len() && (bytes[i] as char).is_whitespace() {
            i += 1;
        }
        if i >= bytes.len() || bytes[i] != b'"' {
            continue;
        }
        let start = i + 1;
        let Some(end) = src[start..].find('"') else {
            continue;
        };
        let id = &src[start..start + end];
        if !id.is_empty()
            && id
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        {
            out.insert(id.to_string());
        }
    }
    out
}

/// Collects `field: "literal"` assignments (struct literals such as
/// `SemanticIssue { issue_type: "missing_title", .. }`).
fn scan_field_literals(src: &str, field: &str) -> BTreeSet<String> {
    let needle = format!("{field}:");
    let mut out = BTreeSet::new();
    for (idx, _) in src.match_indices(&needle) {
        let after = &src[idx + needle.len()..];
        let Some(rest) = after.trim_start().strip_prefix('"') else {
            continue;
        };
        let Some(end) = rest.find('"') else {
            continue;
        };
        let id = &rest[..end];
        if !id.is_empty()
            && id
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        {
            out.insert(id.to_string());
        }
    }
    out
}

/// Parses a `const NAME: ... = {` block and returns the keys declared one per
/// line at the given indentation, e.g. `  title_present: {`.
fn ts_record_keys(src: &str, header: &str) -> BTreeSet<String> {
    let start = src
        .find(header)
        .unwrap_or_else(|| panic!("could not locate TS record header: {header}"));
    let mut keys = BTreeSet::new();
    for line in src[start + header.len()..].lines().skip(1) {
        if line.trim() == "};" {
            return keys;
        }
        let trimmed = line.trim_start();
        if line.len() - trimmed.len() != 2 || !trimmed.contains(": ") {
            continue;
        }
        let key = trimmed.split(':').next().unwrap_or("").trim();
        if !key.is_empty()
            && key
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        {
            keys.insert(key.to_string());
        }
    }
    panic!("TS record block did not terminate with '}};': {header}");
}

/// Parses the `issueNames` / `issueMessages` lookups (`  id: (p) => m['issue.…']`).
fn ts_message_map_keys(src: &str, header: &str, suffix: &str) -> BTreeSet<String> {
    let start = src
        .find(header)
        .unwrap_or_else(|| panic!("could not locate TS map header: {header}"));
    let mut keys = BTreeSet::new();
    for line in src[start + header.len()..].lines().skip(1) {
        if line.trim() == "};" {
            return keys;
        }
        let trimmed = line.trim_start();
        if line.len() - trimmed.len() != 2 || !trimmed.contains("m['issue.") {
            continue;
        }
        let key = trimmed.split(':').next().unwrap_or("").trim();
        let is_wanted = trimmed.contains(&format!(".message']")) == (suffix == "message");
        if is_wanted
            && !key.is_empty()
            && key
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        {
            keys.insert(key.to_string());
        }
    }
    panic!("TS map block did not terminate with '}};': {header}");
}

fn check_ids_in_rust() -> BTreeSet<String> {
    scan_first_arg(without_tests(CHECKS_RS), "check")
}

fn issue_types_in_rust() -> BTreeSet<String> {
    let src = without_tests(PARSER_RS);
    let mut types = scan_first_arg(src, "issue");
    types.extend(scan_field_literals(src, "issue_type"));
    types
}

fn message_keys(json: &str) -> BTreeMap<String, String> {
    serde_json::from_str(json).expect("messages JSON must be valid")
}

/// Pretty diff of two sets, used in the assertion messages.
fn diff(left: &BTreeSet<String>, right: &BTreeSet<String>) -> String {
    let only_left: Vec<_> = left.difference(right).cloned().collect();
    let only_right: Vec<_> = right.difference(left).cloned().collect();
    format!("only in left: {only_left:?}\nonly in right: {only_right:?}")
}

#[test]
fn every_rust_check_id_has_ts_copy() {
    let rust = check_ids_in_rust();
    let dict = ts_record_keys(SEO_CHECKS_TS, "const DICT: Record<string, CheckDictEntry> = {");
    assert!(!rust.is_empty(), "no check ids found in checks.rs");
    assert!(!dict.is_empty(), "no DICT keys found in seo-checks.ts");
    assert!(
        rust.is_subset(&dict),
        "checks emitted in Rust without a DICT entry in src/lib/seo-checks.ts:\n{}",
        diff(&rust, &dict)
    );
}

#[test]
fn no_orphan_ts_check_copy() {
    let rust = check_ids_in_rust();
    let dict = ts_record_keys(SEO_CHECKS_TS, "const DICT: Record<string, CheckDictEntry> = {");
    assert!(
        dict.is_subset(&rust),
        "DICT entries in src/lib/seo-checks.ts that no check emits:\n{}",
        diff(&dict, &rust)
    );
}

#[test]
fn check_copy_is_complete_in_both_languages() {
    let dict = ts_record_keys(SEO_CHECKS_TS, "const DICT: Record<string, CheckDictEntry> = {");
    let why = ts_record_keys(SEO_CHECKS_TS, "const WHY: Record<string, { en: string; es: string }> = {");
    let fixes = ts_record_keys(SEO_CHECKS_TS, "const CHECK_FIXES: Record<string, CheckFixEntry> = {");
    assert_eq!(dict, why, "DICT and WHY must list the same ids\n{}", diff(&dict, &why));
    assert_eq!(
        dict,
        fixes,
        "DICT and CHECK_FIXES must list the same ids\n{}",
        diff(&dict, &fixes)
    );
}

#[test]
fn every_parser_issue_type_is_translated() {
    let rust = issue_types_in_rust();
    let names = ts_message_map_keys(I18N_ISSUES_TS, "const issueNames:", "name");
    let messages = ts_message_map_keys(I18N_ISSUES_TS, "const issueMessages:", "message");
    let fixes = ts_record_keys(I18N_ISSUES_TS, "const ISSUE_FIXES");
    assert!(!rust.is_empty(), "no issue types found in parser.rs");
    assert!(
        rust.is_subset(&names),
        "issue types emitted by the parser without an entry in issueNames \
         (src/lib/i18n-issues.ts):\n{}",
        diff(&rust, &names)
    );
    assert!(
        rust.is_subset(&messages),
        "issue types emitted by the parser without an entry in issueMessages:\n{}",
        diff(&rust, &messages)
    );
    assert_eq!(
        names,
        fixes,
        "issueNames and ISSUE_FIXES must list the same ids\n{}",
        diff(&names, &fixes)
    );
    // Both message maps must cover exactly the same ids.
    assert_eq!(
        names,
        messages,
        "issueNames and issueMessages must list the same ids\n{}",
        diff(&names, &messages)
    );
}

#[test]
fn message_files_cover_every_issue_type_in_both_locales() {
    let en = message_keys(MESSAGES_EN);
    let es = message_keys(MESSAGES_ES);
    let mut missing_in_es: Vec<_> = en.keys().filter(|k| !es.contains_key(*k)).collect();
    let mut missing_in_en: Vec<_> = es.keys().filter(|k| !en.contains_key(*k)).collect();
    missing_in_es.sort();
    missing_in_en.sort();
    assert!(
        missing_in_es.is_empty() && missing_in_en.is_empty(),
        "en/es message keys differ:\nmissing in es: {missing_in_es:?}\nmissing in en: {missing_in_en:?}"
    );

    let rust = issue_types_in_rust();
    let mut problems = Vec::new();
    for id in &rust {
        for locale in [("en", &en), ("es", &es)] {
            for suffix in ["name", "message"] {
                let key = format!("issue.{id}.{suffix}");
                if !locale.1.contains_key(&key) {
                    problems.push(format!("{key} missing in messages/{}.json", locale.0));
                }
            }
        }
    }
    problems.sort();
    assert!(
        problems.is_empty(),
        "parser issue types without messages entries:\n{}",
        problems.join("\n")
    );
}

#[test]
fn seo_category_labels_exist_for_every_scored_category() {
    use crate::seo::score::CATEGORY_ORDER;

    let en = message_keys(MESSAGES_EN);
    let es = message_keys(MESSAGES_ES);
    let mut problems = Vec::new();
    for category in CATEGORY_ORDER {
        let key = format!("seo.category.{category}");
        if !en.contains_key(&key) {
            problems.push(format!("{key} missing in messages/en.json"));
        }
        if !es.contains_key(&key) {
            problems.push(format!("{key} missing in messages/es.json"));
        }
    }
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}
