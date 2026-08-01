# Open Crawler - Development Progress

## Project Overview
SEO audit tool similar to Screaming Frog, built with **Rust + Tauri v2 + Svelte 5 + TypeScript + pnpm**.

**Current Phase:** Sprints 1-3 complete (UI quick wins, performance/filters, advanced crawl config) + caninclude replacement
**Status:** Backend compiles, 0 clippy warnings, 30 tests pass, streaming results, resume capability, frontier, robots.txt, semantic HTML audit with static nesting matrix (no API), virtualized results table, filters (status/severity/depth), URL dedup, include/exclude glob patterns, custom headers + configurable timeout, theme system, i18n (en/es), Android CI workflow

---

## What's Done

### Phase 1: Foundation ✅ COMPLETED

| # | Task | Status |
|---|------|--------|
| 1.1 | Scaffold project with `create-tauri-app` (Svelte + TypeScript + pnpm) | ✅ |
| 1.2 | Directory structure (`commands/`, `crawler/`, `models/`, `db/`) | ✅ |
| 1.3 | Rust dependencies (`Cargo.toml`) | ✅ |
| 1.4 | Frontend dependencies (`package.json`) | ✅ |
| 1.5 | SQLite migration - tables crawl_config, crawled_pages, page_links, crawl_errors | ✅ |
| 1.6 | Model `CrawlConfig` | ✅ |
| 1.7 | Model `CrawlResult` | ✅ |
| 1.8 | Model `CrawlProgress` and `PaginatedResults` | ✅ |
| 1.9 | `AppError` enum with `thiserror` | ✅ |
| 1.10 | Trait `HtmlFetcher` with `HttpFetcher` | ✅ |
| 1.11 | `SeoParser` - SEO data extraction | ✅ |
| 1.12 | `CrawlEngine` - URL queue, crawl loop, DB save | ✅ |
| 1.13 | Tauri commands: `start_crawl`, `stop_crawl`, `get_results`, `export_csv` | ✅ |
| 1.14 | State management with `Arc<RwLock<AppState>>` | ✅ |
| 1.15 | SQLite migrations in `setup()` | ✅ |
| 1.16 | UI Svelte: config form, progress bar, results table | ✅ |
| 1.17 | Tauri events: `crawl-progress`, `crawl-complete`, `crawl-error` | ✅ |
| 1.18-1.20 | `cargo check`, `cargo clippy`, `pnpm build` | ✅ |

### Phase 1.5: Multi-Project Feature ✅ COMPLETED

| # | Task | Status |
|---|------|--------|
| 1.5.1-1.5.14 | Full CRUD, filtering by project_id, frontend sidebar | ✅ |

### Phase 2: Crawler Engine ✅ COMPLETED

| # | Task | Status |
|---|------|--------|
| 2.1 | `CrawlEngine` with `VecDeque` + dedup | ✅ |
| 2.2 | `Semaphore` for concurrency control | ✅ |
| 2.3 | `tokio::sync::mpsc` for parallel URL discovery | ✅ |
| 2.4 | `CancellationToken` for safe stop | ✅ |
| 2.5 | Multi-project state: `HashMap<String, CrawlState>` | ✅ |
| 2.6 | Events with `project_id` | ✅ |
| 2.7-2.12 | Bug fixes (Emitter, $effect, deadlock, etc.) | ✅ |
| 2.13 | LRU visited set (500K capacity) | ✅ |
| 2.14 | Bounded queue (max 100K URLs) | ✅ |
| 2.15 | Same-origin filtering | ✅ |
| 2.16 | `SitemapParser` - robots.txt + sitemap.xml + index + gzip | ✅ |
| 2.17-2.18 | `check_sitemap`, `max_crawl_time_secs` config | ✅ |
| 2.19-2.24 | Hreflang extraction, html_lang, frontend updates | ✅ |

### Phase 2.5: DbWriter + Streaming Results ✅ COMPLETED

| # | Task | Status |
|---|------|--------|
| 2.25 | `DbWriter` actor with mpsc channel | ✅ |
| 2.26 | Batch writes (50 pages per batch, 2s flush interval) | ✅ |
| 2.27 | `crawl-batch` event emitted on each flush | ✅ |
| 2.28 | Frontend: listen for `crawl-batch`, append results in real-time | ✅ |
| 2.29 | `CrawlResultMsg` enum (Page, Links, Error, Flush, Done) | ✅ |

### Phase 2.6: Persistent Crawl State + Resume ✅ COMPLETED

| # | Task | Status |
|---|------|--------|
| 2.30 | `crawl_sessions` table (id, project_id, config_json, status, progress) | ✅ |
| 2.31 | `crawl_queue` table (session_id, url, depth) for resume | ✅ |
| 2.32 | DB migration v4 for new tables | ✅ |
| 2.33 | `CrawlRepo` methods: create_session, update_session_progress, complete_session, interrupt_session | ✅ |
| 2.34 | `CrawlRepo` methods: get_interrupted_session, get_session_config | ✅ |
| 2.35 | `CrawlRepo` methods: save_queue_batch, load_queue, clear_queue | ✅ |
| 2.36 | `CrawlRepo` methods: get_visited_urls_for_project | ✅ |
| 2.37 | `check_resumable_crawl` command | ✅ |
| 2.38 | Engine: detect interrupted session on start, load queue + visited from DB | ✅ |
| 2.39 | Engine: periodic queue flush every 100 URLs | ✅ |
| 2.40 | Engine: mark session "completed" or "interrupted" on stop/crash | ✅ |
| 2.41 | Frontend: resume dialog ("Resume Previous Crawl?" with stats) | ✅ |
| 2.42 | Frontend: "Resume / Start" button when resumable | ✅ |

### Phase 2.7: Frontier (Priority Queue) ✅ COMPLETED

| # | Task | Status |
|---|------|--------|
| 2.43 | `Frontier` struct with domain rotation + depth priority | ✅ |
| 2.44 | Round-robin domain scheduling (fair crawl across domains) | ✅ |
| 2.45 | Max depth + max URLs limits | ✅ |
| 2.46 | `drain_all()` + `restore()` for persistence | ✅ |
| 2.47 | Unit tests (basic push/pop, domain rotation, max_depth, max_urls, drain/restore) | ✅ |

### Phase 2.8: RobotsChecker ✅ COMPLETED

| # | Task | Status |
|---|------|--------|
| 2.48 | `RobotsChecker` with robots.txt download + cache | ✅ |
| 2.49 | Parse disallow rules, crawl-delay, sitemaps | ✅ |
| 2.50 | Cache TTL (1 hour) per domain | ✅ |
| 2.51 | Integration: check before each fetch in CrawlEngine | ✅ |
| 2.52 | Unit tests (basic, specific agent, empty) | ✅ |

### Phase 3.0: Semantic HTML Audit ✅ COMPLETED

| # | Task | Status |
|---|------|--------|
| 3.0.1 | `SemanticIssue` struct (issue_code, severity, element, message) | ✅ |
| 3.0.2 | `analyze_semantics()` with 16 issue checks (missing main/header/footer/nav, img alt, input labels, ARIA, heading hierarchy, etc.) | ✅ |
| 3.0.3 | `check_semantics` config option (default true) | ✅ |
| 3.0.4 | `semantic_issues_json` column added to `crawled_pages` table | ✅ |
| 3.0.5 | `SeoParser::analyze_semantics()` integration in DbWriter | ✅ |
| 3.0.6 | Unit tests for semantic checks | ✅ |

### Phase 3.1: Pagination ✅ COMPLETED

| # | Task | Status |
|---|------|--------|
| 3.1.1 | Backend `get_results` with pagination params (page, page_size) | ✅ |
| 3.1.2 | Frontend page numbers (1, 2, 3...12) | ✅ |
| 3.1.3 | Prev/Next navigation | ✅ |
| 3.1.4 | Page size selector (25/50/100/200) | ✅ |
| 3.1.5 | "Showing X-Y of Z" counter | ✅ |

### Phase 3.2: Crawl-Batch Fix ✅ COMPLETED

| # | Task | Status |
|---|------|--------|
| 3.2.1 | Fixed crawl-batch to not mix streamed results with paginated results | ✅ |
| 3.2.2 | Streaming shows count only, refreshes on complete | ✅ |

### Phase 3.3: Issues Column + Detail View ✅ COMPLETED

| # | Task | Status |
|---|------|--------|
| 3.3.1 | Issues column with colored badges (red/yellow/blue) | ✅ |
| 3.3.2 | Expand/collapse detail view with full issue list | ✅ |
| 3.3.3 | `SemanticIssue` type exported from `crawler/mod.rs` | ✅ |

### Phase 3.4: Virtualized Results Table ✅ COMPLETED

| # | Task | Status |
|---|------|--------|
| 3.4.1 | `@tanstack/virtual-core` installed | ✅ |
| 3.4.2 | `ResultsTable.svelte` component with virtual scroll (600px viewport) | ✅ |
| 3.4.3 | Flattened rows: main row + optional expanded detail row | ✅ |
| 3.4.4 | Sticky header, grid columns, a11y (role=button, keyboard) | ✅ |
| 3.4.5 | Moved inline table styles to component scope | ✅ |

### Phase 3.5: HTML Nesting Validation ✅ COMPLETED (caninclude API replaced)

| # | Task | Status |
|---|------|--------|
| 3.5.1 | `check_element_nesting()` walks DOM, checks parent-child pairs | ✅ |
| 3.5.2 | `invalid_nesting` issue type (severity: error) for forbidden pairs | ✅ |
| 3.5.3 | `context_nesting` issue type (severity: info) for context-dependent pairs | ✅ |
| 3.5.4 | 6+ unit tests: div-in-span, p-in-a, table-in-code, valid combos, table coverage | ✅ |
| 3.5.5 | **Static `nesting_table.rs` (104×104 matrix) embedded — replaces `caninclude.onrender.com` API** | ✅ |
| 3.5.6 | API fallback + `NESTING_RULES` HashMap removed from parser | ✅ |
| 3.5.7 | `tools/generate_nesting_table.js` regenerates matrix from `src-tauri/data/` JSONs | ✅ |
| 3.5.8 | 4 matrix unit tests (`test_valid_combinations`, `test_invalid_combinations`, `test_unknown_tag_returns_none`, `test_matrix_dimensions`) | ✅ |

### Phase 4.0: Page Preview & Screenshot Infrastructure ✅ COMPLETED

| # | Task | Status |
|---|------|--------|
| 4.0.1 | `html_body TEXT` column added to crawled_pages (migration v5) | ✅ |
| 4.0.2 | First 100KB of HTML stored during crawl (`engine.rs` `fetch_and_parse`) | ✅ |
| 4.0.3 | `CrawlResult.html_body: Option<String>` field | ✅ |
| 4.0.4 | `get_page_html` command — returns stored HTML for a page | ✅ |
| 4.0.5 | `inline_assets` command — fetches external CSS/images, converts to data URIs | ✅ |
| 4.0.6 | `crawler/assets.rs` module — `inline_page_assets()`, `collect_asset_urls()`, `fetch_assets()`, `replace_urls()` | ✅ |
| 4.0.7 | `screenshot.rs` module — command-line Chrome headless screenshots | ✅ |
| 4.0.8 | `recrawl_page` command — single page re-crawl + screenshot | ✅ |
| 4.0.9 | `capture_page_screenshot` command — on-demand screenshot capture | ✅ |
| 4.0.10 | `screenshot_png BLOB` column added (migration v6) | ✅ |
| 4.0.11 | All DB queries updated: `save_result`, `save_results_batch`, `row_to_result`, `get_results`, `get_page_detail` | ✅ |

### Phase 4.1: UI Enhancements — Page Preview + Semantic Dashboard ✅ COMPLETED

| # | Task | Status |
|---|------|--------|
| 4.1.1 | `SemanticDashboard.svelte` — severity summary bar (error/warning/info proportionally) | ✅ |
| 4.1.2 | Emoji icons per issue type, grouped counts by severity | ✅ |
| 4.1.3 | Full-width list layout for issues | ✅ |
| 4.1.4 | `PageDetailPanel.svelte` — full-page overlay with header (back + URL + status + re-crawl) | ✅ |
| 4.1.5 | Tab system: Overview | Page Preview | Links | ✅ |
| 4.1.6 | Overview tab: 2-column responsive grid for page stats | ✅ |
| 4.1.7 | `HtmlTree.svelte` — iframe-based preview with CSS highlight injection | ✅ |
| 4.1.8 | Highlight CSS: outlines, badges, tooltips for elements with semantic issues | ✅ |

### Phase 4.2: Bug Fixes ✅ COMPLETED

| # | Task | Status |
|---|------|--------|
| 4.2.1 | `window.confirm()` → Tauri `confirm()` (dialog plugin) | ✅ |
| 4.2.2 | `delete_project()` rewritten with `unchecked_transaction()` + subquery deletes (FK fix) | ✅ |
| 4.2.3 | SQLite positional params fixed (`?N` requires all params up to N) | ✅ |

### Phase 4.3: Enhanced Semantic Issue Context ✅ COMPLETED

| # | Task | Status |
|---|------|--------|
| 4.3.1 | `SemanticIssue` struct enhanced: `xpath`, `css_selector`, `snippet`, `line`, `column` (all optional) | ✅ |
| 4.3.2 | `issue()` builder function for constructing issues with element context | ✅ |
| 4.3.3 | `compute_xpath()` — builds XPath from element tag, id, classes, position | ✅ |
| 4.3.4 | `compute_css_selector()` — builds CSS selector from id/classes | ✅ |
| 4.3.5 | `compute_snippet()` — extracts first 200 chars of element HTML | ✅ |
| 4.3.6 | `tempfile` crate added to Cargo.toml for screenshot temp files | ✅ |
| 4.3.7 | `chromiumoxide` removed from Cargo.toml (replaced by CLI Chrome approach) | ✅ |
| 4.3.8 | All ~20 `SemanticIssue` construction sites updated with `..Default::default()` | ✅ |

### Sprint 1: UI Quick Wins ✅ COMPLETED

| # | Task | Status |
|---|------|--------|
| S1.1 | Toast notifications component | ✅ |
| S1.2 | Server-side search in results table (400ms debounce) | ✅ |
| S1.3 | Export CSV/Excel with Severity column | ✅ |
| S1.4 | README documentation | ✅ |
| S1.5 | Light/dark/system theme support via `tokens.css` + `[data-theme]` + `matchMedia` | ✅ |
| S1.6 | SettingsModal theme selector (System/Light/Dark), persisted in localStorage | ✅ |

### Sprint 2: Performance & Filters ✅ COMPLETED

| # | Task | Status |
|---|------|--------|
| S2.1 | Removed hardcoded colors: `PageDetailPanel` → `.issue-severity-error/warning/info` CSS classes | ✅ |
| S2.2 | Backend filters: `status_filter`, `severity_filter`, `domain_filter`, `depth_filter` in `get_results` (dynamic SQL WHERE) | ✅ |
| S2.3 | ResultsTable virtualized with `Virtualizer` class from `@tanstack/virtual-core` | ✅ |
| S2.4 | `FilterBar.svelte` — status chips, severity chips, depth slider, clear-all (domain dropdown removed from UI) | ✅ |
| S2.5 | i18n filter keys (`filter.*`) in en/es | ✅ |
| S2.6 | Results cache type alias `ResultsCacheArc` + clippy cleanup | ✅ |

### Sprint 3: Advanced Crawl Config ✅ COMPLETED

| # | Task | Status |
|---|------|--------|
| S3.1 | `crawler/dedup.rs` — `Deduplicator` URL normalization (fragments, host case, default ports, trailing slash, query param sort) | ✅ |
| S3.2 | URL normalization integrated into `url_visited`/`mark_visited` in engine | ✅ |
| S3.3 | 8 dedup unit tests | ✅ |
| S3.4 | Include/exclude URL patterns via `glob = "0.3"` (`CrawlConfig.include_patterns`/`exclude_patterns`) | ✅ |
| S3.5 | Custom headers (`CrawlConfig.custom_headers: Vec<(String,String)>`) applied per request | ✅ |
| S3.6 | Configurable timeout (`CrawlConfig.request_timeout_ms`, default 30,000) via `HttpFetcher::new(user_agent, timeout_ms, custom_headers)` | ✅ |

### CI/CD: Android Workflow ✅ COMPLETED

| # | Task | Status |
|---|------|--------|
| CI.1 | GitHub Actions `android.yml` — Android APK build | ✅ |
| CI.2 | NDK toolchain setup + `aarch64-linux-android` target | ✅ |
| CI.3 | reqwest → rustls (no OpenSSL on Android) | ✅ |
| CI.4 | macOS Intel (x86_64) + Apple Silicon builds via cross-compilation | ✅ |

---

## What's Next

### Phase 4: UI de Results (continued)

| # | Task | Status |
|---|------|--------|
| 4.4 | Rewrite `HtmlTree.svelte` to use screenshot-based preview (img + overlay markers) | ⬜ |
| 4.5 | Wire enhanced fields into `analyze_semantics()` — use `issue()` builder with element refs | ⬜ |
| 4.6 | `SiteTree.svelte` - tree view | ⬜ |
| 4.8 | Full CSV export with all SEO fields | ⬜ |
| 4.10 | Filters: missing title, duplicate, 404, noindex | ⬜ |

### Phase 5: Full Functionality

| # | Task | Status |
|---|------|--------|
| 5.3 | Config: `proxy` support | ⬜ |
| 5.4 | Excel export - multiple sheets (All Pages, Issues, Links) | ⬜ |

### Phase 6: JS Rendering

| # | Task | Status |
|---|------|--------|
| 6.1 | Integrate `chromiumoxide` for JS rendering | ⬜ |
| 6.2 | Browser pool - reuse tabs per domain | ⬜ |
| 6.3 | Benchmarking with `criterion` crate | ⬜ |
| 6.4 | Stress test: >10k pages in <5 minutes | ⬜ |
| 6.5 | Optimization: connection pool, batch writes | ⬜ |

### Phase 7: Polish & Packaging

| # | Task | Status |
|---|------|--------|
| 7.3 | Auto-updater with `tauri-plugin-updater` | ⬜ |
| 7.5 | E2E tests with Playwright | ⬜ |
| 7.6 | Documentation (README, CONTRIBUTING, cargo doc) | ⬜ |

---

## Files Modified/Created

### Backend (src-tauri/)

```
src-tauri/src/
├── main.rs                        # Entry point
├── lib.rs                         # AppState + ResultsCacheArc, command registrations, pub mod nesting_table
├── error.rs                       # AppError enum with Http variant
├── nesting_table.rs               # Static 104×104 nesting matrix (auto-generated)
├── commands/mod.rs                # Project CRUD + crawl + get_results (with filters) + recrawl_page + screenshots
├── crawler/
│   ├── mod.rs                     # Exports: CrawlEngine, SemanticIssue, assets, screenshot
│   ├── engine.rs                  # CrawlEngine: URL dedup, include/exclude patterns, resume, html_body storage
│   ├── fetcher.rs                 # HtmlFetcher trait + HttpFetcher (headers + timeout)
│   ├── parser.rs                  # SeoParser (17 semantic checks + nesting via can_include), issue() builder
│   ├── dedup.rs                   # Deduplicator - URL normalization + dedup (8 tests)
│   ├── sitemap.rs                 # SitemapParser - sitemap discovery
│   ├── frontier.rs                # Frontier - priority queue with domain rotation
│   ├── robots.rs                  # RobotsChecker - robots.txt cache + crawl-delay
│   ├── assets.rs                  # inline_page_assets, collect_asset_urls, fetch_assets, replace_urls
│   ├── screenshot.rs              # capture_screenshot via CLI Chrome, find_chrome
│   └── db_writer.rs              # DbWriter actor - batch writes + streaming events
├── models/
│   ├── mod.rs
│   ├── crawl_config.rs            # CrawlConfig: project_id, IMPLICIT_USER_AGENT, patterns, headers, timeout
│   ├── crawl_result.rs            # CrawlResult (with html_body), CrawlProgress, PageLink, SemanticIssue
│   └── project.rs                 # Project, CreateProjectRequest, RenameProjectRequest
├── data/                          # caninclude JSONs + generate.js (reference only for regeneration)
└── db/
    ├── mod.rs
    ├── schema.rs                  # Migrations: v4 (crawl_sessions, queue), v5 (html_body), v6 (screenshot_png)
    └── crawl_repo.rs              # CrawlRepo: filters, cache, sessions, screenshots, html_body
```

### Frontend (src/)

```
src/
├── lib/
│   ├── tokens.css                 # Light/dark/system themes, semantic CSS variables
│   ├── i18n-issues.ts             # Issue name/message translation + param parsing (incl. context_nesting)
│   └── components/
│       ├── ResultsTable.svelte        # Virtualized table (Virtualizer class)
│       ├── FilterBar.svelte           # Filters: status chips, severity chips, depth slider
│       ├── PageDetailPanel.svelte     # Full-page overlay: tabs (Overview | Page Preview | Links)
│       ├── SemanticDashboard.svelte   # Severity summary bar
│       ├── SettingsModal.svelte       # Theme selector (System/Light/Dark)
│       ├── Toast.svelte               # Toast notifications
│       └── HtmlTree.svelte            # iframe preview with CSS highlight injection
├── routes/
│   ├── +layout.svelte                 # Root layout + theme application
│   ├── +layout.ts                     # Static adapter config (ssr = false)
│   └── +page.svelte                   # Main UI: sidebar, config, resume, filters, pagination, detailPageId
```

### Tooling

```
tools/generate_nesting_table.js   # Regenerates nesting_table.rs from src-tauri/data/ JSONs (node tools/generate_nesting_table.js)
.github/workflows/android.yml     # Android APK build + NDK toolchain
```

---

## State Management Architecture

```
AppState
├── db: Mutex<Connection>              # SQLite connection
├── results_cache: ResultsCacheArc     # LRU results cache (Mutex<LruCache<ResultsCacheKey, ...>>)
└── crawls: Arc<RwLock<HashMap<String, CrawlState>>>
    └── CrawlState
        ├── cancellation: CancellationToken
        └── progress: CrawlProgress

CrawlEngine
├── frontier: Frontier                  # Domain-rotating priority queue
├── visited: LruCache<String, ()>       # 500K capacity dedup (normalized URLs)
├── robots: RobotsChecker               # robots.txt cache per domain
└── db_writer: DbWriter (via mpsc)      # Batch writes + streaming events

Persistent State (DB)
├── crawl_sessions: id, project_id, config_json, status, progress
├── crawl_queue: session_id, url, depth  # For resume after crash
├── crawled_pages: + html_body TEXT, screenshot_png BLOB, semantic_issues_json
└── page_links                           # Results (survive crashes)
```

**Key Design Decisions:**
- `DbWriter` actor receives results via mpsc channel, writes in batches (50 pages, 2s interval)
- `Frontier` does round-robin across domains for fair crawling
- `RobotsChecker` caches robots.txt per domain with 1h TTL
- Crawl sessions persisted to DB; on crash, detected as "interrupted" and resumable
- Queue flushed to DB every 100 URLs for crash recovery
- `crawl-batch` event streams results to frontend in real-time
- `html_body` stored (first 100KB) for page preview and re-analysis
- Asset inlining: CSS/images converted to data URIs for self-contained page preview
- Screenshots via CLI Chrome headless (`--headless --screenshot`)
- `SemanticIssue` carries optional element context (xpath, css_selector, snippet) for precise issue location
- Nesting validation uses an embedded static matrix (`nesting_table.rs`) — zero network dependency, O(1) lookup
- URL dedup normalizes before visiting (fragments, case, default ports, trailing slash, sorted query params)
- Results table is virtualized — renders only visible rows regardless of result count

---

## Verification Status

```bash
cargo check:     ✅ Compiles successfully
cargo clippy:    ✅ No warnings (clippy -- -D warnings)
cargo test:      ✅ 30/30 tests pass (parser 9, dedup 8, nesting_table 4, frontier 6, robots 3)
pnpm build:      ✅ Frontend builds to /build
```

---

## Notes

- Using `rusqlite` directly for better control over DB operations
- `CrawlEngine` runs in background tokio task via `tokio::spawn`
- State shared via `Arc<RwLock<AppState>>` for thread-safe access
- Frontend communicates via Tauri IPC (`invoke` for commands, `listen` for events)
- `Emitter` trait must be imported in Tauri v2: `use tauri::Emitter;`
- `$effect` is the Svelte 5 idiom for side effects
- All crawl events include `project_id` for frontend filtering
- `CrawlResultMsg::Page` uses `Box<CrawlResult>` to reduce enum size
- DB location: `~/Library/Application Support/com.opencrawler.app/open-crawler.db`
- `chromiumoxide` removed — screenshots use CLI Chrome instead (simpler, no Chromium download)
- `tempfile` crate used for screenshot temp files
- Asset inlining fetches in batches of 6, 10s timeout, 512KB max per resource
- `@tanstack/virtual-core` exports the `Virtualizer` class (not `createVirtualizer`)
- caninclude data JSONs (in `src-tauri/data/`) are reference-only; regenerate the matrix with `node tools/generate_nesting_table.js`
- Nesting severity mapping: `Cant` → `invalid_nesting` (error), `Doubt` → `context_nesting` (info)
- `Box<dyn ToSql>` cannot be cloned — dynamic filter params built as separate count/query `Vec`s
- `HttpFetcher::new(user_agent: &str, timeout_ms: u64, custom_headers: Vec<(String, String)>)` — all call sites pass config values
- Build requires Node 22 (`.nvmrc`): `source ~/.nvm/nvm.sh && nvm use 22 && pnpm build`

---

*Last updated: Sprints 1-3 + caninclude replacement completed, Android CI workflow added, 30 tests, i18n for context_nesting*
