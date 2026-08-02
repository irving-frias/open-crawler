pub mod commands;
pub mod crawler;
pub mod db;
pub mod error;
pub mod models;
pub mod nesting_table;

use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};
use tauri::Manager;
use tokio::sync::RwLock;
use tracing::info;

use crate::models::CrawlProgress;

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct ResultsCacheKey {
    pub project_id: String,
    pub page: u32,
    pub page_size: u32,
    pub semantic_issue_type: Option<String>,
    pub search: Option<String>,
    pub status_filter: Vec<u32>,
    pub severity_filter: Vec<String>,
    pub domain_filter: Option<String>,
    pub depth_filter: Option<u32>,
    pub missing_title: bool,
    pub duplicate_title: bool,
    pub noindex_only: bool,
    pub is_404: bool,
}

pub struct CrawlState {
    pub cancellation: tokio_util::sync::CancellationToken,
    pub progress: CrawlProgress,
}

type ResultsCacheArc = std::sync::Arc<std::sync::Mutex<lru::LruCache<ResultsCacheKey, (Vec<crate::models::CrawlResult>, u32)>>>;

pub struct AppState {
    pub db: Mutex<rusqlite::Connection>,
    pub crawls: Arc<RwLock<HashMap<String, CrawlState>>>,
    pub results_cache: ResultsCacheArc,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    info!("Starting Open Crawler");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_share::init())
        .setup(|app| {
            let app_data_dir = match app.path().app_data_dir() {
                Ok(dir) => dir,
                Err(e) => {
                    tracing::error!("Failed to get app data dir: {}", e);
                    return Err(e.into());
                }
            };

            if let Err(e) = std::fs::create_dir_all(&app_data_dir) {
                tracing::error!("Failed to create app data dir: {}", e);
                return Err(e.into());
            }

            let db_path = app_data_dir.join("open-crawler.db");
            info!("Opening database at: {:?}", db_path);

            let conn = match rusqlite::Connection::open(&db_path) {
                Ok(conn) => conn,
                Err(e) => {
                    tracing::error!("Failed to open database: {}", e);
                    return Err(Box::new(e) as Box<dyn std::error::Error>);
                }
            };

            if let Err(e) = conn.pragma_update(None, "journal_mode", "WAL") {
                tracing::error!("Failed to enable WAL mode: {}", e);
                return Err(Box::new(e) as Box<dyn std::error::Error>);
            }
            if let Err(e) = conn.pragma_update(None, "busy_timeout", 5000) {
                tracing::error!("Failed to set busy_timeout: {}", e);
                return Err(Box::new(e) as Box<dyn std::error::Error>);
            }

            // Run migrations
            if let Err(e) = crate::db::schema::run_migrations(&conn) {
                tracing::error!("Failed to run migrations: {}", e);
                return Err(Box::new(e) as Box<dyn std::error::Error>);
            }

            let state = AppState {
                db: Mutex::new(conn),
                crawls: Arc::new(RwLock::new(HashMap::new())),
                results_cache: Arc::new(Mutex::new(lru::LruCache::new(NonZeroUsize::new(512).unwrap()))),
            };

            app.manage(Arc::new(RwLock::new(state)));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            crate::commands::is_mobile,
            crate::commands::create_project,
            crate::commands::list_projects,
            crate::commands::get_project,
            crate::commands::rename_project,
            crate::commands::delete_project,
            crate::commands::get_project_stats,
            crate::commands::start_crawl,
            crate::commands::stop_crawl,
            crate::commands::get_crawl_status,
            crate::commands::get_running_crawls,
            crate::commands::check_resumable_crawl,
            crate::commands::get_results,
            crate::commands::get_site_tree,
            crate::commands::get_page_detail,
            crate::commands::get_semantic_issue_counts,
            crate::commands::get_page_html,
            crate::commands::inline_assets,
            crate::commands::recrawl_page,
            crate::commands::capture_page_screenshot,
            crate::commands::get_settings,
            crate::commands::save_settings,
            crate::commands::export_full,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
