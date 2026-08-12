use std::path::Path;
use std::sync::Arc;

use tauri::{AppHandle, Emitter, Manager, State};
use tokio::sync::RwLock;
use tracing::warn;

use crate::crawler::parser::SemanticIssue;
use crate::error::AppError;
use crate::features::{with_repo, with_repo_arc};
use crate::models::CrawlResult;
use crate::seo::audit::SeoAuditResult;
use crate::seo::score::CATEGORY_ORDER;
use crate::AppState;

const PAGE_BATCH_SIZE: u32 = 1000;
const LINK_BATCH_SIZE: u32 = 5000;

/// Progress callback shared by the writers. The export command wires it to
/// `emit_export_progress`; unit tests pass a recorder instead, so the writers
/// stay testable without a running Tauri app.
type EmitProgress = dyn Fn(&'static str, u64, u64) + Sync + 'static;

#[derive(serde::Serialize, Clone)]
struct ExportProgress {
    stage: &'static str,
    processed: u64,
    total: u64,
    percent: f32,
}

fn emit_export_progress(app: &AppHandle, stage: &'static str, processed: u64, total: u64) {
    let percent = if total == 0 {
        100.0
    } else {
        (processed as f32 / total as f32) * 100.0
    };
    let _ = app.emit(
        "export-progress",
        ExportProgress {
            stage,
            processed,
            total,
            percent,
        },
    );
}

#[tauri::command]
pub async fn export_full(
    app: AppHandle,
    state: State<'_, Arc<RwLock<AppState>>>,
    project_id: String,
    file_path: String,
    format: String,
) -> Result<(), AppError> {
    let pid = project_id.clone();
    let (total_pages, total_links, total_issues, total_seo) = with_repo(&state, move |repo| {
        let total_pages = repo.count_pages(&pid)?;
        let total_links = repo.count_links(&pid)?;
        let total_issues = repo.count_issues(&pid)?;
        let total_seo = repo.count_seo_rows(&pid)?;
        Ok((total_pages, total_links, total_issues, total_seo))
    })
    .await?;

    if total_pages > 100_000 {
        warn!(
            "Export requested for {} pages (over 100K). File may be very large.",
            total_pages
        );
    }

    let (write_path, share_after) = export_target(&app, &file_path, &format)?;

    // SAF content URIs (chosen via the save dialog on Android) are not writable
    // with std::fs, so the export is streamed to a temp file first and then
    // copied into the user-chosen document through the fs plugin's mobile bridge.
    let content_uri = if cfg!(mobile) && write_path.starts_with("content://") {
        Some(write_path.clone())
    } else {
        None
    };
    let tmp_path: Option<std::path::PathBuf> = if content_uri.is_some() {
        let ext = if format == "xlsx" { "xlsx" } else { "csv" };
        let dir = app
            .path()
            .app_data_dir()
            .map_err(|e| AppError::Crawl(e.to_string()))?
            .join("exports");
        std::fs::create_dir_all(&dir)?;
        Some(dir.join(format!("export-tmp-{}.{ext}", std::process::id())))
    } else {
        None
    };

    let dest_path = tmp_path
        .as_ref()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|| write_path.clone());

    let app_emit = app.clone();
    let emit = move |stage: &'static str, processed: u64, total: u64| {
        emit_export_progress(&app_emit, stage, processed, total);
    };

    if format == "xlsx" {
        export_xlsx(
            state.inner(),
            &emit,
            &project_id,
            &dest_path,
            total_pages,
            total_links,
            total_issues,
            total_seo,
        )
        .await?;
    } else {
        export_csv_single(state.inner(), &emit, &project_id, &dest_path, total_pages).await?;
    }

    if let (Some(uri), Some(tmp)) = (content_uri, tmp_path) {
        copy_to_content_uri(&app, &uri, &tmp)?;
        let _ = std::fs::remove_file(&tmp);
    }

    if share_after {
        share_export(&app, &write_path, &format);
    }

    Ok(())
}

/// Copies a file into an Android `content://` URI (SAF) by resolving the URI to
/// a writable file descriptor through the fs plugin's mobile bridge.
#[cfg(target_os = "android")]
pub(crate) fn copy_to_content_uri(
    app: &AppHandle,
    uri: &str,
    src: &std::path::Path,
) -> Result<(), AppError> {
    use std::io::Write;

    use tauri_plugin_fs::{FilePath, FsExt, OpenOptions};

    let mut src_file = std::fs::File::open(src)?;
    let mut opts = OpenOptions::new();
    opts.write(true).truncate(true).create(true);
    let mut dst_file = app
        .fs()
        .open(FilePath::Url(url::Url::parse(uri)?), opts)
        .map_err(|e| AppError::Crawl(format!("failed to open content URI: {e}")))?;
    std::io::copy(&mut src_file, &mut dst_file)?;
    dst_file.flush()?;
    Ok(())
}

#[cfg(not(target_os = "android"))]
pub(crate) fn copy_to_content_uri(
    _app: &AppHandle,
    _uri: &str,
    _src: &std::path::Path,
) -> Result<(), AppError> {
    Ok(())
}

/// Copies a file from an Android `content://` URI (SAF picker result) into the
/// app data dir so it can be read with plain `std::fs`.
#[cfg(target_os = "android")]
pub(crate) fn copy_from_content_uri(
    app: &AppHandle,
    uri: &str,
    dst: &std::path::Path,
) -> Result<(), AppError> {
    use tauri_plugin_fs::{FilePath, FsExt, OpenOptions};

    if let Some(parent) = dst.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut opts = OpenOptions::new();
    opts.read(true);
    let mut src_file = app
        .fs()
        .open(FilePath::Url(url::Url::parse(uri)?), opts)
        .map_err(|e| AppError::Crawl(format!("failed to open content URI: {e}")))?;
    let mut dst_file = std::fs::File::create(dst)?;
    std::io::copy(&mut src_file, &mut dst_file)?;
    Ok(())
}

#[cfg(not(target_os = "android"))]
pub(crate) fn copy_from_content_uri(
    _app: &AppHandle,
    _uri: &str,
    _dst: &std::path::Path,
) -> Result<(), AppError> {
    Err(AppError::Crawl("content URI import not supported".into()))
}

/// Resolves the destination file for an export.
///
/// On desktop the user picks a path via the save dialog and the file is
/// written straight to it. On mobile the save dialog returns a `content://`
/// URI (SAF) that `std::fs` cannot open, so that URI is passed through and the
/// write is handled via the fs plugin's mobile bridge. A plain filename on
/// mobile falls back to the legacy behavior: write to the app data dir and
/// open the native share sheet afterwards.
fn export_target(
    app: &AppHandle,
    file_path: &str,
    format: &str,
) -> Result<(String, bool), AppError> {
    if !cfg!(mobile) {
        return Ok((file_path.to_string(), false));
    }

    // A SAF content URI chosen by the user via the save dialog: write straight to it.
    if file_path.starts_with("content://") {
        return Ok((file_path.to_string(), false));
    }

    // Legacy fallback: keep the file in the app data dir and open the share sheet.
    let ext = if format == "xlsx" { "xlsx" } else { "csv" };
    let file_name = Path::new(file_path)
        .file_name()
        .and_then(|n| n.to_str())
        .filter(|n| !n.is_empty() && !n.starts_with("content:"))
        .map(str::to_string)
        .unwrap_or_else(|| format!("crawl-results.{ext}"));

    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Crawl(e.to_string()))?
        .join("exports");
    std::fs::create_dir_all(&dir)?;
    let dest = dir.join(&file_name);

    Ok((dest.to_string_lossy().into_owned(), true))
}

#[cfg(mobile)]
fn share_export(app: &AppHandle, path: &str, format: &str) {
    use tauri_plugin_share::ShareExt;

    let mime = if format == "xlsx" {
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
    } else {
        "text/csv"
    };

    let app = app.clone();
    let path = path.to_string();
    let mime = mime.to_string();

    // The Android share plugin never resolves its invoke after launching the
    // share sheet, so run it on a detached thread to avoid blocking.
    std::thread::spawn(move || {
        let _ = app
            .share_file()
            .share_file(tauri_plugin_share::ShareRequest {
                path: Some(path),
                mime: Some(mime),
                group: None,
            });
    });
}

#[cfg(not(mobile))]
fn share_export(_app: &AppHandle, _path: &str, _format: &str) {}

/// A flat cell produced by [`page_values`]. Kept as a small enum so the CSV and
/// XLSX writers can format numbers (e.g. no trailing `.0`) independently.
enum ExportValue {
    Num(f64),
    Str(String),
}

fn parse_audit(seo_audit_json: &Option<String>) -> Option<SeoAuditResult> {
    seo_audit_json
        .as_deref()
        .and_then(|js| serde_json::from_str::<SeoAuditResult>(js).ok())
}

fn category_score(audit: &SeoAuditResult, category: &str) -> Option<f64> {
    audit
        .categories
        .iter()
        .find(|c| c.category == category)
        .map(|c| c.score)
}

fn category_label(category: &str) -> &'static str {
    match category {
        "meta" => "Meta",
        "technical" => "Technical",
        "social" => "Social",
        "accessibility" => "Accessibility",
        "semantic_html" => "Semantic HTML",
        "performance" => "Performance",
        "ai_readability" => "AI Readability",
        "sxo" => "SXO",
        _ => "Other",
    }
}

/// Headers for the flat pages table. Category columns are generated from
/// `CATEGORY_ORDER` so the header order always matches the values written by
/// [`page_values`].
fn page_headers() -> Vec<&'static str> {
    let mut headers = vec![
        "URL",
        "Status Code",
        "Title",
        "Meta Description",
        "H1",
        "Canonical",
        "HTML Lang",
        "Indexable",
        "Depth",
        "Parent URL",
        "Size (bytes)",
        "Load Time (ms)",
        "Semantic Issues",
        "Readability Score",
        "PageSpeed Score",
        "SEO Score",
        "SEO Grade",
        "SEO Priority Fixes",
        "SEO Failed Checks",
        "Blocked",
        "Duplicate Group",
        "Redirect From",
    ];
    for cat in CATEGORY_ORDER {
        headers.push(category_label(cat));
    }
    headers.extend(["Keywords JSON", "OG JSON", "Hreflang JSON"]);
    headers
}

/// Flattens a page into the 33 export columns. SEO values (score, grade,
/// per-category scores, priority fixes, failed checks) are derived from the
/// stored `seo_audit_json` when it parses; pages without an audit fall back to
/// empty cells / zero.
fn page_values(p: &CrawlResult) -> Vec<ExportValue> {
    let audit = parse_audit(&p.seo_audit_json);
    let indexable = match p.is_indexable {
        Some(true) => "Yes",
        Some(false) => "No",
        None => "Unknown",
    };
    let grade = audit.as_ref().map(|a| a.grade.clone()).unwrap_or_default();
    let priority_fixes = audit
        .as_ref()
        .map(|a| a.priority_fixes.len() as f64)
        .unwrap_or(0.0);
    let failed_checks = audit
        .as_ref()
        .map(|a| a.checks.iter().filter(|c| !c.passed).count() as f64)
        .unwrap_or(0.0);

    let mut values = vec![
        ExportValue::Str(p.url.clone()),
        ExportValue::Num(p.status_code.unwrap_or(0) as f64),
        ExportValue::Str(p.title.clone().unwrap_or_default()),
        ExportValue::Str(p.meta_description.clone().unwrap_or_default()),
        ExportValue::Str(p.h1.clone().unwrap_or_default()),
        ExportValue::Str(p.canonical.clone().unwrap_or_default()),
        ExportValue::Str(p.html_lang.clone().unwrap_or_default()),
        ExportValue::Str(indexable.to_string()),
        ExportValue::Num(p.depth as f64),
        ExportValue::Str(p.parent_url.clone().unwrap_or_default()),
        ExportValue::Num(p.size_bytes.unwrap_or(0) as f64),
        ExportValue::Num(p.load_time_ms.unwrap_or(0) as f64),
        ExportValue::Str(
            p.semantic_issues_json
                .clone()
                .unwrap_or_else(|| "[]".to_string()),
        ),
        ExportValue::Num(p.readability_score.unwrap_or(0.0)),
        ExportValue::Num(p.pagespeed_score.unwrap_or(0.0)),
        ExportValue::Num(p.seo_score.unwrap_or(0.0)),
        ExportValue::Str(grade),
        ExportValue::Num(priority_fixes),
        ExportValue::Num(failed_checks),
        ExportValue::Str(if p.blocked { "Yes" } else { "No" }.to_string()),
        ExportValue::Str(
            p.duplicate_group_id
                .map(|g| g.to_string())
                .unwrap_or_default(),
        ),
        ExportValue::Str(p.redirect_from_url.clone().unwrap_or_default()),
    ];
    for cat in CATEGORY_ORDER {
        let score = audit.as_ref().and_then(|a| category_score(a, cat));
        values.push(match score {
            Some(s) => ExportValue::Num(s),
            None => ExportValue::Str(String::new()),
        });
    }
    values.push(ExportValue::Str(
        p.keywords_json.clone().unwrap_or_default(),
    ));
    values.push(ExportValue::Str(p.og_json.clone().unwrap_or_default()));
    values.push(ExportValue::Str(
        p.hreflang_json.clone().unwrap_or_default(),
    ));
    values
}

fn csv_record(p: &CrawlResult) -> Vec<String> {
    page_values(p)
        .into_iter()
        .map(|v| match v {
            ExportValue::Num(n) => {
                if n.fract() == 0.0 {
                    format!("{n:.0}")
                } else {
                    n.to_string()
                }
            }
            ExportValue::Str(s) => s,
        })
        .collect()
}

/// Concatenates a check's offending elements (up to 3), snippet included when
/// the parser captured one.
fn check_elements(examples: &[SemanticIssue]) -> String {
    examples
        .iter()
        .take(3)
        .map(|e| match e.snippet.as_deref() {
            Some(s) => format!("{}: {}", e.element, s),
            None => e.element.clone(),
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Excel caps cell text at 32,767 characters; longer JSON blobs (semantic
/// issues, hreflang, keywords) would otherwise abort the whole export.
const EXCEL_MAX_STR: usize = 32_767;

fn xlsx_str(
    ws: &mut rust_xlsxwriter::Worksheet,
    row: u32,
    col: u16,
    val: &str,
    fmt: Option<&rust_xlsxwriter::Format>,
) -> Result<(), AppError> {
    let val: String = if val.chars().count() > EXCEL_MAX_STR {
        val.chars().take(EXCEL_MAX_STR).collect()
    } else {
        val.to_string()
    };
    let res = match fmt {
        Some(f) => ws.write_string_with_format(row, col, val, f),
        None => ws.write_string(row, col, val),
    };
    res.map_err(|e| AppError::Crawl(e.to_string()))?;
    Ok(())
}

fn xlsx_num(
    ws: &mut rust_xlsxwriter::Worksheet,
    row: u32,
    col: u16,
    val: f64,
    fmt: Option<&rust_xlsxwriter::Format>,
) -> Result<(), AppError> {
    let res = match fmt {
        Some(f) => ws.write_number_with_format(row, col, val, f),
        None => ws.write_number(row, col, val),
    };
    res.map_err(|e| AppError::Crawl(e.to_string()))?;
    Ok(())
}

const MAX_ROWS_PER_SHEET: u32 = 1_048_575;

fn sheet_name(base: &str, index: usize, total: usize) -> String {
    if total <= 1 {
        base.to_string()
    } else {
        format!("{} {}", base, index + 1)
    }
}

fn sheet_count(rows: u32) -> usize {
    rows.div_ceil(MAX_ROWS_PER_SHEET) as usize
}

/// Shared XLSX cell formats. `Format` is cloneable; each sheet pass borrows
/// the ones it needs.
struct Formats {
    header: rust_xlsxwriter::Format,
    alt: rust_xlsxwriter::Format,
    wrap: rust_xlsxwriter::Format,
    err: rust_xlsxwriter::Format,
    warn: rust_xlsxwriter::Format,
    info: rust_xlsxwriter::Format,
}

impl Formats {
    fn new() -> Self {
        use rust_xlsxwriter::{Format, FormatAlign};
        Self {
            header: Format::new()
                .set_bold()
                .set_font_color(0xFFFFFF)
                .set_background_color(0x2E3440)
                .set_align(FormatAlign::Center),
            alt: Format::new().set_background_color(0xF8F9FA),
            wrap: Format::new().set_text_wrap(),
            err: Format::new()
                .set_background_color(0xFFE0E0)
                .set_font_color(0x721C24),
            warn: Format::new()
                .set_background_color(0xFFF3CD)
                .set_font_color(0x856404),
            info: Format::new()
                .set_background_color(0xD1ECF1)
                .set_font_color(0x0C5460),
        }
    }
}

fn write_headers(
    ws: &mut rust_xlsxwriter::Worksheet,
    header_fmt: &rust_xlsxwriter::Format,
    headers: &[&str],
) -> Result<(), AppError> {
    for (col, h) in headers.iter().enumerate() {
        xlsx_str(ws, 0, col as u16, h, Some(header_fmt))?;
    }
    Ok(())
}

fn apply_widths(
    ws: &mut rust_xlsxwriter::Worksheet,
    widths: &[(u16, f64)],
) -> Result<(), AppError> {
    for (col, w) in widths {
        ws.set_column_width(*col, *w)
            .map_err(|e| AppError::Crawl(e.to_string()))?;
    }
    Ok(())
}

fn write_page_row(
    ws: &mut rust_xlsxwriter::Worksheet,
    row: u32,
    values: &[ExportValue],
    alt: &rust_xlsxwriter::Format,
    wrap: &rust_xlsxwriter::Format,
) -> Result<(), AppError> {
    let band = if row.is_multiple_of(2) {
        Some(alt)
    } else {
        None
    };
    for (col, v) in values.iter().enumerate() {
        let col = col as u16;
        match v {
            ExportValue::Num(n) => xlsx_num(ws, row, col, *n, band)?,
            ExportValue::Str(s) => {
                // Title / Meta Description get text wrap; the rest band by row.
                let fmt = if col == 2 || col == 3 {
                    Some(wrap)
                } else {
                    band
                };
                xlsx_str(ws, row, col, s, fmt)?;
            }
        }
    }
    Ok(())
}

/// Writes a page row for the flat Pages table. Always emits exactly one row.
fn page_row(
    ws: &mut rust_xlsxwriter::Worksheet,
    row: u32,
    p: &CrawlResult,
    fmt: &Formats,
) -> Result<u32, AppError> {
    write_page_row(ws, row, &page_values(p), &fmt.alt, &fmt.wrap)?;
    Ok(1)
}

/// Writes all semantic issues of a page (any severity). Returns the row count.
fn issues_row(
    ws: &mut rust_xlsxwriter::Worksheet,
    row: u32,
    p: &CrawlResult,
    fmt: &Formats,
) -> Result<u32, AppError> {
    let Some(ref js) = p.semantic_issues_json else {
        return Ok(0);
    };
    let Ok(issues) = serde_json::from_str::<Vec<serde_json::Value>>(js) else {
        return Ok(0);
    };
    let mut written = 0;
    for iss in &issues {
        if row + written > MAX_ROWS_PER_SHEET {
            break;
        }
        write_issue_row(
            ws,
            row + written,
            &p.url,
            iss,
            &fmt.wrap,
            &fmt.err,
            &fmt.warn,
            &fmt.info,
        )?;
        written += 1;
    }
    Ok(written)
}

#[allow(clippy::too_many_arguments)]
fn write_issue_row(
    ws: &mut rust_xlsxwriter::Worksheet,
    row: u32,
    url: &str,
    iss: &serde_json::Value,
    wrap: &rust_xlsxwriter::Format,
    err_fmt: &rust_xlsxwriter::Format,
    warn_fmt: &rust_xlsxwriter::Format,
    info_fmt: &rust_xlsxwriter::Format,
) -> Result<(), AppError> {
    xlsx_str(ws, row, 0, url, None)?;
    xlsx_str(
        ws,
        row,
        1,
        iss.get("issue_type").and_then(|v| v.as_str()).unwrap_or(""),
        None,
    )?;
    let sev = iss.get("severity").and_then(|v| v.as_str()).unwrap_or("");
    let sev_fmt = match sev {
        "error" => Some(err_fmt),
        "warning" => Some(warn_fmt),
        "info" => Some(info_fmt),
        _ => None,
    };
    xlsx_str(ws, row, 2, sev, sev_fmt)?;
    xlsx_str(
        ws,
        row,
        3,
        iss.get("message").and_then(|v| v.as_str()).unwrap_or(""),
        Some(wrap),
    )?;
    xlsx_str(
        ws,
        row,
        4,
        iss.get("element").and_then(|v| v.as_str()).unwrap_or(""),
        None,
    )?;
    xlsx_str(
        ws,
        row,
        5,
        iss.get("css_selector")
            .and_then(|v| v.as_str())
            .unwrap_or(""),
        None,
    )?;
    xlsx_str(
        ws,
        row,
        6,
        iss.get("xpath").and_then(|v| v.as_str()).unwrap_or(""),
        None,
    )?;
    Ok(())
}

/// Writes one row per audited category of the page. Returns the row count.
fn audit_row(
    ws: &mut rust_xlsxwriter::Worksheet,
    row: u32,
    p: &CrawlResult,
    _fmt: &Formats,
) -> Result<u32, AppError> {
    let Some(audit) = parse_audit(&p.seo_audit_json) else {
        return Ok(0);
    };
    let mut written = 0;
    for cat in &audit.categories {
        if row + written > MAX_ROWS_PER_SHEET {
            break;
        }
        let r = row + written;
        xlsx_str(ws, r, 0, &p.url, None)?;
        xlsx_str(ws, r, 1, category_label(&cat.category), None)?;
        xlsx_num(ws, r, 2, cat.score, None)?;
        xlsx_num(ws, r, 3, cat.weight, None)?;
        xlsx_num(ws, r, 4, cat.passed_checks as f64, None)?;
        xlsx_num(ws, r, 5, cat.total_checks as f64, None)?;
        written += 1;
    }
    Ok(written)
}

/// Writes one row per failing check of the page. Returns the row count.
fn checks_row(
    ws: &mut rust_xlsxwriter::Worksheet,
    row: u32,
    p: &CrawlResult,
    fmt: &Formats,
) -> Result<u32, AppError> {
    let Some(audit) = parse_audit(&p.seo_audit_json) else {
        return Ok(0);
    };
    let mut written = 0;
    for chk in audit.checks.iter().filter(|c| !c.passed) {
        if row + written > MAX_ROWS_PER_SHEET {
            break;
        }
        let r = row + written;
        xlsx_str(ws, r, 0, &p.url, None)?;
        xlsx_str(ws, r, 1, &chk.id, None)?;
        xlsx_str(ws, r, 2, category_label(&chk.category), None)?;
        let sev_fmt = match chk.severity.as_str() {
            "error" => Some(&fmt.err),
            "warning" => Some(&fmt.warn),
            "info" => Some(&fmt.info),
            _ => None,
        };
        xlsx_str(ws, r, 3, &chk.severity, sev_fmt)?;
        xlsx_str(ws, r, 4, &chk.message, Some(&fmt.wrap))?;
        xlsx_str(ws, r, 5, &chk.guidance, Some(&fmt.wrap))?;
        xlsx_str(ws, r, 6, chk.evidence.as_deref().unwrap_or(""), None)?;
        xlsx_str(ws, r, 7, &check_elements(&chk.examples), None)?;
        written += 1;
    }
    Ok(written)
}

/// Writes one row per priority fix of the page. Returns the row count.
fn fixes_row(
    ws: &mut rust_xlsxwriter::Worksheet,
    row: u32,
    p: &CrawlResult,
    fmt: &Formats,
) -> Result<u32, AppError> {
    let Some(audit) = parse_audit(&p.seo_audit_json) else {
        return Ok(0);
    };
    let mut written = 0;
    for fix in &audit.priority_fixes {
        if row + written > MAX_ROWS_PER_SHEET {
            break;
        }
        let r = row + written;
        xlsx_str(ws, r, 0, &p.url, None)?;
        xlsx_str(ws, r, 1, &fix.priority, None)?;
        xlsx_str(ws, r, 2, category_label(&fix.category), None)?;
        xlsx_str(ws, r, 3, &fix.id, None)?;
        xlsx_str(ws, r, 4, &fix.message, Some(&fmt.wrap))?;
        xlsx_str(ws, r, 5, &fix.guidance, Some(&fmt.wrap))?;
        xlsx_str(ws, r, 6, fix.evidence.as_deref().unwrap_or(""), None)?;
        written += 1;
    }
    Ok(written)
}

fn finalize_sheet(
    ws: &mut rust_xlsxwriter::Worksheet,
    data_rows: u32,
    num_cols: u16,
) -> Result<(), AppError> {
    ws.set_column_width(0, 60.0)
        .map_err(|e| AppError::Crawl(e.to_string()))?;
    if num_cols > 2 {
        ws.set_column_width(2, 40.0)
            .map_err(|e| AppError::Crawl(e.to_string()))?;
    }
    if num_cols > 3 {
        ws.set_column_width(3, 40.0)
            .map_err(|e| AppError::Crawl(e.to_string()))?;
    }
    if data_rows > 0 {
        ws.autofilter(0, 0, data_rows, num_cols - 1)
            .map_err(|e| AppError::Crawl(e.to_string()))?;
    }
    ws.set_freeze_panes(1, 0)
        .map_err(|e| AppError::Crawl(e.to_string()))?;
    Ok(())
}

/// One XLSX sheet pass over the crawled pages. The pass re-reads pages in
/// batches and writes `total_rows` data rows (each pass may split into several
/// physical sheets when the data exceeds `MAX_ROWS_PER_SHEET`).
struct PagePass<'a> {
    /// Progress stage emitted while this pass streams.
    stage: &'static str,
    base_name: &'a str,
    headers: &'a [&'a str],
    total_rows: u32,
    /// Extra per-sheet column widths applied after the defaults.
    widths: &'a [(u16, f64)],
    /// Writes one page's contribution, returning the number of rows written.
    write:
        fn(&mut rust_xlsxwriter::Worksheet, u32, &CrawlResult, &Formats) -> Result<u32, AppError>,
}

const ISSUE_HEADERS: [&str; 7] = [
    "URL",
    "Issue Type",
    "Severity",
    "Message",
    "Element",
    "Selector",
    "XPath",
];
const AUDIT_HEADERS: [&str; 6] = [
    "URL",
    "Category",
    "Score",
    "Weight",
    "Passed Checks",
    "Total Checks",
];
const CHECK_HEADERS: [&str; 8] = [
    "URL", "Check ID", "Category", "Severity", "Message", "Guidance", "Evidence", "Elements",
];
const FIX_HEADERS: [&str; 7] = [
    "URL", "Priority", "Category", "Fix ID", "Message", "Guidance", "Evidence",
];
const LINK_HEADERS: [&str; 5] = ["From URL", "To URL", "Link Type", "Anchor Text", "Follow"];

#[allow(clippy::too_many_arguments)]
async fn export_page_passes(
    state: &Arc<RwLock<AppState>>,
    emit: &EmitProgress,
    project_id: &str,
    workbook: &mut rust_xlsxwriter::Workbook,
    fmt: &Formats,
    passes: &[PagePass<'_>],
    processed: &mut u64,
    total: u64,
) -> Result<(), AppError> {
    for pass in passes {
        if pass.total_rows == 0 {
            continue;
        }
        let num_sheets = sheet_count(pass.total_rows);
        let mut sheet_idx: usize = 0;
        let mut sheet_rows: u32 = 1;
        let mut ws = workbook.add_worksheet_with_constant_memory();
        ws.set_name(sheet_name(pass.base_name, 0, num_sheets))
            .map_err(|e| AppError::Crawl(e.to_string()))?;
        write_headers(ws, &fmt.header, pass.headers)?;

        let mut last_timestamp: Option<String> = None;
        let mut last_id: Option<String> = None;
        loop {
            let pid = project_id.to_string();
            let last_ts = last_timestamp.clone();
            let last_id_cursor = last_id.clone();
            let batch = with_repo_arc(state, move |repo| {
                repo.get_result_batch(
                    &pid,
                    last_ts.as_deref(),
                    last_id_cursor.as_deref(),
                    PAGE_BATCH_SIZE,
                )
            })
            .await?;
            if batch.is_empty() {
                break;
            }
            let mut batch_rows: u64 = 0;
            for p in &batch {
                if sheet_rows > MAX_ROWS_PER_SHEET {
                    finalize_sheet(ws, MAX_ROWS_PER_SHEET, pass.headers.len() as u16)?;
                    apply_widths(ws, pass.widths)?;
                    sheet_idx += 1;
                    ws = workbook.add_worksheet_with_constant_memory();
                    ws.set_name(sheet_name(pass.base_name, sheet_idx, num_sheets))
                        .map_err(|e| AppError::Crawl(e.to_string()))?;
                    write_headers(ws, &fmt.header, pass.headers)?;
                    sheet_rows = 1;
                }
                let written = (pass.write)(ws, sheet_rows, p, fmt)?;
                sheet_rows += written;
                batch_rows += written as u64;
            }
            *processed += batch_rows;
            emit(pass.stage, *processed, total);
            let last = batch.last().expect("batch is not empty");
            last_timestamp = Some(last.crawl_timestamp.clone());
            last_id = Some(last.id.clone());
        }
        finalize_sheet(ws, sheet_rows.saturating_sub(1), pass.headers.len() as u16)?;
        apply_widths(ws, pass.widths)?;
    }
    Ok(())
}

async fn export_csv_single(
    state: &Arc<RwLock<AppState>>,
    emit: &EmitProgress,
    project_id: &str,
    file_path: &str,
    total_pages: u32,
) -> Result<(), AppError> {
    let mut wtr = csv::Writer::from_path(file_path)?;

    let headers = page_headers();
    wtr.write_record(headers.iter().map(|h| h.to_string()))?;

    let mut last_timestamp: Option<String> = None;
    let mut last_id: Option<String> = None;
    let mut processed: u64 = 0;

    loop {
        let pid = project_id.to_string();
        let last_ts = last_timestamp.clone();
        let last_id_cursor = last_id.clone();
        let batch = with_repo_arc(state, move |repo| {
            repo.get_result_batch(
                &pid,
                last_ts.as_deref(),
                last_id_cursor.as_deref(),
                PAGE_BATCH_SIZE,
            )
        })
        .await?;
        if batch.is_empty() {
            break;
        }
        for item in &batch {
            let record = csv_record(item);
            wtr.write_record(record.iter())?;
        }
        processed += batch.len() as u64;
        emit("pages", processed, total_pages as u64);
        let last = batch.last().expect("batch is not empty");
        last_timestamp = Some(last.crawl_timestamp.clone());
        last_id = Some(last.id.clone());
    }

    wtr.flush()?;
    emit("pages", total_pages as u64, total_pages as u64);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn export_xlsx(
    state: &Arc<RwLock<AppState>>,
    emit: &EmitProgress,
    project_id: &str,
    file_path: &str,
    total_pages: u32,
    total_links: u32,
    total_issues: u32,
    total_seo: u32,
) -> Result<(), AppError> {
    use rust_xlsxwriter::Workbook;

    let fmt = Formats::new();
    let mut workbook = Workbook::new();

    let total = total_pages as u64 + total_issues as u64 + total_links as u64 + total_seo as u64;
    let mut processed: u64 = 0;

    let headers = page_headers();
    let passes = [
        PagePass {
            stage: "pages",
            base_name: "Pages",
            headers: &headers,
            total_rows: total_pages,
            widths: &[],
            write: page_row,
        },
        PagePass {
            stage: "issues",
            base_name: "Issues",
            headers: &ISSUE_HEADERS,
            total_rows: total_issues,
            widths: &[(3, 50.0), (5, 50.0), (6, 50.0)],
            write: issues_row,
        },
        PagePass {
            stage: "seo",
            base_name: "SEO Audit",
            headers: &AUDIT_HEADERS,
            total_rows: total_seo,
            widths: &[],
            write: audit_row,
        },
        PagePass {
            stage: "seo",
            base_name: "SEO Checks",
            headers: &CHECK_HEADERS,
            total_rows: total_seo,
            widths: &[(4, 50.0), (5, 50.0), (7, 50.0)],
            write: checks_row,
        },
        PagePass {
            stage: "seo",
            base_name: "SEO Fixes",
            headers: &FIX_HEADERS,
            total_rows: total_seo,
            widths: &[(4, 50.0), (5, 50.0)],
            write: fixes_row,
        },
    ];
    export_page_passes(
        state,
        emit,
        project_id,
        &mut workbook,
        &fmt,
        &passes,
        &mut processed,
        total,
    )
    .await?;

    // === Links sheet (streamed by rowid batch, not derived from pages) ===
    if total_links > 0 {
        let num_sheets = sheet_count(total_links);
        let mut sheet_idx: usize = 0;
        let mut sheet_rows: u32 = 1;
        let mut ws = workbook.add_worksheet_with_constant_memory();
        ws.set_name(sheet_name("Links", 0, num_sheets))
            .map_err(|e| AppError::Crawl(e.to_string()))?;
        for (col, h) in LINK_HEADERS.iter().enumerate() {
            xlsx_str(ws, 0, col as u16, h, Some(&fmt.header))?;
        }

        let mut last_rowid: Option<i64> = None;
        loop {
            let pid = project_id.to_string();
            let cursor = last_rowid;
            let batch = with_repo_arc(state, move |repo| {
                repo.get_links_batch(&pid, cursor, LINK_BATCH_SIZE)
            })
            .await?;
            if batch.is_empty() {
                break;
            }
            for (_, lk) in &batch {
                if sheet_rows > MAX_ROWS_PER_SHEET {
                    finalize_sheet(ws, MAX_ROWS_PER_SHEET, 5)?;
                    apply_widths(ws, &[(1, 60.0)])?;
                    sheet_idx += 1;
                    ws = workbook.add_worksheet_with_constant_memory();
                    ws.set_name(sheet_name("Links", sheet_idx, num_sheets))
                        .map_err(|e| AppError::Crawl(e.to_string()))?;
                    for (col, h) in LINK_HEADERS.iter().enumerate() {
                        xlsx_str(ws, 0, col as u16, h, Some(&fmt.header))?;
                    }
                    sheet_rows = 1;
                }
                let r = sheet_rows;
                let f = if r.is_multiple_of(2) {
                    Some(&fmt.alt)
                } else {
                    None
                };
                xlsx_str(ws, r, 0, &lk.from_url, f)?;
                xlsx_str(ws, r, 1, &lk.to_url, f)?;
                xlsx_str(ws, r, 2, &lk.link_type, f)?;
                xlsx_str(ws, r, 3, lk.anchor_text.as_deref().unwrap_or(""), f)?;
                xlsx_str(ws, r, 4, if lk.is_follow { "Yes" } else { "No" }, f)?;
                sheet_rows += 1;
            }
            processed += batch.len() as u64;
            emit("links", processed, total);
            last_rowid = Some(batch.last().expect("batch is not empty").0);
        }
        finalize_sheet(ws, sheet_rows.saturating_sub(1), 5)?;
        apply_widths(ws, &[(1, 60.0)])?;
    }

    workbook
        .save(file_path)
        .map_err(|e| AppError::Crawl(e.to_string()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::schema::run_migrations;
    use rusqlite::Connection;

    const SAMPLE_AUDIT: &str = r#"{
        "score": 72.0,
        "grade": "C",
        "categories": [
            {"category":"meta","score":80.0,"weight":0.25,"passed_weight":0.2,"total_weight":0.25,"passed_checks":3,"total_checks":4},
            {"category":"sxo","score":50.0,"weight":0.08,"passed_weight":0.04,"total_weight":0.08,"passed_checks":1,"total_checks":3},
            {"category":"security","score":42.0,"weight":0.12,"passed_weight":0.0,"total_weight":0.12,"passed_checks":0,"total_checks":6},
            {"category":"compliance","score":0.0,"weight":0.08,"passed_weight":0.0,"total_weight":0.08,"passed_checks":0,"total_checks":3}
        ],
        "checks": [
            {"id":"title_length","category":"meta","severity":"warning","passed":false,"weight":2.0,"message":"Title length: 10 chars","guidance":"Keep it 30-65","evidence":"10"},
            {"id":"https_used","category":"technical","severity":"error","passed":true,"weight":3.0,"message":"HTTPS ok","guidance":"-"}
        ],
        "priority_fixes": [
            {"id":"title_length","priority":"important","message":"Title too short","guidance":"Extend","category":"meta","evidence":"10"}
        ]
    }"#;

    fn test_state() -> Arc<RwLock<AppState>> {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        conn.execute_batch(
            "INSERT INTO projects (id, name, created_at, updated_at) VALUES ('p1', 'P1', datetime('now'), datetime('now'));
             INSERT INTO crawl_config (id, project_id, seed_urls, created_at) VALUES ('cfg', 'p1', '[]', datetime('now'));",
        )
        .unwrap();
        let cache = Arc::new(std::sync::Mutex::new(lru::LruCache::new(
            std::num::NonZeroUsize::new(8).unwrap(),
        )));
        Arc::new(RwLock::new(AppState {
            db: std::sync::Mutex::new(conn),
            crawls: Arc::new(RwLock::new(std::collections::HashMap::new())),
            seo_audits: Arc::new(RwLock::new(std::collections::HashMap::new())),
            results_cache: cache,
            transfer_server: std::sync::Mutex::new(None),
        }))
    }

    fn sample_page(id: &str, url: &str) -> CrawlResult {
        CrawlResult {
            id: id.to_string(),
            config_id: "cfg".to_string(),
            project_id: "p1".to_string(),
            url: url.to_string(),
            status_code: Some(200),
            blocked: false,
            title: Some("Sample Title".to_string()),
            meta_description: Some("A meta description".to_string()),
            h1: Some("Sample H1".to_string()),
            canonical: Some(url.to_string()),
            size_bytes: Some(2048),
            load_time_ms: Some(50),
            is_indexable: Some(true),
            depth: 1,
            parent_url: Some("https://x.com".to_string()),
            crawl_timestamp: "2026-01-01T00:00:00Z".to_string(),
            links: Vec::new(),
            html_lang: Some("en".to_string()),
            hreflang_json: Some(
                r#"[{"hreflang":"en","href":"https://x.com/"}]"#.to_string(),
            ),
            semantic_issues_json: Some(
                r#"[{"issue_type":"missing_title","severity":"error","element":"head","message":"m1"},{"issue_type":"img_no_alt","severity":"warning","element":"img","message":"m2"}]"#
                    .to_string(),
            ),
            html_body: None,
            readability_score: Some(0.7),
            content_hash: None,
            duplicate_group_id: Some(3),
            keywords_json: Some(r#"[{"keyword":"foo"}]"#.to_string()),
            og_json: Some(r#"{"og:title":"T"}"#.to_string()),
            pagespeed_score: Some(0.85),
            pagespeed_json: Some(r#"{"performance":0.85}"#.to_string()),
            seo_score: Some(72.0),
            seo_audit_json: Some(SAMPLE_AUDIT.to_string()),
            redirect_from_url: Some("https://old.example.com/page".to_string()),
        }
    }

    async fn seed(state: &Arc<RwLock<AppState>>, pages: &[CrawlResult]) {
        let pages = pages.to_vec();
        with_repo_arc(state, move |repo| repo.save_results_batch(&pages))
            .await
            .unwrap();
    }

    async fn seed_redirect(state: &Arc<RwLock<AppState>>, page_id: &str, from_url: &str) {
        let page_id = page_id.to_string();
        let from_url = from_url.to_string();
        with_repo_arc(state, move |repo| {
            repo.save_redirect_batch(&[crate::models::RedirectRecord {
                page_id,
                project_id: "p1".to_string(),
                redirect_from_url: Some(from_url),
                chain: Vec::new(),
            }])
        })
        .await
        .unwrap();
    }

    fn noop_emit(_: &'static str, _: u64, _: u64) {}

    #[test]
    fn page_values_flatten_seo_data() {
        let p = sample_page("a", "https://x.com/a");
        let values = page_values(&p);
        assert_eq!(values.len(), 35, "35 columns expected (33 + security + compliance)");

        let seo_score = &values[15];
        assert!(matches!(seo_score, ExportValue::Num(n) if *n == 72.0));
        let grade = &values[16];
        assert!(matches!(grade, ExportValue::Str(s) if s == "C"));
        let priority_fixes = &values[17];
        assert!(matches!(priority_fixes, ExportValue::Num(n) if *n == 1.0));
        let failed_checks = &values[18];
        assert!(matches!(failed_checks, ExportValue::Num(n) if *n == 1.0));
        let redirect = &values[21];
        assert!(matches!(redirect, ExportValue::Str(s) if s == "https://old.example.com/page"));

        // Category columns follow CATEGORY_ORDER at indices 22..32.
        assert!(matches!(&values[22], ExportValue::Num(n) if *n == 80.0));
        assert!(
            matches!(&values[22 + CATEGORY_ORDER.len() - 1], ExportValue::Num(n) if *n == 0.0)
        );
    }

    #[tokio::test]
    async fn test_export_csv_includes_seo_columns() {
        let state = test_state();
        seed(&state, &[sample_page("a", "https://x.com/a")]).await;
        seed_redirect(&state, "a", "https://old.example.com/page").await;

        let counts = with_repo_arc(&state, |repo| {
            Ok((repo.count_issues("p1")?, repo.count_seo_rows("p1")?))
        })
        .await
        .unwrap();
        assert_eq!(counts.0, 2, "count_issues covers all severities");
        assert_eq!(counts.1, 6, "4 categories + 1 failing check + 1 fix");
        let path = std::env::temp_dir().join("open-crawler-test-export.csv");
        let _ = std::fs::remove_file(&path);

        let calls: Arc<std::sync::Mutex<Vec<(String, u64, u64)>>> =
            Arc::new(std::sync::Mutex::new(Vec::new()));
        let emit = move |s: &'static str, p: u64, t: u64| {
            calls.lock().unwrap().push((s.to_string(), p, t));
        };
        export_csv_single(&state, &emit, "p1", path.to_str().unwrap(), 1)
            .await
            .unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        for header in [
            "URL",
            "Semantic Issues",
            "Readability Score",
            "SEO Score",
            "SEO Grade",
            "Redirect From",
            "Hreflang JSON",
        ] {
            assert!(content.contains(header), "missing header: {header}");
        }
        assert!(content.contains("https://old.example.com/page"));
        assert!(content.contains("missing_title"), "error severity kept");
        assert!(content.contains("img_no_alt"), "warning severity kept");
        assert!(content.contains("SXO"));
        let _ = std::fs::remove_file(&path);
    }

    #[tokio::test]
    async fn test_export_xlsx_creates_seo_sheets() {
        use rust_xlsxwriter::Workbook;

        let state = test_state();
        seed(&state, &[sample_page("a", "https://x.com/a")]).await;

        let mut workbook = Workbook::new();
        let fmt = Formats::new();
        let headers = page_headers();
        let passes = [
            PagePass {
                stage: "pages",
                base_name: "Pages",
                headers: &headers,
                total_rows: 1,
                widths: &[],
                write: page_row,
            },
            PagePass {
                stage: "issues",
                base_name: "Issues",
                headers: &ISSUE_HEADERS,
                total_rows: 2,
                widths: &[],
                write: issues_row,
            },
            PagePass {
                stage: "seo",
                base_name: "SEO Audit",
                headers: &AUDIT_HEADERS,
                total_rows: 2,
                widths: &[],
                write: audit_row,
            },
            PagePass {
                stage: "seo",
                base_name: "SEO Checks",
                headers: &CHECK_HEADERS,
                total_rows: 1,
                widths: &[],
                write: checks_row,
            },
            PagePass {
                stage: "seo",
                base_name: "SEO Fixes",
                headers: &FIX_HEADERS,
                total_rows: 1,
                widths: &[],
                write: fixes_row,
            },
        ];
        let mut processed: u64 = 0;
        export_page_passes(
            &state,
            &noop_emit,
            "p1",
            &mut workbook,
            &fmt,
            &passes,
            &mut processed,
            5,
        )
        .await
        .unwrap();

        assert_eq!(
            processed, 9,
            "1 page + 2 issues + 4 categories + 1 check + 1 fix"
        );
        let names: Vec<String> = workbook.worksheets().iter().map(|w| w.name()).collect();
        for expected in ["Pages", "Issues", "SEO Audit", "SEO Checks", "SEO Fixes"] {
            assert!(
                names.contains(&expected.to_string()),
                "missing sheet {expected}"
            );
        }
    }

    #[tokio::test]
    async fn test_export_xlsx_writes_file() {
        let state = test_state();
        seed(&state, &[sample_page("a", "https://x.com/a")]).await;

        let path = std::env::temp_dir().join("open-crawler-test-export.xlsx");
        let _ = std::fs::remove_file(&path);

        export_xlsx(&state, &noop_emit, "p1", path.to_str().unwrap(), 1, 0, 2, 4)
            .await
            .unwrap();

        let meta = std::fs::metadata(&path).unwrap();
        assert!(meta.len() > 0, "workbook file should not be empty");
        let _ = std::fs::remove_file(&path);
    }
}
