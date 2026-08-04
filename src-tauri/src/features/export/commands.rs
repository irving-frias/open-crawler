use std::path::Path;
use std::sync::Arc;

use tauri::{AppHandle, Emitter, Manager, State};
use tokio::sync::RwLock;
use tracing::warn;

use crate::db::CrawlRepo;
use crate::error::AppError;
use crate::features::with_repo;
use crate::AppState;

const PAGE_BATCH_SIZE: u32 = 1000;
const LINK_BATCH_SIZE: u32 = 5000;

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
    let (total_pages, total_links, total_issues) = with_repo(&state, move |repo| {
        let total_pages = repo.count_pages(&pid)?;
        let total_links = repo.count_links(&pid)?;
        let total_issues = repo.count_issues(&pid)?;
        Ok((total_pages, total_links, total_issues))
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

    if format == "xlsx" {
        let app_export = app.clone();
        let pid_export = project_id.clone();
        let dest_export = dest_path.clone();
        with_repo(&state, move |repo| {
            export_xlsx(
                repo,
                &app_export,
                &pid_export,
                &dest_export,
                total_pages,
                total_links,
                total_issues,
            )
        })
        .await?;
    } else {
        let app_export = app.clone();
        let pid_export = project_id.clone();
        let dest_export = dest_path.clone();
        with_repo(&state, move |repo| {
            export_csv_single(repo, &app_export, &pid_export, &dest_export, total_pages)
        })
        .await?;
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
fn copy_to_content_uri(app: &AppHandle, uri: &str, src: &std::path::Path) -> Result<(), AppError> {
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
fn copy_to_content_uri(_app: &AppHandle, _uri: &str, _src: &std::path::Path) -> Result<(), AppError> {
    Ok(())
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
        let _ = app.share_file().share_file(tauri_plugin_share::ShareRequest {
            path: Some(path),
            mime: Some(mime),
            group: None,
        });
    });
}

#[cfg(not(mobile))]
fn share_export(_app: &AppHandle, _path: &str, _format: &str) {}

fn export_csv_single(
    repo: &CrawlRepo,
    app: &AppHandle,
    project_id: &str,
    file_path: &str,
    total_pages: u32,
) -> Result<(), AppError> {
    let mut wtr = csv::Writer::from_path(file_path)?;

    wtr.write_record([
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
    ])?;

    let mut last_timestamp: Option<String> = None;
    let mut last_id: Option<String> = None;
    let mut processed: u64 = 0;

    loop {
        let batch = repo.get_result_batch(
            project_id,
            last_timestamp.as_deref(),
            last_id.as_deref(),
            PAGE_BATCH_SIZE,
        )?;
        if batch.is_empty() {
            break;
        }
        for item in &batch {
            let indexable = item
                .is_indexable
                .map(|i| if i { "Yes".to_string() } else { "No".to_string() })
                .unwrap_or_else(|| "Unknown".to_string());
            let issues_str = item
                .semantic_issues_json
                .clone()
                .unwrap_or_else(|| "[]".to_string());
            wtr.write_record([
                &item.url,
                &item.status_code.map(|s| s.to_string()).unwrap_or_default(),
                &item.title.clone().unwrap_or_default(),
                &item.meta_description.clone().unwrap_or_default(),
                &item.h1.clone().unwrap_or_default(),
                &item.canonical.clone().unwrap_or_default(),
                &item.html_lang.clone().unwrap_or_default(),
                &indexable,
                &item.depth.to_string(),
                &item.parent_url.clone().unwrap_or_default(),
                &item.size_bytes.map(|s| s.to_string()).unwrap_or_default(),
                &item.load_time_ms.map(|l| l.to_string()).unwrap_or_default(),
                &issues_str,
            ])?;
        }
        processed += batch.len() as u64;
        emit_export_progress(app, "pages", processed, total_pages as u64);
        let last = batch.last().expect("batch is not empty");
        last_timestamp = Some(last.crawl_timestamp.clone());
        last_id = Some(last.id.clone());
    }

    wtr.flush()?;
    emit_export_progress(app, "pages", total_pages as u64, total_pages as u64);
    Ok(())
}

fn xlsx_str(ws: &mut rust_xlsxwriter::Worksheet, row: u32, col: u16, val: &str, fmt: Option<&rust_xlsxwriter::Format>) -> Result<(), AppError> {
    let res = match fmt {
        Some(f) => ws.write_string_with_format(row, col, val.to_string(), f),
        None => ws.write_string(row, col, val.to_string()),
    };
    res.map_err(|e| AppError::Crawl(e.to_string()))?;
    Ok(())
}

fn xlsx_num(ws: &mut rust_xlsxwriter::Worksheet, row: u32, col: u16, val: f64, fmt: Option<&rust_xlsxwriter::Format>) -> Result<(), AppError> {
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

fn write_page_row(ws: &mut rust_xlsxwriter::Worksheet, row: u32, p: &crate::models::CrawlResult, alt: &rust_xlsxwriter::Format, wrap: &rust_xlsxwriter::Format) -> Result<(), AppError> {
    let f = if row.is_multiple_of(2) { Some(alt) } else { None };
    xlsx_str(ws, row, 0, &p.url, f)?;
    xlsx_num(ws, row, 1, p.status_code.unwrap_or(0) as f64, f)?;
    xlsx_str(ws, row, 2, p.title.as_deref().unwrap_or(""), Some(wrap))?;
    xlsx_str(ws, row, 3, p.meta_description.as_deref().unwrap_or(""), Some(wrap))?;
    xlsx_str(ws, row, 4, p.h1.as_deref().unwrap_or(""), f)?;
    xlsx_str(ws, row, 5, p.canonical.as_deref().unwrap_or(""), f)?;
    xlsx_str(ws, row, 6, p.html_lang.as_deref().unwrap_or(""), f)?;
    let idx = p.is_indexable.map(|v| if v {"Yes"} else {"No"}).unwrap_or("Unknown");
    xlsx_str(ws, row, 7, idx, f)?;
    xlsx_num(ws, row, 8, p.depth as f64, f)?;
    xlsx_str(ws, row, 9, p.parent_url.as_deref().unwrap_or(""), f)?;
    xlsx_num(ws, row, 10, p.size_bytes.unwrap_or(0) as f64, f)?;
    xlsx_num(ws, row, 11, p.load_time_ms.unwrap_or(0) as f64, f)?;
    Ok(())
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
    xlsx_str(ws, row, 1, iss.get("issue_type").and_then(|v| v.as_str()).unwrap_or(""), None)?;
    let sev = iss.get("severity").and_then(|v| v.as_str()).unwrap_or("");
    let sev_fmt = match sev {
        "error" => Some(err_fmt),
        "warning" => Some(warn_fmt),
        "info" => Some(info_fmt),
        _ => None,
    };
    xlsx_str(ws, row, 2, sev, sev_fmt)?;
    xlsx_str(ws, row, 3, iss.get("message").and_then(|v| v.as_str()).unwrap_or(""), Some(wrap))?;
    xlsx_str(ws, row, 4, iss.get("element").and_then(|v| v.as_str()).unwrap_or(""), None)?;
    xlsx_str(ws, row, 5, iss.get("css_selector").and_then(|v| v.as_str()).unwrap_or(""), None)?;
    xlsx_str(ws, row, 6, iss.get("xpath").and_then(|v| v.as_str()).unwrap_or(""), None)?;
    Ok(())
}

fn finalize_sheet(ws: &mut rust_xlsxwriter::Worksheet, data_rows: u32, num_cols: u16) -> Result<(), AppError> {
    ws.set_column_width(0, 60.0).map_err(|e| AppError::Crawl(e.to_string()))?;
    if num_cols > 2 { ws.set_column_width(2, 40.0).map_err(|e| AppError::Crawl(e.to_string()))?; }
    if num_cols > 3 { ws.set_column_width(3, 40.0).map_err(|e| AppError::Crawl(e.to_string()))?; }
    if data_rows > 0 {
        ws.autofilter(0, 0, data_rows, num_cols - 1).map_err(|e| AppError::Crawl(e.to_string()))?;
    }
    ws.set_freeze_panes(1, 0).map_err(|e| AppError::Crawl(e.to_string()))?;
    Ok(())
}

const PAGE_HEADERS: [&str; 12] = ["URL","Status Code","Title","Meta Description","H1","Canonical","HTML Lang","Indexable","Depth","Parent URL","Size (bytes)","Load Time (ms)"];
const ISSUE_HEADERS: [&str; 7] = ["URL","Issue Type","Severity","Message","Element","Selector","XPath"];
const LINK_HEADERS: [&str; 5] = ["From URL","To URL","Link Type","Anchor Text","Follow"];

fn export_xlsx(
    repo: &CrawlRepo,
    app: &AppHandle,
    project_id: &str,
    file_path: &str,
    total_pages: u32,
    total_links: u32,
    total_issues: u32,
) -> Result<(), AppError> {
    use rust_xlsxwriter::{Format, FormatAlign, Workbook};

    let mut workbook = Workbook::new();
    let header_fmt = Format::new()
        .set_bold()
        .set_font_color(0xFFFFFF)
        .set_background_color(0x2E3440)
        .set_align(FormatAlign::Center);
    let alt = Format::new().set_background_color(0xF8F9FA);
    let wrap = Format::new().set_text_wrap();
    let err_fmt = Format::new().set_background_color(0xFFE0E0).set_font_color(0x721C24);
    let warn_fmt = Format::new().set_background_color(0xFFF3CD).set_font_color(0x856404);
    let info_fmt = Format::new().set_background_color(0xD1ECF1).set_font_color(0x0C5460);

    let total = total_pages as u64 + total_issues as u64 + total_links as u64;
    let mut processed: u64 = 0;

    // === Pass 1: Pages sheets (streamed by batch, split if > 1,048,575 pages) ===
    if total_pages > 0 {
        let num_sheets = sheet_count(total_pages);
        let mut sheet_idx: usize = 0;
        let mut sheet_rows: u32 = 1;
        let mut ws = workbook.add_worksheet_with_constant_memory();
        ws.set_name(sheet_name("Pages", 0, num_sheets)).map_err(|e| AppError::Crawl(e.to_string()))?;
        for (col, h) in PAGE_HEADERS.iter().enumerate() {
            xlsx_str(ws, 0, col as u16, h, Some(&header_fmt))?;
        }

        let mut last_timestamp: Option<String> = None;
        let mut last_id: Option<String> = None;
        loop {
            let batch = repo.get_result_batch(
                project_id,
                last_timestamp.as_deref(),
                last_id.as_deref(),
                PAGE_BATCH_SIZE,
            )?;
            if batch.is_empty() {
                break;
            }
            for p in &batch {
                if sheet_rows > MAX_ROWS_PER_SHEET {
                    finalize_sheet(ws, MAX_ROWS_PER_SHEET, 12)?;
                    sheet_idx += 1;
                    ws = workbook.add_worksheet_with_constant_memory();
                    ws.set_name(sheet_name("Pages", sheet_idx, num_sheets)).map_err(|e| AppError::Crawl(e.to_string()))?;
                    for (col, h) in PAGE_HEADERS.iter().enumerate() {
                        xlsx_str(ws, 0, col as u16, h, Some(&header_fmt))?;
                    }
                    sheet_rows = 1;
                }
                write_page_row(ws, sheet_rows, p, &alt, &wrap)?;
                sheet_rows += 1;
            }
            processed += batch.len() as u64;
            emit_export_progress(app, "pages", processed, total);
            let last = batch.last().expect("batch is not empty");
            last_timestamp = Some(last.crawl_timestamp.clone());
            last_id = Some(last.id.clone());
        }
        finalize_sheet(ws, sheet_rows.saturating_sub(1), 12)?;
    }

    // === Pass 2: Issues sheets (re-reads pages by batch, split if > 1,048,575 issues) ===
    if total_issues > 0 {
        let num_sheets = sheet_count(total_issues);
        let mut sheet_idx: usize = 0;
        let mut sheet_rows: u32 = 1;
        let mut ws = workbook.add_worksheet_with_constant_memory();
        ws.set_name(sheet_name("Issues", 0, num_sheets)).map_err(|e| AppError::Crawl(e.to_string()))?;
        for (col, h) in ISSUE_HEADERS.iter().enumerate() {
            xlsx_str(ws, 0, col as u16, h, Some(&header_fmt))?;
        }

        let mut last_timestamp: Option<String> = None;
        let mut last_id: Option<String> = None;
        loop {
            let batch = repo.get_result_batch(
                project_id,
                last_timestamp.as_deref(),
                last_id.as_deref(),
                PAGE_BATCH_SIZE,
            )?;
            if batch.is_empty() {
                break;
            }
            let mut issues_written: u64 = 0;
            for p in &batch {
                if let Some(ref js) = p.semantic_issues_json {
                    if let Ok(issues) = serde_json::from_str::<Vec<serde_json::Value>>(js) {
                        for iss in &issues {
                            if sheet_rows > MAX_ROWS_PER_SHEET {
                                finalize_sheet(ws, MAX_ROWS_PER_SHEET, 7)?;
                                ws.set_column_width(3, 50.0).map_err(|e| AppError::Crawl(e.to_string()))?;
                                ws.set_column_width(5, 50.0).map_err(|e| AppError::Crawl(e.to_string()))?;
                                ws.set_column_width(6, 50.0).map_err(|e| AppError::Crawl(e.to_string()))?;
                                sheet_idx += 1;
                                ws = workbook.add_worksheet_with_constant_memory();
                                ws.set_name(sheet_name("Issues", sheet_idx, num_sheets)).map_err(|e| AppError::Crawl(e.to_string()))?;
                                for (col, h) in ISSUE_HEADERS.iter().enumerate() {
                                    xlsx_str(ws, 0, col as u16, h, Some(&header_fmt))?;
                                }
                                sheet_rows = 1;
                            }
                            write_issue_row(ws, sheet_rows, &p.url, iss, &wrap, &err_fmt, &warn_fmt, &info_fmt)?;
                            sheet_rows += 1;
                            issues_written += 1;
                        }
                    }
                }
            }
            processed += issues_written;
            emit_export_progress(app, "issues", processed, total);
            let last = batch.last().expect("batch is not empty");
            last_timestamp = Some(last.crawl_timestamp.clone());
            last_id = Some(last.id.clone());
        }
        finalize_sheet(ws, sheet_rows.saturating_sub(1), 7)?;
        ws.set_column_width(3, 50.0).map_err(|e| AppError::Crawl(e.to_string()))?;
        ws.set_column_width(5, 50.0).map_err(|e| AppError::Crawl(e.to_string()))?;
        ws.set_column_width(6, 50.0).map_err(|e| AppError::Crawl(e.to_string()))?;
    }

    // === Pass 3: Links sheets (streamed by rowid batch, split if > 1,048,575 links) ===
    if total_links > 0 {
        let num_sheets = sheet_count(total_links);
        let mut sheet_idx: usize = 0;
        let mut sheet_rows: u32 = 1;
        let mut ws = workbook.add_worksheet_with_constant_memory();
        ws.set_name(sheet_name("Links", 0, num_sheets)).map_err(|e| AppError::Crawl(e.to_string()))?;
        for (col, h) in LINK_HEADERS.iter().enumerate() {
            xlsx_str(ws, 0, col as u16, h, Some(&header_fmt))?;
        }

        let mut last_rowid: Option<i64> = None;
        loop {
            let batch = repo.get_links_batch(project_id, last_rowid, LINK_BATCH_SIZE)?;
            if batch.is_empty() {
                break;
            }
            for (_, lk) in &batch {
                if sheet_rows > MAX_ROWS_PER_SHEET {
                    finalize_sheet(ws, MAX_ROWS_PER_SHEET, 5)?;
                    ws.set_column_width(1, 60.0).map_err(|e| AppError::Crawl(e.to_string()))?;
                    sheet_idx += 1;
                    ws = workbook.add_worksheet_with_constant_memory();
                    ws.set_name(sheet_name("Links", sheet_idx, num_sheets)).map_err(|e| AppError::Crawl(e.to_string()))?;
                    for (col, h) in LINK_HEADERS.iter().enumerate() {
                        xlsx_str(ws, 0, col as u16, h, Some(&header_fmt))?;
                    }
                    sheet_rows = 1;
                }
                let r = sheet_rows;
                let f = if r.is_multiple_of(2) { Some(&alt) } else { None };
                xlsx_str(ws, r, 0, &lk.from_url, f)?;
                xlsx_str(ws, r, 1, &lk.to_url, f)?;
                xlsx_str(ws, r, 2, &lk.link_type, f)?;
                xlsx_str(ws, r, 3, lk.anchor_text.as_deref().unwrap_or(""), f)?;
                xlsx_str(ws, r, 4, if lk.is_follow { "Yes" } else { "No" }, f)?;
                sheet_rows += 1;
            }
            processed += batch.len() as u64;
            emit_export_progress(app, "links", processed, total);
            last_rowid = Some(batch.last().expect("batch is not empty").0);
        }
        finalize_sheet(ws, sheet_rows.saturating_sub(1), 5)?;
        ws.set_column_width(1, 60.0).map_err(|e| AppError::Crawl(e.to_string()))?;
    }

    workbook.save(file_path).map_err(|e| AppError::Crawl(e.to_string()))?;
    Ok(())
}
