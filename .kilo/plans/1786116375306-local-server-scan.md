# Plan: Local Server Scan Feature

## Goal
Allow users to manually input local web server URLs (e.g. nginx+php, Docker containers on `localhost`) and crawl them with the existing engine, reusing the same project/crawl config flow.

## Decisions (resolved)
- **Discovery**: Manual URL input only. User pastes `http://localhost:8080` style URLs.
- **Crawl behavior**: Same crawl config and engine. Local URLs are treated as additional seeds.
- **UI**: New "Local URLs" textarea + "Scan type" toggle in the existing crawl form.
- **Validation**: Validate reachability with a quick HTTP request before starting the crawl.
- **Persistence**: Add `scan_type` column to `crawl_config` table (migration v12). No new tables.

## Scope
- Backend: `CrawlConfig` model, DB migration, repo save/load, `start_crawl` validation, default robots override for local scans.
- Frontend: API types, crawl form UI, validation feedback.
- Out of scope: port scanning, LAN sweep, `file://` protocol support, mobile-specific localhost proxying.

## Implementation Tasks

### 1. Database Migration (v12)
**File**: `src-tauri/src/db/schema.rs`
- Add `scan_type TEXT DEFAULT 'web'` column to `crawl_config` using `ALTER TABLE` with `column_exists` guard.

### 2. CrawlConfig Model
**File**: `src-tauri/src/models/crawl_config.rs`
- Add `scan_type: ScanType` enum (`Web`, `Local`) with `Default` = `Web`.
- Add `local_urls: Vec<String>` with `#[serde(default)]`.
- No changes to other fields.

### 3. CrawlRepo Updates
**File**: `src-tauri/src/db/repos/crawl.rs`
- Update `save_config` to INSERT/REPLACE `scan_type`.
- Update `get_latest_session_config` (and any other config-loading query) to SELECT `scan_type`.
- On load, default to `ScanType::Web` if NULL/empty.

### 4. start_crawl Validation & defaults
**File**: `src-tauri/src/features/crawl/commands.rs`
- In `start_crawl_internal`, before spawning the engine:
  - If `config.scan_type == Local`:
    - Append `config.local_urls` to `config.seed_urls`.
    - Set `config.respect_robots = false` unless the user explicitly set it to `true` in the UI.
  - Validate each local URL with a lightweight request (HEAD, fallback GET, ~5s timeout) using the existing `HttpFetcher`.
  - If any local URL fails validation, return `AppError::Crawl` with a message listing the failing URLs and reasons. Do not start the engine.
- Keep existing behavior for `ScanType::Web`.

### 5. Frontend API Types
**File**: `src/lib/api/types.ts`
- Add `scan_type?: 'web' | 'local'` and `local_urls?: string[]` to the `CrawlConfig` interface.

### 6. Frontend Crawl Form UI
**File**: `src/routes/+page.svelte` (or crawl form component)
- Add a radio/toggle group: `Scan type: Web | Local`.
- When `Local` is selected, show a textarea labeled "Local URLs" (one URL per line).
- Pass `scan_type` and `local_urls` into the crawl config on submit.
- Show validation errors returned from `start_crawl` (e.g., "Could not reach http://localhost:8080: connection refused").

### 7. Tests / Validation
- Run `cargo check` / `cargo test` to verify Rust compiles.
- Run `npm run check` / `npm run lint` to verify frontend types.
- Manual test: create project, select Local scan, paste `http://localhost:3000`, verify crawl starts and pages appear in results.
- Edge case: submit unreachable local URL, verify error message.

## Risks / Notes
- **Mobile**: On mobile, `localhost` resolves to the device itself, not the host machine. The UI should show a small hint that local scans are intended for desktop.
- **Robots**: For local scans, `robots.txt` is usually irrelevant. We disable it by default for `Local` scans unless the user explicitly toggles it on.
- **CORS / Host headers**: Some local dev servers require specific `Host` headers. The existing `custom_headers` field in `CrawlConfig` already covers this; no new changes needed.
- **Link discovery**: Since local URLs share the same `same_origin_only` logic, links discovered from `localhost:3000` will only expand to `localhost:3000` origins, preventing accidental crawling of external sites.
