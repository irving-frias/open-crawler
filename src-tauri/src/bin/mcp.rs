//! `open-crawler-mcp`: MCP server (stdio) + headless crawl runner (T6).
//!
//! Shares the desktop app's SQLite database (`<data_dir>/com.opencrawler.app/
//! open-crawler.db`, overridable with `--db` / `OPEN_CRAWLER_DB`).
//!
//! Usage:
//!   open-crawler-mcp                  # MCP server on stdio (default)
//!   open-crawler-mcp serve [--db P]
//!   open-crawler-mcp crawl --config crawl.json [--db P] [--project-name N]
//!
//! The `crawl` mode runs one crawl from a JSON config file to completion and
//! prints a JSON summary, for CI/scripts without an MCP client.

use std::path::PathBuf;
use std::sync::Arc;

use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{Implementation, ServerCapabilities, ServerConfig},
    tool, tool_handler, tool_router, ErrorData, ServerHandler, ServiceExt,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{error, info};

use open_crawler_lib::{
    db::CrawlRepo,
    error::AppError,
    features::crawl::commands::{run_crawl_blocking, start_crawl_headless},
    models::{CrawlConfig, Project},
    open_app_state, AppState,
};

const DEFAULT_DB_SUFFIX: [&str; 2] = ["com.opencrawler.app", "open-crawler.db"];

/// Resolve the database path: `--db`, else `OPEN_CRAWLER_DB`, else the
/// desktop app's database.
fn resolve_db_path(explicit: Option<PathBuf>) -> PathBuf {
    if let Some(p) = explicit {
        return p;
    }
    if let Ok(p) = std::env::var("OPEN_CRAWLER_DB") {
        if !p.is_empty() {
            return PathBuf::from(p);
        }
    }
    let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join(DEFAULT_DB_SUFFIX[0]).join(DEFAULT_DB_SUFFIX[1])
}

/// Run a repo query against the shared state (mirrors `with_repo_arc`,
/// which is crate-private).
async fn with_repo<T, F>(state: &Arc<RwLock<AppState>>, f: F) -> Result<T, AppError>
where
    F: FnOnce(&CrawlRepo) -> Result<T, AppError>,
{
    let guard = state.read().await;
    let db = guard
        .db
        .lock()
        .map_err(|_| AppError::Crawl("Database lock poisoned".into()))?;
    let repo = CrawlRepo::new(&db, None);
    f(&repo)
}

fn invalid_params(msg: String) -> ErrorData {
    ErrorData::invalid_params(msg, None)
}

fn internal(msg: String) -> ErrorData {
    ErrorData::internal_error(msg, None)
}

// ---------------------------------------------------------------------------
// MCP server
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct StartCrawlArgs {
    /// Project id to crawl.
    project_id: String,
    /// Crawl configuration (CrawlConfig JSON: seed_urls, url_list, max_depth, ...).
    config: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct ProjectIdArgs {
    project_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct GetResultsArgs {
    project_id: String,
    /// Page number, 1-based. Defaults to 1.
    #[serde(default = "default_page")]
    page: u32,
    /// Rows per page (max 500). Defaults to 50.
    #[serde(default = "default_page_size")]
    page_size: u32,
}

fn default_page() -> u32 {
    1
}

fn default_page_size() -> u32 {
    50
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct GetPageDetailArgs {
    project_id: String,
    /// URL of the page (most recent crawl wins).
    url: String,
}

#[derive(Clone)]
struct OpenCrawlerMcp {
    state: Arc<RwLock<AppState>>,
    tool_router: ToolRouter<Self>,
}

impl OpenCrawlerMcp {
    fn new(state: Arc<RwLock<AppState>>) -> Self {
        Self {
            state,
            tool_router: Self::tool_router(),
        }
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for OpenCrawlerMcp {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new(
                "open-crawler",
                env!("CARGO_PKG_VERSION"),
            ))
            .with_instructions(
                "OpenCrawler SEO audit tool. Use list_projects to see projects, \
                 start_crawl to audit a site (pass a CrawlConfig object), \
                 get_crawl_status to poll it, get_results for paginated results \
                 and get_page_detail for one URL.",
            )
    }
}

#[tool_router(router = tool_router)]
impl OpenCrawlerMcp {
    /// List all projects.
    #[tool(name = "list_projects", description = "List all crawl projects.")]
    async fn list_projects(&self) -> Result<String, ErrorData> {
        let projects: Vec<Project> = with_repo(&self.state, |repo| repo.list_projects())
            .await
            .map_err(|e| internal(e.to_string()))?;
        serde_json::to_string(&projects).map_err(|e| internal(e.to_string()))
    }

    /// Start a crawl for a project with the given config.
    #[tool(
        name = "start_crawl",
        description = "Start a crawl for a project. Pass a CrawlConfig object (seed_urls, url_list, max_depth, ...). Returns immediately; poll get_crawl_status."
    )]
    async fn start_crawl(&self, params: Parameters<StartCrawlArgs>) -> Result<String, ErrorData> {
        let args = params.0;
        let config: CrawlConfig =
            serde_json::from_value(args.config).map_err(|e| invalid_params(e.to_string()))?;
        if config.seed_urls.is_empty() && config.url_list.is_empty() {
            return Err(invalid_params(
                "config needs seed_urls or url_list".to_string(),
            ));
        }
        start_crawl_headless(self.state.clone(), config, &args.project_id, false)
            .await
            .map_err(|e| internal(e.to_string()))?;
        Ok(serde_json::json!({
            "started": true,
            "project_id": args.project_id,
        })
        .to_string())
    }

    /// Status of a running crawl (null when idle).
    #[tool(
        name = "get_crawl_status",
        description = "Status of a running crawl for a project, or null when idle."
    )]
    async fn get_crawl_status(
        &self,
        params: Parameters<ProjectIdArgs>,
    ) -> Result<String, ErrorData> {
        let args = params.0;
        let state = self.state.read().await;
        let crawls = state.crawls.read().await;
        let progress = crawls.get(&args.project_id).map(|s| s.progress.clone());
        serde_json::to_string(&progress).map_err(|e| internal(e.to_string()))
    }

    /// Paginated crawl results for a project.
    #[tool(
        name = "get_results",
        description = "Paginated crawl results (light rows with SEO score) for a project."
    )]
    async fn get_results(&self, params: Parameters<GetResultsArgs>) -> Result<String, ErrorData> {
        let args = params.0;
        let page = args.page.max(1);
        let page_size = args.page_size.clamp(1, 500);
        let pid = args.project_id.clone();
        let (items, total) = with_repo(&self.state, move |repo| {
            repo.get_results(
                &pid, page, page_size, None, None, None, None, None, None, false, false, false,
                false,
            )
        })
        .await
        .map_err(|e| internal(e.to_string()))?;
        serde_json::to_string(&serde_json::json!({
            "items": items,
            "total": total,
            "page": page,
            "page_size": page_size,
        }))
        .map_err(|e| internal(e.to_string()))
    }

    /// Full detail (SEO audit, issues, links) of one crawled URL.
    #[tool(
        name = "get_page_detail",
        description = "Full detail of one crawled URL (SEO audit, issues, links)."
    )]
    async fn get_page_detail(
        &self,
        params: Parameters<GetPageDetailArgs>,
    ) -> Result<String, ErrorData> {
        let args = params.0;
        let pid = args.project_id.clone();
        let url = args.url.clone();
        let detail = with_repo(&self.state, move |repo| {
            let id = repo
                .get_page_id_by_url(&pid, &url)?
                .ok_or_else(|| AppError::Crawl(format!("No crawled page for URL: {url}")))?;
            repo.get_page_detail(&id)
        })
        .await
        .map_err(|e| internal(e.to_string()))?;
        serde_json::to_string(&detail).map_err(|e| internal(e.to_string()))
    }
}

// ---------------------------------------------------------------------------
// CLI crawl mode (config file)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct CrawlFile {
    project_id: Option<String>,
    project_name: Option<String>,
    #[serde(default)]
    config: CrawlConfig,
}

async fn run_crawl_file(
    state: Arc<RwLock<AppState>>,
    config_path: &str,
    project_name: Option<String>,
) -> Result<serde_json::Value, AppError> {
    let text = std::fs::read_to_string(config_path)
        .map_err(|e| AppError::Crawl(format!("Cannot read {config_path}: {e}")))?;
    let file: CrawlFile = serde_json::from_str(&text)
        .map_err(|e| AppError::Crawl(format!("Invalid config file {config_path}: {e}")))?;
    if file.config.seed_urls.is_empty() && file.config.url_list.is_empty() {
        return Err(AppError::Crawl(
            "Config needs seed_urls or url_list".to_string(),
        ));
    }

    let project_id = match file.project_id {
        Some(id) => {
            let exists = with_repo(&state, |repo| {
                Ok(repo.list_projects()?.into_iter().any(|p| p.id == id))
            })
            .await?;
            if !exists {
                return Err(AppError::Crawl(format!("Unknown project_id: {id}")));
            }
            id
        }
        None => {
            let name = project_name
                .or(file.project_name)
                .or_else(|| {
                    file.config
                        .seed_urls
                        .first()
                        .or(file.config.url_list.first())
                        .and_then(|u| url::Url::parse(u).ok())
                        .and_then(|u| u.host_str().map(|h| h.to_string()))
                })
                .unwrap_or_else(|| "MCP crawl".to_string());
            let project = with_repo(&state, move |repo| repo.create_project(&name)).await?;
            project.id
        }
    };

    let (pages_crawled, elapsed_secs) =
        run_crawl_blocking(state, file.config, &project_id, false).await?;
    Ok(serde_json::json!({
        "project_id": project_id,
        "pages_crawled": pages_crawled,
        "elapsed_secs": elapsed_secs,
    }))
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn usage() -> &'static str {
    "open-crawler-mcp — MCP server and headless crawl runner\n\n\
     USAGE:\n  \
     open-crawler-mcp [serve] [--db PATH]   MCP server on stdio (default)\n  \
     open-crawler-mcp crawl --config FILE [--db PATH] [--project-name NAME]\n\n\
     The database defaults to the desktop app's \
     (<data_dir>/com.opencrawler.app/open-crawler.db), \
     or OPEN_CRAWLER_DB when set."
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "open_crawler_mcp=info,open_crawler_lib=info".into()),
        )
        .with_writer(std::io::stderr)
        .init();

    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut db_path: Option<PathBuf> = None;
    let mut mode = String::from("serve");
    let mut config_path: Option<String> = None;
    let mut project_name: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "serve" | "crawl" => mode = args[i].clone(),
            "--db" => {
                i += 1;
                db_path = args.get(i).map(PathBuf::from);
            }
            "--config" => {
                i += 1;
                config_path = args.get(i).cloned();
            }
            "--project-name" => {
                i += 1;
                project_name = args.get(i).cloned();
            }
            "--help" | "-h" => {
                eprintln!("{}", usage());
                return;
            }
            other => {
                eprintln!("Unknown argument: {other}\n\n{}", usage());
                std::process::exit(2);
            }
        }
        i += 1;
    }

    let db_path = resolve_db_path(db_path);
    if let Some(parent) = db_path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            error!("Cannot create DB directory: {e}");
            std::process::exit(1);
        }
    }
    let state = match open_app_state(&db_path) {
        Ok(state) => Arc::new(RwLock::new(state)),
        Err(e) => {
            error!("Cannot open database at {:?}: {e}", db_path);
            std::process::exit(1);
        }
    };

    if mode == "crawl" {
        let Some(config_path) = config_path else {
            eprintln!("crawl mode needs --config FILE\n\n{}", usage());
            std::process::exit(2);
        };
        match run_crawl_file(state, &config_path, project_name).await {
            Ok(summary) => {
                println!("{}", serde_json::to_string(&summary).unwrap());
            }
            Err(e) => {
                error!("Crawl failed: {e}");
                std::process::exit(1);
            }
        }
        return;
    }

    info!("OpenCrawler MCP server on stdio (db: {:?})", db_path);
    let service = OpenCrawlerMcp::new(state);
    match service.serve(rmcp::transport::stdio()).await {
        Ok(server) => {
            if let Err(e) = server.waiting().await {
                error!("MCP server error: {e}");
                std::process::exit(1);
            }
        }
        Err(e) => {
            error!("Cannot start MCP server: {e}");
            std::process::exit(1);
        }
    }
}
