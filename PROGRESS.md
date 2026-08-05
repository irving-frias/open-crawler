# Open Crawler - Development Progress

## Project Overview
SEO audit tool similar to Screaming Frog, built with **Rust + Tauri v2 + Svelte 5 + TypeScript + Bun**.

**Current Phase:** Sprints 1-3 complete + caninclude replacement + **Sprint 4 (SEO features F0-F8) complete** + **Sprint 5 (performance: page_issues, spawn_blocking, WAL)** + **Sprint 6 (OR filters, radar icon, README, cookies, site Basic Auth) complete** — now building **data sharing between devices (WiFi ✅ + Bluetooth/Nearby ✅ + P2P over internet)**
**Status:** Backend compiles, 0 clippy warnings, 77 tests pass, streaming results, resume capability, frontier, robots.txt, semantic HTML audit with static nesting matrix (no API), virtualized results table, filters (status/severity/depth + OR across categories), URL dedup, include/exclude glob patterns, custom headers + configurable timeout + cookies + site Basic Auth, proxy (URL+auth), theme system, i18n (en/es), Android CI workflow, dashboard overview, crawl comparison, site tree with issue badges, PageSpeed audits, readability scores, duplicate detection (simhash), keyword aggregation, Open Graph/Twitter view, visual style system (Classic/Neumorph/Clay/Glass/Brutalism), XLSX/CSV export with native save dialog + Android SAF + native share sheet, `.ocproj` package export/import (P1 done) — import re-keyed globally to fix cross-project config `FOREIGN KEY constraint failed`, WiFi LAN transfer server + QR (P2 done), **Bluetooth/Nearby sharing: receive-as-share-target on Android + native share sheet on desktop macOS (AirDrop) (P3 done)**

---

## What's Done

### Phase 1: Foundation ✅ COMPLETED

| # | Task | Status |
|---|------|--------|
| 1.1 | Scaffold project with `create-tauri-app` (Svelte + TypeScript + Bun) | ✅ |
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
| 1.18-1.20 | `cargo check`, `cargo clippy`, `bun run build` | ✅ |

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
| CI.1 | GitHub Actions `android.yml` — Android APK/AAB build (`tauri android build -t aarch64`) | ✅ |
| CI.2 | NDK r28 toolchain + `aarch64-linux-android24-clang` for CC/CXX/linker (ring/rusqlite) | ✅ |
| CI.3 | reqwest → rustls (no OpenSSL on Android) | ✅ |
| CI.4 | `src-tauri/gen/android` project committed (`tauri android init`) | ✅ |
| CI.5 | Release signing via `keystore.properties` + `signingConfigs` in `build.gradle.kts` | ✅ |
| CI.6 | Artifacts: `gen/android/app/build/outputs/{apk/universal/release,aab/universalRelease}` | ✅ |
| CI.7 | Verified signed APK (`apksigner verify`, CN=Open Crawler) | ✅ |

### Sprint 4: Advanced SEO Features (F0-F8) ✅ COMPLETED

**Phase 0: Foundation (migrations + models + parser)**

| # | Task | Status |
|---|------|--------|
| F0.1 | SQL migration runner (`db/migrations/mod.rs`) — `.sql` files via `include_str!`, `schema_migrations` table, transactional, idempotent; invoked from `schema.rs::run_migrations` | ✅ |
| F0.2 | `simhash = "0.3"` for content fingerprinting | ✅ |
| F0.3 | `AppError::Pagespeed(String)` variant | ✅ |
| F0.4 | Model fields: `content_hash`, `keywords_json`, `word_count`, `readability_score`, `og_title`, `og_description`, `og_image`, `og_image_alt`, `og_type`, `og_url`, `og_site_name`, `twitter_card`, `twitter_title`, `twitter_description`, `twitter_image` | ✅ |
| F0.5 | Parser: `extract_visible_text`, `compute_readability` (Flesch, clamped 0-100), `compute_content_hash` (simhash hex), `extract_keywords` (top 20, EN+ES stopwords), `extract_og_meta` | ✅ |
| F0.6 | Persistence in `save_result`/`save_results_batch`/`get_results`/`get_page_detail`/`row_to_result_export` + test helper `page()` | ✅ |

**F1: Dashboard Overview**

| # | Task | Status |
|---|------|--------|
| F1.1 | `DashboardStats`/`StatusBucket` models | ✅ |
| F1.2 | `get_dashboard_stats(project_id)` repo + registered command | ✅ |
| F1.3 | `Dashboard.svelte` + `charts/DonutChart.svelte` + `charts/BarChart.svelte` (pure SVG) | ✅ |
| F1.4 | "Overview" tab in `+page.svelte` | ✅ |

**F2: Crawl Comparison**

| # | Task | Status |
|---|------|--------|
| F2.1 | Models `CrawlSnapshot`, `SnapshotStats`, `UrlFieldDiff`, `ChangedUrl`, `CompareResult` | ✅ |
| F2.2 | Repo: `create_crawl_snapshot` (transactional full-dump to `crawl_snapshot_data`), `list_crawl_snapshots`, `snapshot_stats`, `snapshot_rows`, `compare_crawl_snapshots` (7 diff fields) | ✅ |
| F2.3 | Engine hook after `crawl-complete` — non-blocking `tokio::spawn` snapshot | ✅ |
| F2.4 | Commands `list_crawl_snapshots`, `compare_crawls` + `Comparador.svelte` (A/B select, new/removed/changed/unchanged cards) | ✅ |

**F3: Site Tree**

| # | Task | Status |
|---|------|--------|
| F3.1 | `SiteTreeNode.issue_count` + SQL `json_array_length(semantic_issues_json)` in root/children queries | ✅ |
| F3.2 | `SiteTree.svelte` with issue-count badges (destructive variant) + All/With issues/Without issues filter | ✅ |

**F4: PageSpeed Insights**

| # | Task | Status |
|---|------|--------|
| F4.1 | `pagespeed.rs` module — Google PSI v5 client (`runPagespeed`, strategy=desktop, category=performance, 90s timeout) → `PageSpeedData { score, fcp, lcp, cls, tbt, speed_index, error }` | ✅ |
| F4.2 | Repo `find_page_id`/`get_pagespeed`/`update_pagespeed` — cache persisted in `pagespeed_json` column | ✅ |
| F4.3 | `get_pagespeed_score` command (cache-first, API fallback) | ✅ |
| F4.4 | PageDetailPanel: circular score ring (≥90 success / ≥50 warning / <50 danger) + metrics + Run/Re-run; `pagespeed_api_key` setting in SettingsModal | ✅ |

**F5: Readability**

| # | Task | Status |
|---|------|--------|
| F5.1 | ResultsTable badge (BookOpenText icon) + tooltip with label/variant per score | ✅ |
| F5.2 | PageDetailPanel Progress bar + value/label | ✅ |

**F6: Duplicate Content**

| # | Task | Status |
|---|------|--------|
| F6.1 | `compute_duplicate_groups` — loads content hashes, union-find on `simhash::hamming_distance <= 10`, gid only for groups ≥2 members | ✅ |
| F6.2 | `get_duplicate_groups` (grouped, sorted size DESC) + engine hook on crawl complete + registered command | ✅ |
| F6.3 | `Duplicates.svelte` (groups, copy-URL button with execCommand fallback) + "Duplicates" tab | ✅ |

**F7: Keyword Aggregation**

| # | Task | Status |
|---|------|--------|
| F7.1 | `KeywordAggregate { keyword, count, pages }` — aggregates `keywords_json` across pages | ✅ |
| F7.2 | `get_project_keywords` command + `Keywords.svelte` (proportional bars, normalized) + "Keywords" tab | ✅ |

**F8: Open Graph / Twitter Cards**

| # | Task | Status |
|---|------|--------|
| F8.1 | PageDetailPanel "Social & Open Graph" accordion — `parseOg` maps og_* + twitter_* fields, preview image, linkable URLs | ✅ |

**F9: Final Verification**

| # | Task | Status |
|---|------|--------|
| F9.1 | `cargo check` clean, `cargo test --lib` → 53 passed / 3 ignored, `cargo clippy -- -D warnings` clean, `bun run build` + `bun run check` 0 errors | ✅ |
| F9.2 | 13 commands registered in `lib.rs` invoke_handler | ✅ |

### Sprint 5: Performance (page_issues + DB off async runtime + WAL) ✅ COMPLETED

| # | Task | Status |
|---|------|--------|
| S5.x | `page_issues` normalized table (migration 006), all DB work on `spawn_blocking` via `with_repo`, WAL + `synchronous=NORMAL` + 64MB cache + mmap + `temp_store=MEMORY`, single-round-trip site tree, windowed results, export only errors | ✅ |

### Sprint 6: OR Filters + Icon + Cookies + Site Auth ✅ COMPLETED

| # | Task | Status |
|---|------|--------|
| S6.1 | Result filters rewritten with union (OR) semantics across categories: `and_clauses` (missing_title/duplicate_title/noindex/404) + `filter_clauses` (status/severity/domain/depth/issue) joined with OR; `test_filter_combined`, `test_filter_or_across_categories`, `test_filter_or_still_scoped_to_project` — 64 tests pass, clippy clean, svelte-check 0 errors | ✅ |
| S6.2 | Radar app icon (`src-tauri/icons/icon-source.svg` → `bunx tauri icon`, all platforms incl. Android/iOS) | ✅ |
| S6.3 | README refresh for current features/architecture | ✅ |
| S6.4 | **Cookies** in advanced options: `CrawlConfig.cookies: Vec<String>`; `cookie_header_value()` helper (`crawler/mod.rs`); Cookie header in `HttpFetcher`, `RobotsChecker`, `SitemapParser`; engine wires `config.cookies`; `recrawl_page` reuses project cookies; frontend textarea (one cookie/line) in `CrawlControls.svelte`, i18n `config.cookies*` | ✅ |
| S6.5 | **Site Basic Auth**: `SiteAuth { username, password }` model, `CrawlConfig.site_auth`, `apply_basic_auth()` helper (`crawler/mod.rs`), `Authorization: Basic` on fetcher/robots/sitemap, `recrawl_page` reuses site_auth; frontend "Site username/password" fields, i18n `config.site_*` | ✅ |
| S6.6 | Committed: `bc19c1f` (OR filters), `3b3d0c8` (icon), `1ac1bf8` (README), `a149bc7` (cookies + Basic Auth) — all pushed to `main` | ✅ |

---

## CURRENT TASK: Data Sharing between devices (WiFi + Bluetooth + P2P) — IN PROGRESS

**Goal:** Export & share the app's data (projects + results + configs) between devices via an **importable `.ocproj` package**, transferable over **WiFi (LAN HTTP server + QR)**, **Bluetooth/Nearby (Android share sheet + receive-as-share-target)** and **P2P over the internet (WebRTC via PeerJS, libp2p fallback) when devices are on different networks**. All device pairs (Android↔Android, Desktop↔Android, Desktop↔Desktop). Receiver MUST be able to import. **Direct share**: choosing WiFi/Bluetooth/P2P exports the package automatically (no separate export step); Export/Import tabs stay as standalone features.

**User decisions locked:**
- Data = app-data package (importable), NOT just reports.
- Channels = all of them (WiFi LAN+QR AND Bluetooth/Nearby AND P2P when on different networks).
- Receiver = must import to continue analysis.
- Package = `{ importable }`; channels = `{ ambos }` (WiFi LAN+QR y Bluetooth/Nearby); receptor = `{ importar }`; pares = `{ todas las combinaciones }`; conflict modes `skip` (default) + `copy` (+ `overwrite`).
- P2P plan approved: **Option A = WebRTC DataChannels via PeerJS** (primary, `0.peerjs.com` cloud signaling, no infra, TURN fallback; Android Chromium WebView ✅, macOS WKWebView ✅, Windows WebView2 ✅; Linux WebKitGTK ❌ WebRTC) → **Option B = libp2p in Rust** (fallback Linux/native). Browser receiver `static/receive.html`. QR scan via `jsQR` + camera permission; manual code entry always available as fallback.

### Design

**Package format `.ocproj`** = ZIP (`zip 8.6.0`) with:
- `manifest.json`: `{ format: 1, app_version, schema_version, exported_at, projects: [{id,name,page_count,size_bytes,has_html,has_screenshots}], sha256, include_credentials }`
- `open-crawler.db`: clean copy via `VACUUM INTO` (works with WAL). Zip compresses `html_body` (TEXT, first 100KB, gzip+base64) + `screenshot_png` (BLOB gzip) well.

Export options: project subset (default: selected), **lightweight** (exclude html_body+screenshot_png — for Bluetooth), **include credentials** (`cookies`/`site_auth`/`proxy` in config_json + crawl_config columns are SECRETS — scrub by default).

**New backend module `src-tauri/src/transfer/`:**
- `package.rs` (P1 DONE): `export_package` (VACUUM INTO → prune unselected projects in reverse-FK order → scrub creds / lightweight → zip + manifest sha256), `import_package` (validate format<=1 + sha256 → tempdir → `run_migrations` → copy all 10 tables in FK order with full re-key `HashMap<old_id,new_id>` for project/config/session/page/snapshot in `unchecked_transaction`; `skip` (default) | `copy` | `overwrite` by project name; legacy `default` placeholder skipped; per-project errors → `summary.warnings`; no import cancel in UI).
- `server.rs` (P2 backend DONE): `start_transfer_server(path) -> TransferInfo {urls, port, token, expires_in_secs, file_name, file_size_bytes}` — bind `0.0.0.0` port 45231 (ephemeral fallback) with `tiny_http`. Routes: `GET /` landing HTML, `GET /dl/<token>/<name>` streams file (validates token, `percent_encode`), `GET /health`. Auto-expiry 15 min (configurable TTL) + `stop_transfer_server()`. LAN IPs via `local-ip-address`. Server stored in `AppState.transfer_server: Mutex<Option<TransferServerState>>` (`{path, token, port, urls, expires_at, stop: Arc<AtomicBool>, _server: Arc<Server>}`). `lib.rs` requires `tauri::Manager` for `app.path()`.
- `commands.rs` (P2 backend DONE): `export_package`, `import_package` (mobile content:// via `copy_from_content_uri`), `start_transfer_server`, `stop_transfer_server`, `get_active_transfer` (reconstructs urls/port from saved state), `download_transfer` (reqwest `stream` feature, events `transfer-progress`, requires `futures::StreamExt` + `tokio::io::AsyncWriteExt`), `share_package` (mobile-only via `tauri_plugin_share`), `package_target`. All registered in `lib.rs` invoke_handler.
- `p2p.rs` (P5, plan approved, NOT started): libp2p = `0.56` (features `tcp, noise, yamux, relay, dcutr, mdns, kad, identify, tokio`), `Relay(client)` + `Identify` + `Dcutr` + `Mdns` + `Kademlia` + `StreamProtocol` streaming; rendezvous by relay multiaddr in QR or DHT `hash(code)`; DCUtR hole-punch after relay. Feature-flag in `Cargo.toml` (proposed default-on).

**Deep-link (QR/links):** `ocp2p:<format>:<code>` with format `wr` (WebRTC room) or `lp` (libp2p multiaddr). Emisor = callee, receptor = caller. DataChannel messages: `handshake {file_name,size,sha256}` → binary chunks 16–64 KB with backpressure via `bufferedAmount` → `done`/`error`. Receiver writes to `<appData>/transfers/<name>` then calls `import_package(dest, mode)` (receiver picks conflict mode). Progress via `transfer-progress` event.

**Frontend (P2 written, compiles):** `src/lib/features/transfer/TransferDialog.svelte` (tabs Export / WiFi / Import + QR via npm `qrcode`, expiry countdown, copy URLs, TTL select, conflict-mode select, download-with-progress bar `{stage, processed, total, percent}`), `src/lib/api/transfer.ts` wrappers, `app.svelte.ts` state/actions (`exportPackage` save-dialog `.ocproj` / mobile share; `importPackage`; `startTransferServer`/`stopTransferServer`; `downloadTransfer` + `listen('transfer-progress')`), Share button in `CrawlControls.svelte` (visible when `hasResults`), i18n `transfer.*` keys (334 keys en/es), paraglide compiled. **PENDING**: direct-share (WiFi/Bluetooth/P2P buttons call `exportPackage()` internally).

**Bluetooth/Nearby sharing (Phase P3, DONE):**
- **Receive on mobile (share target):** `tauri-plugin-mobile-sharetarget = "2"` in `[target.'cfg(any(target_os = "android", target_os = "ios"))'.dependencies]`; SEND intent-filter (mimes `application/octet-stream` + `application/x-opencrawler-package`) in `AndroidManifest.xml`; `tauri_app_lib_name=open_crawler_lib` in `gradle.properties`; capability `capabilities/mobile.json` (perm `mobile-sharetarget:default`, platforms `["android", "iOS"]` — `iOS` capitalised, `ios` fails the build). Command `import_shared_intent` (móvil): `STREAM` content:// → `copy_from_content_uri` to managed dir → `import_package`; `TEXT` → parse JSON payload. Intent values are percent-encoded → decoded with `percent-encoding` crate (not `url::form_urlencoded`). Frontend `checkIncomingShare()` runs once per session (`shareChecked` flag) on launch + `tauri://focus` → `importSharedIntent('skip')` (drains queue, skips if busy). Gate: mobile-only code behind `#[cfg(any(mobile, test))]` so desktop tests compile.
- **Share/export (send):** mobile button delegates to `exportPackage(includeCredentials, lightweight, true)` → silent export + `tauri_plugin_share::share_file` (native Android share sheet → Bluetooth/Nearby, Files, etc.). ⚠️ **Bug found & fixed:** `exportAndShare` used to set `transferBusy = true` BEFORE calling `exportPackage`, whose guard `if (… || transferBusy) return null` aborted instantly → button "did nothing". Fix: resolve `isMobile()` first and delegate without touching the busy flag.
- **Share on desktop (macOS AirDrop):** `tauri-plugin-share` is a no-op on desktop and `tauri-plugin-vnidrop-share` is base64-only (unsuitable for large `.ocproj`) → native module `transfer/desktop_share.rs` using `objc2` + `NSSharingServicePicker` (AirDrop/share sheet). Deps (macOS target only): `objc2`, `objc2-app-kit`, `objc2-foundation`, `objc2-core-foundation` (feature `CFCGTypes`), `raw-window-handle = "0.6.2"`. `NSSharingServicePicker::alloc()` needs the `AnyThread` trait in scope (not `ClassType`). Delegate retained in `ACTIVE_DELEGATES` thread_local; mpsc channels for setup/completion; `SHARE_COMPLETION_TIMEOUT = 300s`; `get_webview_window("main")`. `open_share_sheet` command registered; `share_package` desktop branch calls it (non-macOS returns a clear error → hint to WiFi/Export). **Explicit non-goal kept:** no OBEX/serial Bluetooth; this is the native share sheet.

**P2P deps (P4/P5):** JS `peerjs@1.5.5`, `jsQR`, `@types/qrcode`; Rust `libp2p = "0.56"`. P4 camera perms: `android.permission.CAMERA` in AndroidManifest, `NSCameraUsageDescription` in macOS Info.plist.

**Explicit non-goals:** real desktop Bluetooth (OBEX) — covered by WiFi + native macOS share sheet (AirDrop); iOS receive (needs Share Extension) — future. Verify `usesCleartextTraffic` (`${usesCleartextTraffic}` in manifest) is true for HTTP LAN on Android.

### Phases
- **P1 Package+Import** — DONE ✅ (package.rs, commands.rs, 8 tests). Export/Import UI done in P2 frontend.
- **P2 WiFi** — backend DONE ✅ (server.rs + 1 test `test_transfer_server_serves_file_and_health`, 6 commands registered, import handles `content://`); frontend written & compiles (TransferDialog + QR + download progress). **Remaining**: direct-share wiring (WiFi button exports internally).
- **P3 Bluetooth/Nearby** — NOT started (share-target receive + manifest intent-filter + Android share).
- **P4 WebRTC P2P (PeerJS)** — NOT started (PeerJS + jsQR camera scan + `static/receive.html` + deep-link `ocp2p:wr`).
- **P5 libp2p (Linux/native)** — NOT started (`transfer/p2p.rs` + feature flag + deep-link `ocp2p:lp`).
- **P6 Polish** — errors, a11y, README section, mime/asset.

### Current state
- **P1 (Package + Import) — DONE.** `src-tauri/src/transfer/` created: `mod.rs`, `package.rs`, `commands.rs`. Registered in `lib.rs` (`pub mod transfer;` + export/import in invoke_handler).
  - `package.rs`: `export_package` (VACUUM INTO → scrub secrets → optional lightweight → manifest sha256 → ZIP via `zip` 8.6), `import_package` (validate format+sha256 → extract DB → `run_migrations` → copy all 10 tables with full re-key of project/config/session/page/snapshot ids; `skip`/`copy`/`overwrite` modes by project name; skips legacy `default` placeholder).
  - `commands.rs`: mirrors `export_target` (desktop path / mobile SAF content:// or share-sheet fallback). `copy_to_content_uri` made `pub(crate)`; `copy_from_content_uri` added for mobile import.
  - Real bugs fixed while writing tests: `crawl_queue` has NO `status` column; `html_body` is TEXT (gzip+base64) not BLOB; `page_issues.position` is INTEGER; page/session/snapshot ids must be re-keyed for `copy` mode (FK + UNIQUE collisions).
  - 8 new tests in `package.rs` (round-trip row counts per table, skip/copy/overwrite conflicts, project filter, lightweight, credential scrub, corrupt checksum). `cargo test --lib` → 72 passed.
- **P2 (WiFi LAN) — backend DONE.** `server.rs` (tiny_http 0.12, bind 0.0.0.0:45231 + ephemeral fallback, TTL 15min, `/`, `/dl/<token>/<name>`, `/health`, `TransferServerState`, `TransferInfo`). Commands `start_transfer_server`/`stop_transfer_server`/`get_active_transfer`/`download_transfer` (with `TransferProgress` + `transfer-progress` event) registered. `import_package` handles `content://` on mobile. `reqwest` + feature `stream`. **Test fixed** (RwLock tokio vs std + Content-Length parsing in `http_get` instead of read_to_string) → **73 tests pass**, clippy `-D warnings` clean.
- **Direct-share (post-P2) — done.** `export_package` gained `silent` (writes to managed `<app_data>/transfers`, no save dialog / share sheet); `app.exportAndStartWifi()` = silent export + start server in one step; WiFi tab Start button now exports automatically (no prior Export step needed). **Persistence**: `lastPackage` + `activeTransfer` saved to localStorage; on reopen `lastPackage` is restored and, if a transfer was active when the app closed, the server auto-restarts (fresh token/TTL).
- **Frontend (TransferDialog) — written and compiles:** `src/lib/api/transfer.ts` + export; `app.svelte.ts` state/actions; `TransferDialog.svelte` tabs Export/WiFi/Import with QR (`qrcode`), expiry countdown, copy URLs, TTL + conflict-mode selects, download-progress bar; Share button in `CrawlControls.svelte`; i18n `transfer.*` (334 keys en/es), paraglide compiled. `bun run check` = 0 errors, `bun run build` = ok.
- Prior feature (cookies+BasicAuth) fully merged as `a149bc7`.

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
├── lib.rs                         # AppState + ResultsCacheArc, command registrations (13), pub mod nesting_table, pagespeed
├── error.rs                       # AppError enum with Http + Pagespeed variants
├── pagespeed.rs                   # Google PSI v5 client → PageSpeedData (score/fcp/lcp/cls/tbt/speed_index)
├── nesting_table.rs               # Static 104×104 nesting matrix (auto-generated)
├── commands/mod.rs                # Project CRUD + crawl + get_results + recrawl_page + screenshots + dashboard/comparator/pagespeed/duplicates/keywords
├── crawler/
│   ├── mod.rs                     # Exports: CrawlEngine, SemanticIssue, assets, screenshot
│   ├── engine.rs                  # CrawlEngine: URL dedup, include/exclude patterns, resume, html_body, snapshot+duplicates hooks
│   ├── fetcher.rs                 # HtmlFetcher trait + HttpFetcher (headers + timeout)
│   ├── parser.rs                  # SeoParser (17 semantic checks + nesting), readability, simhash, keywords, OG meta, visible text
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
│   ├── crawl_result.rs            # CrawlResult (+7 SEO fields), CrawlProgress, PageLink, SemanticIssue
│   └── project.rs                 # Project, CreateProjectRequest, RenameProjectRequest
├── data/                          # caninclude JSONs + generate.js (reference only for regeneration)
└── db/
    ├── mod.rs
    ├── schema.rs                  # Migrations: v4 (sessions/queue), v5 (html_body), v6 (screenshot_png) + runs migrations runner
    ├── migrations/                # mod.rs runner + 001-004 .sql migrations
    └── crawl_repo.rs              # CrawlRepo: filters, cache, sessions, screenshots, dashboard, snapshots, pagespeed, duplicates, keywords
```

### Frontend (src/)

```
src/
├── lib/
│   ├── tokens.css                 # Light/dark/system themes, semantic CSS variables
│   ├── i18n-issues.ts             # Issue name/message translation + param parsing (incl. context_nesting)
│   └── components/
│       ├── ResultsTable.svelte        # Virtualized table (Virtualizer class) + readability badge
│       ├── FilterBar.svelte           # Filters: status chips, severity chips, depth slider
│       ├── PageDetailPanel.svelte     # Full-page overlay: Overview | Page Preview | Links | PageSpeed | Social/OG
│       ├── SemanticDashboard.svelte   # Severity summary bar
│       ├── Dashboard.svelte           # Overview stats (status donut + keyword bars)
│       ├── Comparador.svelte          # Crawl A/B comparison (snapshots)
│       ├── SiteTree.svelte            # Tree view with issue badges + filter
│       ├── Duplicates.svelte          # Duplicate content groups (simhash)
│       ├── Keywords.svelte            # Keyword aggregation bars
│       ├── charts/DonutChart.svelte   # Pure-SVG donut
│       ├── charts/BarChart.svelte     # Pure-SVG bar chart
│       ├── SettingsModal.svelte       # Theme selector + pagespeed_api_key
│       ├── Toast.svelte               # Toast notifications
│       └── HtmlTree.svelte            # iframe preview with CSS highlight injection
├── routes/
│   ├── +layout.svelte                 # Root layout + theme application
│   ├── +layout.ts                     # Static adapter config (ssr = false)
│   └── +page.svelte                   # Main UI: sidebar, config, resume, filters, tabs (Results/Dashboard/Tree/Compare/Duplicates/Keywords), detailPageId
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
├── crawls: Arc<RwLock<HashMap<String, CrawlState>>>
│   └── CrawlState
│       ├── cancellation: CancellationToken
│       └── progress: CrawlProgress
└── transfer_server: Mutex<Option<TransferServerState>>  # active WiFi LAN server (P2)

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
cargo test:      ✅ 77/77 tests pass (parser, dedup, nesting_table, frontier, robots, migrations, duplicates, page_issues, filters incl. OR semantics, cookies/auth wiring, package round-trips incl. cross-project config re-homing, transfer server)
bun run build:  ✅ Frontend builds to /build
bun run check:  ✅ 0 errors / 0 warnings
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
- Build requires Bun 1.3+: `bun run build`
- SQL migrations live in `src-tauri/src/db/migrations/*.sql`, run by the `db/migrations/mod.rs` runner (transactional + idempotent, tracked in `schema_migrations`)
- PageSpeed uses the Google PSI v5 API (`runPagespeed`) — no API key required at low volume; set `pagespeed_api_key` in Settings to raise quota; results cached in `pagespeed_json`
- Duplicate detection uses `simhash::simhash` + `hamming_distance <= 10` (tunable); runs automatically after each crawl via non-blocking `tokio::spawn`
- Crawl snapshots are dumped transactionally to `crawl_snapshot_data` after each crawl for A/B comparison
- Content hash is `format!("{:x}", simhash::simhash(text))`; readability is Flesch Reading Ease clamped to 0-100
- Edits to `messages/en.json`/`es.json` require `node_modules/.bin/paraglide-js compile --project ./project.inlang --outdir ./src/lib/paraglide` (or `bun run build`) to regenerate Paraglide before `bun run check`
- `HttpFetcher::new(user_agent, timeout_ms, custom_headers, cookies, site_auth, proxy)` — arg order updated (cookies+site_auth added); robots/sitemap follow same pattern
- `RobotsChecker::new(user_agent, cookies, site_auth, proxy)` / `SitemapParser::new(user_agent, cookies, site_auth, proxy)`
- Helpers in `crawler/mod.rs`: `cookie_header_value(&[String]) -> Option<String>` (joins with "; "), `apply_basic_auth(RequestBuilder, &Option<SiteAuth>)` (basic_auth if username non-empty)
- `tauri-plugin-share` already registered (`lib.rs:65`) — mobile-only native share sheet; `share_export` in `export/commands.rs` is no-op on non-mobile
- DB lives at `~/Library/Application Support/com.opencrawler.app/open-crawler.db`; `crawled_pages` carries `html_body` (first 100KB) + `screenshot_png` BLOB — these make packages large; lightweight mode should drop them
- App targets: desktop (macOS/Win/Linux) + Android (`gen/` only has android; iOS noted but not built)

---

*Last updated: data sharing — P1 (.ocproj package + import) done, P2 (WiFi LAN server + TransferDialog frontend) backend done + frontend compiles, test fix → 73 tests, clippy clean. P2P plan approved (WebRTC PeerJS primary + libp2p Linux fallback + browser receiver). Next: P2 direct-share wiring, then P3 Bluetooth, P4 PeerJS, P5 libp2p.*
