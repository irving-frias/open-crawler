# Open Crawler

SEO audit tool similar to Screaming Frog, built with **Rust + Tauri v2 + Svelte 5 + TypeScript**. Desktop (macOS, Windows, Linux) and Android via Tauri.

## Features

- **Parallel Web Crawl** — Concurrent fetching with configurable concurrency, delay, max depth and time limit, domain-restricted frontier, include/exclude patterns, custom headers, and optional HTTP proxy
- **33 Semantic HTML Checks** — Missing tags, heading hierarchy, ARIA, image alt, form labels, nesting validation, article/figure/table/blockquote/iframe/video semantics
- **Site-wide SEO Audit** — 97 deterministic checks across 10 categories (meta, technical, accessibility, semantic HTML, performance, AI readability, SXO, security, compliance) with weighted 0-100 scoring
- **Structured Data Checks** — JSON-LD validation, schema completeness, FAQ/author/freshness signals, robots.txt & sitemap.xml validation
- **Auto-updates** — Signed update bundles via GitHub Releases with in-app update checks (launcher + Settings)
- **HTML Nesting Validation** — Flow/phrasing content rules with static table + caninclude API fallback
- **Sitemap Discovery** — Automatic robots.txt and sitemap.xml parsing (gzip supported)
- **Resume Interrupted Crawls** — Persistent crawl state with automatic recovery
- **Multi-Project Support** — Manage multiple crawl projects independently
- **Search & Filter** — Real-time search by URL/title/H1 with highlighting; facet filters (status, severity, issue type, depth, domain, missing/duplicate title, noindex, 404) combined as a union
- **Issue Dashboard** — Severity breakdown, issue type counts, filterable, deep-links to results
- **Site Tree** — Interactive tree of crawled pages with issue filtering
- **Crawl Snapshots & Comparator** — Save crawl results and diff two snapshots field by field
- **Duplicate Detection** — Groups pages with duplicate titles or content hashes
- **Keyword Extraction** — Frequency analysis of title/H1 keywords across pages
- **PageSpeed Insights** — Optional per-page Core Web Vitals via the PageSpeed API
- **Export** — XLSX (formatted, multi-sheet) and CSV with severity coloring and live progress
- **Visual Style System** — Five UI styles (Classic, Neumorph, Clay, Glass, Brutalism) with dark/light/system themes
- **Notifications** — Native desktop and mobile notifications for crawl lifecycle events
- **i18n** — English and Spanish (Paraglide)
- **Responsive Design** — Works on desktop and mobile (sidebar drawer)

## Screenshots

| Launcher                                   | Crawl results                            | SEO audit                                    |
| ------------------------------------------ | ---------------------------------------- | -------------------------------------------- |
| ![Launcher](docs/screenshots/launcher.png) | ![Results](docs/screenshots/results.png) | ![SEO audit](docs/screenshots/seo-audit.png) |

## Installation

### Pre-built (Recommended)

Download the latest release for your platform from [Releases](https://github.com/irving-frias/open-crawler/releases):

| Platform              | Installer |
| --------------------- | --------- |
| macOS (Apple Silicon) | `.dmg`    |
| macOS (Intel)         | `.dmg`    |
| Windows (x64)         | `.msi`    |
| Linux (x64)           | `.deb`    |
| Android               | `.apk`    |

### From Source

**Prerequisites:**

- [Rust](https://rustup.rs/) (stable)
- [Bun](https://bun.sh/) 1.3+

```bash
git clone https://github.com/irving-frias/open-crawler.git
cd open-crawler
bun install
bun tauri dev
```

## Usage

1. **Create a project** — Click "New project" in the sidebar and enter a name
2. **Configure crawl** — Enter a seed URL and adjust settings (max depth, time limit, concurrency, etc.)
3. **Start crawling** — Click "Start Crawl" and watch results stream in real-time
4. **Analyze** — Use the tabs: Overview, Issues Dashboard, Site Tree, Comparator, Duplicates, Keywords
5. **View details** — Click any row to inspect page metadata, semantic issues, links, HTML preview, and PageSpeed scores
6. **Export** — Click "Export XLSX/CSV" to download formatted results

## Configuration

### Crawl settings

| Setting                      | Default | Description                                              |
| ---------------------------- | ------- | -------------------------------------------------------- |
| **Max Depth**                | 10      | How deep to crawl from the seed URL (1 = seed page only) |
| **Time Limit**               | 3600s   | Maximum crawl duration in seconds                        |
| **Concurrency**              | 10      | Number of parallel page fetches                          |
| **Crawl Delay**              | 100ms   | Delay between requests to the same host                  |
| **Respect robots.txt**       | Yes     | Honor crawl-delay and disallow rules                     |
| **Check sitemap.xml**        | Yes     | Discover URLs from sitemaps referenced in robots.txt     |
| **Same-origin only**         | Yes     | Restrict crawling to the seed domain                     |
| **Check semantic HTML**      | Yes     | Run 33 accessibility and SEO checks                      |
| **Include/Exclude patterns** | —       | URL glob patterns to include or exclude                  |
| **Custom headers**           | —       | Extra HTTP headers sent on each request                  |
| **Proxy**                    | Off     | Optional HTTP proxy with basic auth                      |

### App settings

| Setting                  | Default | Description                                   |
| ------------------------ | ------- | --------------------------------------------- |
| **Theme**                | System  | Dark / Light / System                         |
| **UI Style**             | Classic | Classic / Neumorph / Clay / Glass / Brutalism |
| **Results per page**     | 50      | Default table page size                       |
| **Notifications**        | On      | Native desktop/mobile notifications           |
| **PageSpeed API key**    | —       | Google API key for per-page Core Web Vitals   |
| **Default crawl config** | —       | Values pre-filled on new crawls               |
| **Update check**         | On      | Check for app updates on launch + Settings    |

## Semantic Audit Checks

### Page-level HTML (33 checks)

| Check                             | Severity | Description                                   |
| --------------------------------- | -------- | --------------------------------------------- |
| `missing_html_lang`               | Error    | Missing `lang` attribute on `<html>`          |
| `missing_title`                   | Error    | Missing `<title>` tag                         |
| `missing_meta_description`        | Warning  | Missing meta description                      |
| `missing_canonical`               | Warning  | Missing canonical link                        |
| `missing_main`                    | Warning  | No `<main>` element                           |
| `missing_header`                  | Warning  | No `<header>` element                         |
| `missing_footer`                  | Warning  | No `<footer>` element                         |
| `missing_nav`                     | Warning  | No `<nav>` element                            |
| `missing_h1`                      | Warning  | Missing `<h1>` tag                            |
| `multiple_h1`                     | Warning  | Multiple `<h1>` tags                          |
| `heading_skip`                    | Warning  | Heading level skips (e.g., h1 to h3)          |
| `img_no_alt`                      | Error    | Images missing `alt` attribute                |
| `img_no_dimensions`               | Warning  | Images missing width/height                   |
| `input_no_id`                     | Warning  | Form elements missing `id`                    |
| `input_no_label`                  | Error    | Inputs without associated `<label>`           |
| `empty_link_text`                 | Warning  | Links with no text or aria-label              |
| `empty_heading`                   | Warning  | Headings with empty content                   |
| `empty_paragraph`                 | Warning  | Empty `<p>` elements                          |
| `empty_content_tag`               | Warning  | Empty generic content tags                    |
| `missing_aria`                    | Warning  | Form controls missing aria-label              |
| `invalid_nesting`                 | Error    | Flow content inside phrasing content          |
| `context_nesting`                 | Error    | Elements nested outside their allowed context |
| `multiple_main`                   | Error    | More than one `<main>` element                |
| `article_missing`                 | Warning  | `<main>` with text but no `<article>`         |
| `time_without_datetime`           | Warning  | `<time>` without `datetime`                   |
| `figure_without_caption`          | Info     | `<figure>` without `<figcaption>`             |
| `table_without_headers`           | Error    | `<table>` without `<th>`                      |
| `table_without_caption`           | Info     | `<table>` without `<caption>`                 |
| `blockquote_without_attribution`  | Warning  | `<blockquote>` without `cite` or author       |
| `text_in_div_instead_of_p`        | Warning  | `<div>` with substantial direct text          |
| `iframe_without_title`            | Warning  | `<iframe>` without `title`                    |
| `video_without_track_or_controls` | Warning  | `<video>` without controls or track           |
| `aria_current_nav`                | Info     | `aria-current` absent in nav                  |

### Site-wide SEO audit (97 checks)

Separate audit engine (`seo/`) covering 10 categories: **meta**, **technical**
(robots.txt, sitemap.xml, hreflang self-reference, canonical), **accessibility**,
**semantic_html**, **performance** (render-blocking scripts, lazy loading, ratios),
**ai_readability**, **sxo** (JSON-LD validity/completeness, FAQ/author/freshness),
**security** (HSTS, CSP, X-Frame-Options, nosniff, referrer, permissions policy)
and **compliance** (privacy policy, cookie consent, data protection schema).
Results are scored 0-100 with category weights and exported per page.

## Architecture

```
open-crawler/
├── src/                          # Frontend (Svelte 5 + TypeScript)
│   ├── lib/api/                  # Typed Tauri command wrappers
│   ├── lib/components/
│   │   ├── ui/                   # shadcn-svelte UI primitives
│   │   └── charts/               # Chart.js wrappers (theme-aware)
│   ├── lib/features/
│   │   ├── results/              # Results table + filter bar + pagination
│   │   ├── dashboard/            # Overview (KPIs, charts)
│   │   ├── semantic/             # Issues dashboard
│   │   ├── site-tree/            # Interactive crawl tree
│   │   ├── comparator/           # Snapshot diffing
│   │   ├── duplicates/           # Duplicate groups
│   │   ├── keywords/             # Keyword frequency
│   │   ├── page-detail/          # Page inspector panel
│   │   ├── project/              # App header, project sidebar
│   │   ├── settings/             # Settings modal
│   │   ├── crawl-controls/       # Crawl start/stop/resume + progress
│   │   ├── export/               # Export progress UI
│   │   └── splash/               # Startup splash
│   ├── lib/tokens.css            # Design tokens (5 UI styles × themes)
│   └── routes/                   # SvelteKit routes
├── src-tauri/                    # Backend (Rust + Tauri v2)
│   ├── src/
│   │   ├── features/             # Command groups by domain
│   │   │   ├── analytics/        # Dashboard statistics
│   │   │   ├── crawl/            # Crawl engine control
│   │   │   ├── export/           # XLSX/CSV export
│   │   │   ├── links/            # Link analysis
│   │   │   ├── pagespeed/        # PageSpeed integration
│   │   │   ├── projects/         # Project CRUD
│   │   │   ├── results/          # Paginated/filtered results
│   │   │   ├── schedule/         # Scheduled crawls
│   │   │   ├── seo/              # Site-wide SEO audit
│   │   │   ├── settings/         # Key/value settings store
│   │   │   ├── snapshots/        # Crawl snapshots + compare
│   │   │   └── transfer/         # .ocproj package import/export
│   │   ├── crawler/              # Crawl engine
│   │   │   ├── engine.rs         # Parallel crawl with Frontier
│   │   │   ├── parser.rs         # SEO parsing + semantic audit
│   │   │   ├── frontier.rs       # Domain-rotating priority queue
│   │   │   ├── fetcher.rs        # HTTP client + response headers
│   │   │   ├── robots.rs         # robots.txt parser
│   │   │   ├── sitemap.rs        # sitemap.xml parser
│   │   │   ├── dedup.rs          # Duplicate detection
│   │   │   ├── assets.rs         # Asset inlining (CSS/images)
│   │   │   ├── screenshot.rs     # Headless screenshots
│   │   │   └── db_writer.rs      # Batch DB writer actor
│   │   ├── seo/                  # Audit checks, scoring, priorities
│   │   │   ├── audit.rs          # AuditContext + orchestration
│   │   │   ├── checks.rs         # 97 checks across 10 categories
│   │   │   ├── score.rs          # Category weights + 0-100 score
│   │   │   ├── priority.rs       # severity → priority mapping
│   │   │   └── site.rs           # robots.txt/sitemap resources
│   │   ├── db/                   # SQLite schema + repositories
│   │   ├── models/               # Data models
│   │   └── nesting_table.rs      # HTML nesting rules
│   ├── gen/                      # Android generated project
│   └── icons/                    # App icons (radar, all platforms)
├── messages/                     # i18n translations (EN, ES)
└── .github/workflows/            # CI/CD (GitHub Actions)
```

## Tech Stack

| Layer        | Technology                                                                                                                     |
| ------------ | ------------------------------------------------------------------------------------------------------------------------------ |
| **Backend**  | Rust, Tauri v2, rusqlite (SQLite), tokio, reqwest                                                                              |
| **Frontend** | Svelte 5, TypeScript, SvelteKit, shadcn-svelte, Chart.js, Lucide icons, Paraglide i18n                                         |
| **Build**    | Vite, Bun, adapter-static                                                                                                      |
| **CI/CD**    | GitHub Actions — lint/test, desktop builds (macOS ARM/Intel, Linux, Windows), Android APK, signed update bundles (latest.json) |

## Development

```bash
# Install dependencies
bun install

# Start dev server (frontend only)
bun run dev

# Start Tauri dev (full app)
bun tauri dev

# Build for production
bun tauri build

# Svelte type-check
bun run check

# Run Rust tests
cargo test --lib

# Lint
cargo clippy -- -D warnings
```

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/my-feature`)
3. Make your changes
4. Run tests (`cargo test --lib`), lint (`cargo clippy -- -D warnings`) and type-check (`bun run check`)
5. Commit your changes
6. Push to the branch and open a Pull Request

## License

[MIT](LICENSE)
