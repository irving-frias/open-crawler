use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tauri::Manager;
use tracing::{info, warn};
use uuid::Uuid;
use zip::write::SimpleFileOptions;

use crate::db::repos::CrawlRepo;
use crate::error::AppError;
use crate::models::CrawlConfig;

pub const PACKAGE_FORMAT: u32 = 1;
const DB_ENTRY: &str = "open-crawler.db";
const MANIFEST_ENTRY: &str = "manifest.json";

// ==================== Manifest ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestProject {
    pub id: String,
    pub name: String,
    pub page_count: u64,
    pub size_bytes: u64,
    pub has_html: bool,
    pub has_screenshots: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageManifest {
    pub format: u32,
    pub app_version: String,
    pub exported_at: String,
    pub sha256: String,
    pub include_credentials: bool,
    pub projects: Vec<ManifestProject>,
}

// ==================== Results ====================

#[derive(Debug, Clone, Serialize)]
pub struct ExportPackageInfo {
    pub path: String,
    pub file_name: String,
    pub size_bytes: u64,
    pub project_count: usize,
    pub lightweight: bool,
    pub include_credentials: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImportEntry {
    pub id: String,
    pub name: String,
    pub page_count: u64,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct ImportSummary {
    pub imported: Vec<ImportEntry>,
    pub skipped: Vec<ImportEntry>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportMode {
    Skip,
    Copy,
    Overwrite,
}

impl ImportMode {
    pub fn parse(s: &str) -> ImportMode {
        match s {
            "copy" => ImportMode::Copy,
            "overwrite" => ImportMode::Overwrite,
            _ => ImportMode::Skip,
        }
    }
}

// ==================== Work dir helper ====================

fn transfers_dir(app: &tauri::AppHandle) -> Result<PathBuf, AppError> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Crawl(e.to_string()))?
        .join("transfers");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

// ==================== Export ====================

pub fn export_package(
    repo: &CrawlRepo,
    app: &tauri::AppHandle,
    project_ids: Option<Vec<String>>,
    lightweight: bool,
    include_credentials: bool,
    dest_path: &Path,
) -> Result<ExportPackageInfo, AppError> {
    let work = transfers_dir(app)?;
    export_package_inner(
        repo,
        &work,
        project_ids,
        lightweight,
        include_credentials,
        dest_path,
    )
}

fn export_package_inner(
    repo: &CrawlRepo,
    work: &Path,
    project_ids: Option<Vec<String>>,
    lightweight: bool,
    include_credentials: bool,
    dest_path: &Path,
) -> Result<ExportPackageInfo, AppError> {
    let db_file = work.join(format!("export-src-{}.db", Uuid::new_v4()));
    let db_str = db_file.to_string_lossy().replace('\'', "''");

    // 1. Clean single-file snapshot (safe with WAL active).
    repo.connection()
        .execute_batch(&format!("VACUUM INTO '{}'", db_str))?;

    // 2. Open the snapshot and shape it.
    let src = Connection::open(&db_file)?;
    {
        let tx = src.unchecked_transaction()?;

        let all: Vec<String> = tx
            .prepare("SELECT id FROM projects ORDER BY name")?
            .query_map([], |r| r.get(0))?
            .filter_map(|r| r.ok())
            .collect();

        if let Some(keep) = &project_ids {
            let keep_set: std::collections::HashSet<String> = keep.iter().cloned().collect();
            for pid in &all {
                if !keep_set.contains(pid) {
                    delete_project_rows(&tx, pid)?;
                }
            }
        }

        if lightweight {
            tx.execute(
                "UPDATE crawled_pages SET html_body = NULL, screenshot_png = NULL",
                [],
            )?;
        }

        if !include_credentials {
            scrub_session_configs(&tx)?;
        }

        tx.commit()?;
    }

    // 3. Project metadata for the manifest.
    let mut projects = Vec::new();
    {
        let mut stmt = src.prepare(
            "SELECT p.id, p.name,
                    (SELECT COUNT(*) FROM crawled_pages c WHERE c.project_id = p.id),
                    COALESCE((SELECT SUM(LENGTH(c.html_body) + LENGTH(c.screenshot_png)) FROM crawled_pages c WHERE c.project_id = p.id), 0),
                    EXISTS(SELECT 1 FROM crawled_pages c WHERE c.project_id = p.id AND c.html_body IS NOT NULL LIMIT 1),
                    EXISTS(SELECT 1 FROM crawled_pages c WHERE c.project_id = p.id AND c.screenshot_png IS NOT NULL LIMIT 1)
             FROM projects p ORDER BY p.name",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok(ManifestProject {
                id: r.get(0)?,
                name: r.get(1)?,
                page_count: r.get(2)?,
                size_bytes: r.get(3)?,
                has_html: r.get::<_, i64>(4)? != 0,
                has_screenshots: r.get::<_, i64>(5)? != 0,
            })
        })?;
        for row in rows {
            projects.push(row?);
        }
    }
    drop(src);

    if projects.is_empty() {
        let _ = fs::remove_file(&db_file);
        return Err(AppError::Crawl(
            "No projects selected for export".to_string(),
        ));
    }

    // 4. SHA-256 of the DB snapshot.
    let sha256 = file_sha256(&db_file)?;

    let manifest = PackageManifest {
        format: PACKAGE_FORMAT,
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        exported_at: chrono::Utc::now().to_rfc3339(),
        sha256,
        include_credentials,
        projects,
    };

    // 5. Zip it all into the destination.
    let file = File::create(dest_path)?;
    let mut zip = zip::ZipWriter::new(file);
    let opts = SimpleFileOptions::default();
    zip.start_file(MANIFEST_ENTRY, opts)?;
    zip.write_all(serde_json::to_string_pretty(&manifest)?.as_bytes())?;
    zip.start_file(DB_ENTRY, opts)?;
    let mut db = File::open(&db_file)?;
    std::io::copy(&mut db, &mut zip)?;
    let inner = zip.finish()?;
    inner.sync_all()?;

    let _ = fs::remove_file(&db_file);

    let size_bytes = fs::metadata(dest_path)?.len();
    let file_name = dest_path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();

    info!(
        "Exported package {} with {} projects ({} bytes, lightweight={}, credentials={})",
        dest_path.display(),
        manifest.projects.len(),
        size_bytes,
        lightweight,
        include_credentials
    );

    Ok(ExportPackageInfo {
        path: dest_path.to_string_lossy().into_owned(),
        file_name,
        size_bytes,
        project_count: manifest.projects.len(),
        lightweight,
        include_credentials,
    })
}

fn file_sha256(path: &Path) -> Result<String, AppError> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 64 * 1024];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    let digest = hasher.finalize();
    Ok(digest.iter().map(|b| format!("{b:02x}")).collect())
}

/// Removes every table row belonging to a project, in FK-safe order.
fn delete_project_rows(conn: &Connection, id: &str) -> Result<(), AppError> {
    conn.execute(
        "DELETE FROM crawl_queue WHERE session_id IN (SELECT id FROM crawl_sessions WHERE project_id = ?1)",
        params![id],
    )?;
    conn.execute("DELETE FROM crawl_sessions WHERE project_id = ?1", params![id])?;
    conn.execute(
        "DELETE FROM page_links WHERE config_id IN (SELECT id FROM crawl_config WHERE project_id = ?1)",
        params![id],
    )?;
    conn.execute(
        "DELETE FROM crawl_errors WHERE config_id IN (SELECT id FROM crawl_config WHERE project_id = ?1)",
        params![id],
    )?;
    conn.execute(
        "DELETE FROM crawled_pages WHERE config_id IN (SELECT id FROM crawl_config WHERE project_id = ?1)",
        params![id],
    )?;
    conn.execute("DELETE FROM page_issues WHERE project_id = ?1", params![id])?;
    conn.execute(
        "DELETE FROM crawl_snapshot_data WHERE snapshot_id IN (SELECT id FROM crawl_snapshots WHERE project_id = ?1)",
        params![id],
    )?;
    conn.execute(
        "DELETE FROM crawl_snapshots WHERE project_id = ?1",
        params![id],
    )?;
    conn.execute("DELETE FROM crawl_config WHERE project_id = ?1", params![id])?;
    conn.execute("DELETE FROM projects WHERE id = ?1", params![id])?;
    Ok(())
}

/// Strips secret material (cookies, site auth, proxy, custom headers) from every
/// stored crawl session config before the package leaves the device.
fn scrub_session_configs(conn: &Connection) -> Result<(), AppError> {
    let rows: Vec<(String, String)> = conn
        .prepare("SELECT id, config_json FROM crawl_sessions")?
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
        .filter_map(|r| r.ok())
        .collect();

    let mut stmt = conn.prepare("UPDATE crawl_sessions SET config_json = ?1 WHERE id = ?2")?;
    for (id, json) in rows {
        match serde_json::from_str::<CrawlConfig>(&json) {
            Ok(mut cfg) => {
                cfg.cookies = Vec::new();
                cfg.site_auth = None;
                cfg.proxy = None;
                cfg.custom_headers = Vec::new();
                let clean = serde_json::to_string(&cfg)?;
                stmt.execute(params![clean, id])?;
            }
            Err(e) => warn!("Skipping config scrub for session {}: {}", id, e),
        }
    }
    Ok(())
}

// ==================== Import ====================

pub fn import_package(
    repo: &CrawlRepo,
    app: &tauri::AppHandle,
    file_path: &Path,
    mode: ImportMode,
) -> Result<ImportSummary, AppError> {
    let work = transfers_dir(app)?;
    import_package_inner(repo, &work, file_path, mode)
}

fn import_package_inner(
    repo: &CrawlRepo,
    work: &Path,
    file_path: &Path,
    mode: ImportMode,
) -> Result<ImportSummary, AppError> {
    let id = Uuid::new_v4();
    let dir = work.join(format!("import-{id}"));
    fs::create_dir_all(&dir)?;

    let manifest = read_package(file_path, &dir)?;

    // A package built with a future format version is not safe to import.
    if manifest.format > PACKAGE_FORMAT {
        let _ = fs::remove_dir_all(&dir);
        return Err(AppError::Crawl(format!(
            "Package format {} is newer than supported ({PACKAGE_FORMAT})",
            manifest.format
        )));
    }

    // Verify the checksum before touching any data.
    let db_path = dir.join(DB_ENTRY);
    let actual = file_sha256(&db_path)?;
    if actual != manifest.sha256 {
        let _ = fs::remove_dir_all(&dir);
        return Err(AppError::Crawl(
            "Package checksum mismatch — file is corrupted or tampered with".to_string(),
        ));
    }

    let src = Connection::open(&db_path)?;
    // Bring older snapshots up to the current schema before copying.
    crate::db::schema::run_migrations(&src)?;

    let mut summary = ImportSummary::default();
    copy_projects(&src, repo, &mut summary, mode)?;

    drop(src);
    let _ = fs::remove_dir_all(&dir);

    info!(
        "Imported package: {} imported, {} skipped",
        summary.imported.len(),
        summary.skipped.len()
    );
    Ok(summary)
}

/// Opens the package zip, extracts the two entries to `dir`, and returns the
/// parsed manifest. Only the expected entry names are touched (no path
/// traversal possible because nothing user-controlled is treated as a path).
fn read_package(file_path: &Path, dir: &Path) -> Result<PackageManifest, AppError> {
    let file = File::open(file_path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    let mut manifest: Option<PackageManifest> = None;
    let mut db_written = false;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        match entry.name() {
            MANIFEST_ENTRY => {
                let mut buf = String::new();
                entry.read_to_string(&mut buf)?;
                manifest = Some(serde_json::from_str(&buf)?);
            }
            DB_ENTRY => {
                let mut out = File::create(dir.join(DB_ENTRY))?;
                std::io::copy(&mut entry, &mut out)?;
                db_written = true;
            }
            _ => {} // ignore anything else
        }
    }

    let manifest = manifest.ok_or_else(|| AppError::Crawl("Package has no manifest".to_string()))?;
    if !db_written {
        return Err(AppError::Crawl("Package has no database".to_string()));
    }
    Ok(manifest)
}

#[derive(Debug)]
struct SrcProject {
    id: String,
    name: String,
    created_at: String,
    updated_at: String,
    page_count: u64,
}

/// Copies every project in `src` into the live database, re-keying all ids so
/// no collision with existing data is possible. Conflict handling is by project
/// *name*: `Skip` leaves it out, `Copy` duplicates it, `Overwrite` replaces it.
///
/// Copying is global (one pass over all source rows) rather than per-project.
/// Real databases contain cross-project references — pages and links whose
/// `config_id` points at the legacy `default` config that belongs to a
/// different project — which a per-project re-keying would orphan and then
/// fail with `FOREIGN KEY constraint failed`. Global maps let every reference
/// resolve to the freshly re-keyed parent row. A config whose owning project is
/// skipped is re-homed to the first imported project that references it;
/// orphaned queue / snapshot-data rows are dropped.
fn copy_projects(
    src: &Connection,
    repo: &CrawlRepo,
    summary: &mut ImportSummary,
    mode: ImportMode,
) -> Result<(), AppError> {
    let dest = repo.connection();

    let projects: Vec<SrcProject> = src
        .prepare(
            "SELECT p.id, p.name, p.created_at, p.updated_at,
                    (SELECT COUNT(*) FROM crawled_pages c WHERE c.project_id = p.id)
             FROM projects p ORDER BY p.name",
        )?
        .query_map([], |r| {
            Ok(SrcProject {
                id: r.get(0)?,
                name: r.get(1)?,
                created_at: r.get(2)?,
                updated_at: r.get(3)?,
                page_count: r.get(4)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    if projects.is_empty() {
        return Ok(());
    }

    let tx = dest.unchecked_transaction()?;

    // Re-key every source project, applying the conflict mode by name. Skipped
    // projects never reach `imported`, so none of their own rows are copied,
    // but they still get an entry in `project_map` so re-homed references from
    // imported projects can resolve to a valid parent row.
    let mut project_map: HashMap<String, String> = HashMap::new();
    let mut imported: HashSet<String> = HashSet::new();
    for project in &projects {
        let new_id = Uuid::new_v4().to_string();
        project_map.insert(project.id.clone(), new_id.clone());

        // The legacy 'default' placeholder is empty and meaningless to share.
        // It is still registered in `project_map` so re-homed references from
        // imported projects can resolve, but it is never copied itself.
        if project.id == "default" {
            continue;
        }

        if name_exists(dest, &project.name)? {
            match mode {
                ImportMode::Skip => {
                    summary.skipped.push(ImportEntry {
                        id: project.id.clone(),
                        name: project.name.clone(),
                        page_count: project.page_count,
                    });
                    continue;
                }
                ImportMode::Overwrite => {
                    if let Some(tid) = find_project_id_by_name(dest, &project.name)? {
                        delete_project_rows(&tx, &tid)?;
                    }
                }
                ImportMode::Copy => {}
            }
        }

        imported.insert(project.id.clone());
        tx.execute(
            "INSERT INTO projects (id, name, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
            params![
                new_id,
                project.name,
                project.created_at,
                project.updated_at
            ],
        )?;
    }

    if imported.is_empty() {
        tx.commit()?;
        return Ok(());
    }

    let config_map = copy_configs(src, &tx, &project_map, &imported)?;
    let session_map = copy_sessions(src, &tx, &project_map, &imported)?;
    let page_map = copy_pages(src, &tx, &project_map, &imported, &config_map)?;
    copy_page_links(src, &tx, &project_map, &imported, &config_map)?;
    copy_errors(src, &tx, &project_map, &imported, &config_map)?;
    copy_page_issues(src, &tx, &project_map, &imported, &page_map)?;
    copy_snapshots(src, &tx, &project_map, &imported, &page_map, &config_map)?;
    copy_queue(src, &tx, &imported, &session_map)?;

    tx.commit()?;

    for project in projects {
        if imported.contains(&project.id) {
            if let Some(new_id) = project_map.get(&project.id) {
                summary.imported.push(ImportEntry {
                    id: new_id.clone(),
                    name: project.name.clone(),
                    page_count: project.page_count,
                });
            }
        }
    }

    Ok(())
}

fn name_exists(conn: &Connection, name: &str) -> Result<bool, AppError> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM projects WHERE name = ?1 COLLATE NOCASE",
        params![name],
        |r| r.get(0),
    )?;
    Ok(count > 0)
}

fn find_project_id_by_name(conn: &Connection, name: &str) -> Result<Option<String>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id FROM projects WHERE name = ?1 COLLATE NOCASE ORDER BY rowid LIMIT 1",
    )?;
    let mut rows = stmt.query_map(params![name], |r| r.get(0))?;
    if let Some(id) = rows.next() {
        return Ok(Some(id?));
    }
    Ok(None)
}

/// Builds an `IN (?, ?, ...)` placeholder list of the given length.
fn in_placeholders(count: usize) -> String {
    vec!["?"; count].join(",")
}

/// Re-keys every `crawl_config` row globally. A config whose owning project is
/// skipped is re-homed to the first imported project that references it (via
/// pages, links or errors); a config referenced by nothing imported is dropped.
fn copy_configs(
    src: &Connection,
    tx: &rusqlite::Transaction<'_>,
    project_map: &HashMap<String, String>,
    imported: &HashSet<String>,
) -> Result<HashMap<String, String>, AppError> {
    let mut references: HashMap<String, Vec<Option<String>>> = HashMap::new();
    {
        let mut stmt = src.prepare(
            "SELECT DISTINCT config_id, project_id FROM crawled_pages
             UNION
             SELECT DISTINCT config_id, project_id FROM page_links
             UNION
             SELECT DISTINCT config_id, project_id FROM crawl_errors",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, Option<String>>(1)?,
            ))
        })?;
        for row in rows {
            let (config_id, project_id) = row?;
            references.entry(config_id).or_default().push(project_id);
        }
    }

    let mut stmt = src.prepare(
        "SELECT id, project_id, seed_urls, max_pages, max_depth, user_agent,
                respect_robots, created_at
         FROM crawl_config",
    )?;
    let mut rows = stmt.query([])?;
    let mut insert = tx.prepare(
        "INSERT INTO crawl_config
            (id, project_id, seed_urls, max_pages, max_depth, user_agent,
             respect_robots, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
    )?;
    let mut config_map: HashMap<String, String> = HashMap::new();

    while let Some(row) = rows.next()? {
        let old_id: String = row.get(0)?;
        let old_project: String = row.get(1)?;
        let new_id = Uuid::new_v4().to_string();
        config_map.insert(old_id.clone(), new_id.clone());

        let owner = if imported.contains(&old_project) {
            project_map.get(&old_project).cloned()
        } else {
            references
                .get(&old_id)
                .and_then(|pids| {
                    pids.iter().find_map(|p| match p {
                        Some(pid) if imported.contains(pid) => Some(pid),
                        _ => None,
                    })
                })
                .and_then(|p| project_map.get(p).cloned())
        };
        let Some(owner) = owner else { continue; };

        insert.execute(params![
            new_id,
            owner,
            row.get::<_, String>(2)?,
            row.get::<_, i64>(3)?,
            row.get::<_, i64>(4)?,
            row.get::<_, Option<String>>(5)?,
            row.get::<_, i64>(6)?,
            row.get::<_, String>(7)?,
        ])?;
    }
    Ok(config_map)
}

fn copy_sessions(
    src: &Connection,
    tx: &rusqlite::Transaction<'_>,
    project_map: &HashMap<String, String>,
    imported: &HashSet<String>,
) -> Result<HashMap<String, String>, AppError> {
    let mut stmt = src.prepare(&format!(
        "SELECT id, project_id, config_json, status, pages_crawled, errors,
                elapsed_secs, seed_urls, created_at, updated_at
         FROM crawl_sessions WHERE project_id IN ({})",
        in_placeholders(imported.len())
    ))?;
    let project_ids: Vec<&String> = imported.iter().collect();
    let mut rows = stmt.query(rusqlite::params_from_iter(project_ids))?;
    let mut insert = tx.prepare(
        "INSERT INTO crawl_sessions
            (id, project_id, config_json, status, pages_crawled, errors,
             elapsed_secs, seed_urls, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
    )?;
    let mut session_map: HashMap<String, String> = HashMap::new();

    while let Some(row) = rows.next()? {
        let old_id: String = row.get(0)?;
        let old_project: String = row.get(1)?;
        let new_id = Uuid::new_v4().to_string();
        let new_project = project_map.get(&old_project).cloned().unwrap_or_default();
        let config_json: String = row.get(2)?;
        // Point the embedded project_id at the freshly created project so
        // restoring the config later keeps working.
        let config_json = match serde_json::from_str::<CrawlConfig>(&config_json) {
            Ok(mut cfg) => {
                cfg.project_id = Some(new_project.clone());
                serde_json::to_string(&cfg).unwrap_or(config_json)
            }
            Err(_) => config_json,
        };
        insert.execute(params![
            new_id,
            new_project,
            config_json,
            row.get::<_, String>(3)?,
            row.get::<_, i64>(4)?,
            row.get::<_, i64>(5)?,
            row.get::<_, Option<f64>>(6)?,
            row.get::<_, String>(7)?,
            row.get::<_, String>(8)?,
            row.get::<_, String>(9)?,
        ])?;
        session_map.insert(old_id, new_id);
    }
    Ok(session_map)
}

fn copy_pages(
    src: &Connection,
    tx: &rusqlite::Transaction<'_>,
    project_map: &HashMap<String, String>,
    imported: &HashSet<String>,
    config_map: &HashMap<String, String>,
) -> Result<HashMap<String, String>, AppError> {
    let mut stmt = src.prepare(&format!(
        "SELECT id, config_id, project_id, url, status_code, title,
                meta_description, h1, canonical, size_bytes, load_time_ms,
                is_indexable, depth, parent_url, crawl_timestamp, html_lang,
                hreflang_json, semantic_issues_json, html_body, screenshot_png,
                readability_score, content_hash, duplicate_group_id,
                keywords_json, og_json, pagespeed_score, pagespeed_json, blocked
         FROM crawled_pages WHERE project_id IN ({})",
        in_placeholders(imported.len())
    ))?;
    let project_ids: Vec<&String> = imported.iter().collect();
    let mut rows = stmt.query(rusqlite::params_from_iter(project_ids))?;
    let mut insert = tx.prepare(
        "INSERT INTO crawled_pages
            (id, config_id, project_id, url, status_code, title, meta_description,
             h1, canonical, size_bytes, load_time_ms, is_indexable, depth,
             parent_url, crawl_timestamp, html_lang, hreflang_json,
             semantic_issues_json, html_body, screenshot_png, readability_score,
             content_hash, duplicate_group_id, keywords_json, og_json,
             pagespeed_score, pagespeed_json, blocked)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14,
                 ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28)",
    )?;
    let mut page_map: HashMap<String, String> = HashMap::new();

    while let Some(row) = rows.next()? {
        let old_page_id: String = row.get(0)?;
        let old_config_id: String = row.get(1)?;
        let old_project: String = row.get(2)?;
        let new_page_id = Uuid::new_v4().to_string();
        let new_config_id = config_map.get(&old_config_id).cloned().unwrap_or_default();
        let new_project = project_map.get(&old_project).cloned().unwrap_or_default();
        insert.execute(params![
            new_page_id,
            new_config_id,
            new_project,
            row.get::<_, String>(3)?,
            row.get::<_, Option<i64>>(4)?,
            row.get::<_, Option<String>>(5)?,
            row.get::<_, Option<String>>(6)?,
            row.get::<_, Option<String>>(7)?,
            row.get::<_, Option<String>>(8)?,
            row.get::<_, Option<i64>>(9)?,
            row.get::<_, Option<i64>>(10)?,
            row.get::<_, i64>(11)?,
            row.get::<_, Option<i64>>(12)?,
            row.get::<_, Option<String>>(13)?,
            row.get::<_, String>(14)?,
            row.get::<_, Option<String>>(15)?,
            row.get::<_, Option<String>>(16)?,
            row.get::<_, Option<String>>(17)?,
            row.get::<_, Option<String>>(18)?,
            row.get::<_, Option<Vec<u8>>>(19)?,
            row.get::<_, Option<f64>>(20)?,
            row.get::<_, Option<String>>(21)?,
            row.get::<_, Option<i64>>(22)?,
            row.get::<_, Option<String>>(23)?,
            row.get::<_, Option<String>>(24)?,
            row.get::<_, Option<f64>>(25)?,
            row.get::<_, Option<String>>(26)?,
            row.get::<_, i64>(27)?,
        ])?;
        page_map.insert(old_page_id, new_page_id);
    }
    Ok(page_map)
}

fn copy_page_links(
    src: &Connection,
    tx: &rusqlite::Transaction<'_>,
    project_map: &HashMap<String, String>,
    imported: &HashSet<String>,
    config_map: &HashMap<String, String>,
) -> Result<(), AppError> {
    if config_map.is_empty() {
        return Ok(());
    }
    let mut stmt = src.prepare(&format!(
        "SELECT from_url, to_url, config_id, project_id, link_type, anchor_text, is_follow
         FROM page_links WHERE config_id IN ({})",
        in_placeholders(config_map.len())
    ))?;
    let config_ids: Vec<&String> = config_map.keys().collect();
    let mut rows = stmt.query(rusqlite::params_from_iter(config_ids))?;
    let mut insert = tx.prepare(
        "INSERT INTO page_links (from_url, to_url, config_id, project_id, link_type, anchor_text, is_follow)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
    )?;
    while let Some(row) = rows.next()? {
        let old_config_id: String = row.get(2)?;
        let old_project: Option<String> = row.get(3)?;
        let new_config_id = config_map.get(&old_config_id).cloned().unwrap_or_default();
        let new_project = old_project
            .and_then(|p| imported.contains(&p).then(|| project_map.get(&p).cloned()))
            .flatten();
        insert.execute(params![
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            new_config_id,
            new_project,
            row.get::<_, String>(4)?,
            row.get::<_, Option<String>>(5)?,
            row.get::<_, Option<i64>>(6)?,
        ])?;
    }
    Ok(())
}

fn copy_errors(
    src: &Connection,
    tx: &rusqlite::Transaction<'_>,
    project_map: &HashMap<String, String>,
    imported: &HashSet<String>,
    config_map: &HashMap<String, String>,
) -> Result<(), AppError> {
    if config_map.is_empty() {
        return Ok(());
    }
    let mut stmt = src.prepare(&format!(
        "SELECT url, config_id, project_id, error_type, error_message, timestamp
         FROM crawl_errors WHERE config_id IN ({})",
        in_placeholders(config_map.len())
    ))?;
    let config_ids: Vec<&String> = config_map.keys().collect();
    let mut rows = stmt.query(rusqlite::params_from_iter(config_ids))?;
    let mut insert = tx.prepare(
        "INSERT INTO crawl_errors (url, config_id, project_id, error_type, error_message, timestamp)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
    )?;
    while let Some(row) = rows.next()? {
        let old_config_id: String = row.get(1)?;
        let old_project: Option<String> = row.get(2)?;
        let new_config_id = config_map.get(&old_config_id).cloned().unwrap_or_default();
        let new_project = old_project
            .and_then(|p| imported.contains(&p).then(|| project_map.get(&p).cloned()))
            .flatten();
        insert.execute(params![
            row.get::<_, String>(0)?,
            new_config_id,
            new_project,
            row.get::<_, String>(3)?,
            row.get::<_, Option<String>>(4)?,
            row.get::<_, String>(5)?,
        ])?;
    }
    Ok(())
}

fn copy_page_issues(
    src: &Connection,
    tx: &rusqlite::Transaction<'_>,
    project_map: &HashMap<String, String>,
    imported: &HashSet<String>,
    page_map: &HashMap<String, String>,
) -> Result<(), AppError> {
    let mut stmt = src.prepare(&format!(
        "SELECT page_id, issue_type, severity, message, element, css_selector,
                xpath, position, project_id
         FROM page_issues WHERE project_id IN ({})",
        in_placeholders(imported.len())
    ))?;
    let project_ids: Vec<&String> = imported.iter().collect();
    let mut rows = stmt.query(rusqlite::params_from_iter(project_ids))?;
    let mut insert = tx.prepare(
        "INSERT INTO page_issues
            (project_id, page_id, issue_type, severity, message, element,
             css_selector, xpath, position)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
    )?;
    while let Some(row) = rows.next()? {
        let old_page_id: String = row.get(0)?;
        let new_page_id = page_map.get(&old_page_id).cloned().unwrap_or_default();
        if new_page_id.is_empty() {
            continue; // page belongs to a skipped project
        }
        let old_project: String = row.get(8)?;
        let new_project = project_map.get(&old_project).cloned().unwrap_or_default();
        insert.execute(params![
            new_project,
            new_page_id,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, Option<String>>(3)?,
            row.get::<_, Option<String>>(4)?,
            row.get::<_, Option<String>>(5)?,
            row.get::<_, Option<String>>(6)?,
            row.get::<_, i64>(7)?,
        ])?;
    }
    Ok(())
}

fn copy_snapshots(
    src: &Connection,
    tx: &rusqlite::Transaction<'_>,
    project_map: &HashMap<String, String>,
    imported: &HashSet<String>,
    page_map: &HashMap<String, String>,
    config_map: &HashMap<String, String>,
) -> Result<(), AppError> {
    let mut snapshot_map: HashMap<String, String> = HashMap::new();
    {
        let mut stmt = src.prepare(&format!(
            "SELECT id, project_id, config_id, snapshot_time, total_pages,
                    indexed_pages, broken_pages, avg_load_ms, avg_size_bytes,
                    avg_readability, status_counts_json
             FROM crawl_snapshots WHERE project_id IN ({})",
            in_placeholders(imported.len())
        ))?;
        let project_ids: Vec<&String> = imported.iter().collect();
        let mut rows = stmt.query(rusqlite::params_from_iter(project_ids))?;
        let mut insert = tx.prepare(
            "INSERT INTO crawl_snapshots
                (id, project_id, config_id, snapshot_time, total_pages,
                 indexed_pages, broken_pages, avg_load_ms, avg_size_bytes,
                 avg_readability, status_counts_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        )?;
        while let Some(row) = rows.next()? {
            let old_sid: String = row.get(0)?;
            let old_project: String = row.get(1)?;
            let old_config: String = row.get(2)?;
            let new_sid = Uuid::new_v4().to_string();
            let new_project = project_map.get(&old_project).cloned().unwrap_or_default();
            let new_config = config_map.get(&old_config).cloned().unwrap_or_default();
            insert.execute(params![
                new_sid,
                new_project,
                new_config,
                row.get::<_, String>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, i64>(5)?,
                row.get::<_, i64>(6)?,
                row.get::<_, Option<f64>>(7)?,
                row.get::<_, Option<f64>>(8)?,
                row.get::<_, Option<f64>>(9)?,
                row.get::<_, Option<String>>(10)?,
            ])?;
            snapshot_map.insert(old_sid, new_sid);
        }
    }

    let mut stmt = src.prepare(
        "SELECT snapshot_id, page_id, url, status_code, title, meta_description,
                size_bytes, load_time_ms, is_indexable, readability_score
         FROM crawl_snapshot_data
         WHERE snapshot_id IN (SELECT id FROM crawl_snapshots)",
    )?;
    let mut rows = stmt.query([])?;
    let mut insert = tx.prepare(
        "INSERT INTO crawl_snapshot_data
            (snapshot_id, page_id, url, status_code, title, meta_description,
             size_bytes, load_time_ms, is_indexable, readability_score)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
    )?;
    while let Some(row) = rows.next()? {
        let old_sid: String = row.get(0)?;
        let old_page_id: String = row.get(1)?;
        let new_sid = snapshot_map.get(&old_sid).cloned().unwrap_or_default();
        if new_sid.is_empty() {
            continue; // snapshot belongs to a skipped project
        }
        let new_page_id = page_map.get(&old_page_id).cloned().unwrap_or_default();
        if new_page_id.is_empty() {
            continue; // page belongs to a skipped project
        }
        insert.execute(params![
            new_sid,
            new_page_id,
            row.get::<_, String>(2)?,
            row.get::<_, Option<i64>>(3)?,
            row.get::<_, Option<String>>(4)?,
            row.get::<_, Option<String>>(5)?,
            row.get::<_, Option<i64>>(6)?,
            row.get::<_, Option<i64>>(7)?,
            row.get::<_, Option<i64>>(8)?,
            row.get::<_, Option<f64>>(9)?,
        ])?;
    }
    Ok(())
}

fn copy_queue(
    src: &Connection,
    tx: &rusqlite::Transaction<'_>,
    imported: &HashSet<String>,
    session_map: &HashMap<String, String>,
) -> Result<(), AppError> {
    let mut stmt = src.prepare(&format!(
        "SELECT q.session_id, q.url, q.depth
         FROM crawl_queue q
         WHERE q.session_id IN (SELECT id FROM crawl_sessions WHERE project_id IN ({}))",
        in_placeholders(imported.len())
    ))?;
    let project_ids: Vec<&String> = imported.iter().collect();
    let mut rows = stmt.query(rusqlite::params_from_iter(project_ids))?;
    let mut insert = tx.prepare(
        "INSERT INTO crawl_queue (session_id, url, depth)
         VALUES (?1, ?2, ?3)",
    )?;
    while let Some(row) = rows.next()? {
        let old_sid: String = row.get(0)?;
        let new_sid = session_map.get(&old_sid).cloned().unwrap_or_default();
        if new_sid.is_empty() {
            continue; // session belongs to a skipped project
        }
        insert.execute(params![
            new_sid,
            row.get::<_, String>(1)?,
            row.get::<_, i64>(2)?,
        ])?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::schema::run_migrations;
    use crate::db::CrawlRepo;

    struct TempDir(std::path::PathBuf);
    impl TempDir {
        fn new(tag: &str) -> Self {
            let p = std::env::temp_dir().join(format!(
                "ocproj-test-{tag}-{}-{}",
                std::process::id(),
                Uuid::new_v4()
            ));
            fs::create_dir_all(&p).unwrap();
            TempDir(p)
        }
    }
    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn temp_conn(path: &Path) -> Connection {
        let conn = Connection::open(path).unwrap();
        run_migrations(&conn).unwrap();
        conn
    }

    /// Removes the placeholder 'default' project from a *source* DB so exported
    /// packages only contain the projects the test seeds.
    fn rm_default(conn: &Connection) {
        conn.execute("DELETE FROM projects WHERE id = 'default'", [])
            .unwrap();
    }

    fn hex(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{b:02x}")).collect()
    }

    fn seed_project(conn: &Connection, name: &str) -> String {
        let pid = format!("proj-{name}");
        conn.execute_batch(&format!(
            "INSERT INTO projects (id, name, created_at, updated_at)
                 VALUES ('{pid}', '{name}', datetime('now'), datetime('now'));
             INSERT INTO crawl_config (id, project_id, seed_urls, max_pages, created_at)
                 VALUES ('cfg-{pid}', '{pid}', '[\"https://x.com\"]', 10, datetime('now'));
             INSERT INTO crawl_sessions
                 (id, project_id, config_json, status, pages_crawled, errors,
                  elapsed_secs, seed_urls, created_at, updated_at)
                 VALUES ('sess-{pid}', '{pid}', '{{\"id\":\"cfg-{pid}\",\"project_id\":\"{pid}\",\"seed_urls\":[\"https://x.com\"],\"cookies\":[\"session=abc\"],\"custom_headers\":[[\"X-A\",\"1\"]]}}',
                         'completed', 2, 0, 5, '[\"https://x.com\"]', datetime('now'), datetime('now'));
             INSERT INTO crawled_pages
                 (id, config_id, project_id, url, status_code, title, is_indexable, depth, crawl_timestamp, html_body)
                 VALUES ('page-{pid}-1', 'cfg-{pid}', '{pid}', 'https://x.com/a', 200, 'A', 1, 0,
                         datetime('now'), '<html><title>A</title></html>'),
                        ('page-{pid}-2', 'cfg-{pid}', '{pid}', 'https://x.com/b', 404, 'B', 1, 1,
                         datetime('now'), '<html><title>B</title></html>');
             INSERT INTO page_links (from_url, to_url, config_id, link_type)
                 VALUES ('https://x.com/a', 'https://x.com/b', 'cfg-{pid}', 'href');
             INSERT INTO crawl_errors (url, config_id, error_type, error_message, timestamp)
                 VALUES ('https://x.com/b', 'cfg-{pid}', 'status', '404', datetime('now'));
             INSERT INTO page_issues (project_id, page_id, issue_type, severity, message)
                 VALUES ('{pid}', 'page-{pid}-1', 'missing_meta', 'warning', 'No meta');
             INSERT INTO crawl_snapshots
                 (id, project_id, config_id, snapshot_time, total_pages, indexed_pages)
                 VALUES ('snap-{pid}', '{pid}', 'cfg-{pid}', datetime('now'), 2, 1);
             INSERT INTO crawl_snapshot_data (snapshot_id, page_id, url, title)
                 VALUES ('snap-{pid}', 'page-{pid}-1', 'https://x.com/a', 'A');
             INSERT INTO crawl_queue (session_id, url, depth)
                 VALUES ('sess-{pid}', 'https://x.com/c', 0);"
        ))
        .unwrap();
        pid
    }

    fn counts(conn: &Connection, _pid: &str) -> (i64, i64, i64, i64, i64, i64, i64, i64) {
        let q = |sql: &str| conn.query_row(sql, [], |r| r.get::<_, i64>(0)).unwrap();
        (
            q("SELECT COUNT(*) FROM crawled_pages"),
            q("SELECT COUNT(*) FROM page_links"),
            q("SELECT COUNT(*) FROM crawl_errors"),
            q("SELECT COUNT(*) FROM page_issues"),
            q("SELECT COUNT(*) FROM crawl_snapshots"),
            q("SELECT COUNT(*) FROM crawl_snapshot_data"),
            q("SELECT COUNT(*) FROM crawl_sessions"),
            q("SELECT COUNT(*) FROM crawl_queue"),
        )
    }

    #[test]
    fn test_import_rehomes_config_referenced_across_projects() {
        // Regression: a package where pages/links in one project reference the
        // legacy 'default' config owned by the (skipped) 'default' project used
        // to fail with `FOREIGN KEY constraint failed`. The config must be
        // re-homed to the importing project instead of being dropped.
        let work = TempDir::new("xh");
        let src_path = work.0.join("src.db");
        let pkg_path = work.0.join("out.ocproj");

        let src_conn = temp_conn(&src_path);
        src_conn
            .execute(
                "INSERT INTO crawl_config (id, project_id, seed_urls, max_pages, created_at)
                 VALUES ('cfg-shared', 'default', '[\"https://x.com\"]', 10, datetime('now'))",
                [],
            )
            .unwrap();
        src_conn
            .execute_batch(
                "INSERT INTO projects (id, name, created_at, updated_at)
                 VALUES ('proj-afa', 'afa', datetime('now'), datetime('now'));
                 INSERT INTO crawled_pages
                     (id, config_id, project_id, url, status_code, is_indexable, crawl_timestamp)
                     VALUES ('page-afa-1', 'cfg-shared', 'proj-afa', 'https://x.com/a', 200, 1,
                             datetime('now')),
                            ('page-afa-2', 'cfg-shared', 'proj-afa', 'https://x.com/b', 404, 0,
                             datetime('now'));
                 INSERT INTO page_links (from_url, to_url, config_id, project_id, link_type)
                     VALUES ('https://x.com/a', 'https://x.com/b', 'cfg-shared', 'proj-afa', 'href');",
            )
            .unwrap();

        let repo = CrawlRepo::new(&src_conn, None);
        export_package_inner(&repo, &work.0, None, false, false, &pkg_path).unwrap();

        let dest_conn = temp_conn(&work.0.join("dest.db"));
        let dest_repo = CrawlRepo::new(&dest_conn, None);
        let summary = import_package_inner(&dest_repo, &work.0, &pkg_path, ImportMode::Skip).unwrap();

        assert_eq!(summary.imported.len(), 1, "afa imports, legacy default is skipped");
        assert_eq!(summary.imported[0].name, "afa");
        assert!(summary.warnings.is_empty());

        let (configs, config_project, pages, links): (i64, Option<String>, i64, i64) = dest_conn
            .query_row(
                "SELECT (SELECT COUNT(*) FROM crawl_config),
                        (SELECT project_id FROM crawl_config),
                        (SELECT COUNT(*) FROM crawled_pages),
                        (SELECT COUNT(*) FROM page_links)",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
            .unwrap();
        assert_eq!(configs, 1);
        assert_eq!(config_project.as_deref(), Some(summary.imported[0].id.as_str()));
        assert_eq!(pages, 2);
        assert_eq!(links, 1);

        // The imported pages point at the re-homed config.
        let dest_cfg: String = dest_conn
            .query_row("SELECT id FROM crawl_config LIMIT 1", [], |r| r.get(0))
            .unwrap();
        let page_cfg: String = dest_conn
            .query_row(
                "SELECT config_id FROM crawled_pages WHERE url = 'https://x.com/a'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(page_cfg, dest_cfg);
    }

    #[test]
    fn test_round_trip_preserves_all_tables() {
        let work = TempDir::new("rt");
        let src_path = work.0.join("src.db");
        let pkg_path = work.0.join("out.ocproj");

        let src_conn = temp_conn(&src_path);
        rm_default(&src_conn);
        let pid = seed_project(&src_conn, "Site A");
        let (_, src_links, src_errors, src_issues, src_snaps, src_snapdata, src_sess, src_queue) =
            counts(&src_conn, &pid);

        let repo = CrawlRepo::new(&src_conn, None);
        export_package_inner(
            &repo,
            &work.0,
            None,
            false,
            false,
            &pkg_path,
        )
        .unwrap();

        // Zip contains exactly the two entries and a correct checksum.
        let archive = File::open(&pkg_path).unwrap();
        let mut zip = zip::ZipArchive::new(archive).unwrap();
        let names: Vec<String> = zip.file_names().map(str::to_string).collect();
        assert_eq!(names, vec!["manifest.json", DB_ENTRY]);
        let mut mf = zip.by_name(MANIFEST_ENTRY).unwrap();
        let mut raw = String::new();
        mf.read_to_string(&mut raw).unwrap();
        let manifest: PackageManifest = serde_json::from_str(&raw).unwrap();
        assert_eq!(manifest.format, PACKAGE_FORMAT);
        assert_eq!(manifest.projects.len(), 1);
        assert_eq!(manifest.projects[0].page_count, 2);
        assert_eq!(manifest.projects[0].has_html, true);
        drop(mf);

        let mut db = zip.by_name(DB_ENTRY).unwrap();
        let mut buf = Vec::new();
        db.read_to_end(&mut buf).unwrap();
        assert_eq!(hex(&sha2::Sha256::digest(&buf)), manifest.sha256);
        drop(db);
        drop(zip);

        // Import into a fresh database and check every table is repopulated.
        let dest_path = work.0.join("dest.db");
        let dest_conn = temp_conn(&dest_path);
        let dest_repo = CrawlRepo::new(&dest_conn, None);
        let summary = import_package_inner(&dest_repo, &work.0, &pkg_path, ImportMode::Skip).unwrap();
        assert_eq!(summary.imported.len(), 1);
        assert!(summary.skipped.is_empty());
        assert!(summary.warnings.is_empty());

        let new_pid = &summary.imported[0].id;
        let new_counts = counts(&dest_conn, new_pid);
        assert_eq!(new_counts.0, 2);
        assert_eq!(new_counts.1, src_links);
        assert_eq!(new_counts.2, src_errors);
        assert_eq!(new_counts.3, src_issues);
        assert_eq!(new_counts.4, src_snaps);
        assert_eq!(new_counts.5, src_snapdata);
        assert_eq!(new_counts.6, src_sess);
        assert_eq!(new_counts.7, src_queue);

        // Pages keep their rows but point at the re-keyed config/project.
        let (url, cfg_id, project_id): (String, String, String) = dest_conn
            .query_row(
                "SELECT url, config_id, project_id FROM crawled_pages WHERE url = 'https://x.com/a'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .unwrap();
        assert_eq!(url, "https://x.com/a");
        assert_eq!(project_id, *new_pid);
        assert_ne!(cfg_id, "cfg-proj-Site A");
    }

    #[test]
    fn test_skip_mode_conflict_by_name() {
        let work = TempDir::new("skip");
        let src_path = work.0.join("src.db");
        let pkg_path = work.0.join("out.ocproj");

        let src_conn = temp_conn(&src_path);
        rm_default(&src_conn);
        let repo = CrawlRepo::new(&src_conn, None);
        seed_project(&src_conn, "Dup");
        export_package_inner(&repo, &work.0, None, false, false, &pkg_path).unwrap();

        let dest_conn = temp_conn(&work.0.join("dest.db"));
        seed_project(&dest_conn, "Dup");
        let dest_repo = CrawlRepo::new(&dest_conn, None);
        let summary = import_package_inner(&dest_repo, &work.0, &pkg_path, ImportMode::Skip).unwrap();
        assert!(summary.imported.is_empty());
        assert_eq!(summary.skipped.len(), 1);
        assert_eq!(summary.skipped[0].name, "Dup");
        let (pages, _, _, _, _, _, _, _) = counts(&dest_conn, "proj-Dup");
        assert_eq!(pages, 2, "existing project must stay untouched");
    }

    #[test]
    fn test_copy_mode_duplicates_conflicting_project() {
        let work = TempDir::new("copy");
        let src_path = work.0.join("src.db");
        let pkg_path = work.0.join("out.ocproj");

        let src_conn = temp_conn(&src_path);
        rm_default(&src_conn);
        let repo = CrawlRepo::new(&src_conn, None);
        seed_project(&src_conn, "Dup");
        export_package_inner(&repo, &work.0, None, false, false, &pkg_path).unwrap();

        let dest_conn = temp_conn(&work.0.join("dest.db"));
        seed_project(&dest_conn, "Dup");
        let dest_repo = CrawlRepo::new(&dest_conn, None);
        let summary = import_package_inner(&dest_repo, &work.0, &pkg_path, ImportMode::Copy).unwrap();
        assert_eq!(summary.imported.len(), 1);
        let name_count: i64 = dest_conn
            .query_row("SELECT COUNT(*) FROM projects WHERE name = 'Dup'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(name_count, 2);
        // Total pages = 2 (existing) + 2 (imported duplicate).
        let total_pages: i64 = dest_conn
            .query_row("SELECT COUNT(*) FROM crawled_pages", [], |r| r.get(0))
            .unwrap();
        assert_eq!(total_pages, 4);
    }

    #[test]
    fn test_overwrite_mode_replaces_conflicting_project() {
        let work = TempDir::new("ow");
        let src_path = work.0.join("src.db");
        let pkg_path = work.0.join("out.ocproj");

        let src_conn = temp_conn(&src_path);
        rm_default(&src_conn);
        let repo = CrawlRepo::new(&src_conn, None);
        seed_project(&src_conn, "Repl");
        export_package_inner(&repo, &work.0, None, false, false, &pkg_path).unwrap();

        let dest_conn = temp_conn(&work.0.join("dest.db"));
        seed_project(&dest_conn, "Repl");
        let dest_repo = CrawlRepo::new(&dest_conn, None);
        let summary = import_package_inner(&dest_repo, &work.0, &pkg_path, ImportMode::Overwrite).unwrap();
        assert_eq!(summary.imported.len(), 1);
        let name_count: i64 = dest_conn
            .query_row("SELECT COUNT(*) FROM projects WHERE name = 'Repl'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(name_count, 1);
        let total_pages: i64 = dest_conn
            .query_row("SELECT COUNT(*) FROM crawled_pages", [], |r| r.get(0))
            .unwrap();
        assert_eq!(total_pages, 2);
    }

    #[test]
    fn test_export_filtered_projects_excludes_others() {
        let work = TempDir::new("filter");
        let src_path = work.0.join("src.db");
        let pkg_path = work.0.join("out.ocproj");

        let src_conn = temp_conn(&src_path);
        rm_default(&src_conn);
        let repo = CrawlRepo::new(&src_conn, None);
        seed_project(&src_conn, "Keep");
        seed_project(&src_conn, "Drop");
        export_package_inner(&repo, &work.0, Some(vec!["proj-Keep".into()]), false, false, &pkg_path)
            .unwrap();

        let dest_conn = temp_conn(&work.0.join("dest.db"));
        let dest_repo = CrawlRepo::new(&dest_conn, None);
        let summary = import_package_inner(&dest_repo, &work.0, &pkg_path, ImportMode::Skip).unwrap();
        assert_eq!(summary.imported.len(), 1);
        assert_eq!(summary.imported[0].name, "Keep");
    }

    #[test]
    fn test_lightweight_strips_html_and_screenshots() {
        let work = TempDir::new("light");
        let src_path = work.0.join("src.db");
        let pkg_path = work.0.join("out.ocproj");

        let src_conn = temp_conn(&src_path);
        rm_default(&src_conn);
        let repo = CrawlRepo::new(&src_conn, None);
        seed_project(&src_conn, "Lite");
        export_package_inner(&repo, &work.0, None, true, false, &pkg_path).unwrap();

        let dest_conn = temp_conn(&work.0.join("dest.db"));
        let dest_repo = CrawlRepo::new(&dest_conn, None);
        let summary = import_package_inner(&dest_repo, &work.0, &pkg_path, ImportMode::Skip).unwrap();
        let null_count: i64 = dest_conn
            .query_row(
                "SELECT COUNT(*) FROM crawled_pages WHERE html_body IS NOT NULL",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(null_count, 0);
        let _ = summary;
    }

    #[test]
    fn test_credentials_scrubbed_by_default() {
        let work = TempDir::new("scrub");
        let src_path = work.0.join("src.db");
        let pkg_path = work.0.join("out.ocproj");

        let src_conn = temp_conn(&src_path);
        rm_default(&src_conn);
        let repo = CrawlRepo::new(&src_conn, None);
        seed_project(&src_conn, "Sec");
        export_package_inner(&repo, &work.0, None, false, false, &pkg_path).unwrap();

        let dest_conn = temp_conn(&work.0.join("dest.db"));
        let dest_repo = CrawlRepo::new(&dest_conn, None);
        import_package_inner(&dest_repo, &work.0, &pkg_path, ImportMode::Skip).unwrap();

        let (cookies, headers): (String, String) = dest_conn
            .query_row(
                "SELECT json_extract(config_json, '$.cookies'), json_extract(config_json, '$.custom_headers')
                 FROM crawl_sessions LIMIT 1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(cookies, "[]");
        assert_eq!(headers, "[]");
    }

    #[test]
    fn test_corrupted_package_is_rejected() {
        let work = TempDir::new("corrupt");
        let src_path = work.0.join("src.db");
        let pkg_path = work.0.join("bad.ocproj");

        let src_conn = temp_conn(&src_path);
        rm_default(&src_conn);
        let repo = CrawlRepo::new(&src_conn, None);
        seed_project(&src_conn, "Ok");
        export_package_inner(&repo, &work.0, None, false, false, &pkg_path).unwrap();

        // Tamper with the DB entry so the checksum no longer matches.
        fs::write(&pkg_path, b"garbage").unwrap();
        let dest_conn = Connection::open(work.0.join("dest.db")).unwrap();
        dest_conn
            .execute_batch(
                "CREATE TABLE projects (id TEXT PRIMARY KEY, name TEXT NOT NULL,
                                        created_at TEXT NOT NULL, updated_at TEXT NOT NULL);",
            )
            .unwrap();
        let dest_repo = CrawlRepo::new(&dest_conn, None);
        let err = import_package_inner(&dest_repo, &work.0, &pkg_path, ImportMode::Skip)
            .err()
            .expect("import must fail");
        assert!(err.to_string().contains("checksum") || err.to_string().contains("archive"));
        let project_count: i64 = dest_conn
            .query_row("SELECT COUNT(*) FROM projects", [], |r| r.get(0))
            .unwrap();
        assert_eq!(project_count, 0, "no data must be imported on failure");
    }
}
