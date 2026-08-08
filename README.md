# Open Crawler

SEO audit tool similar to Screaming Frog, built with **Rust + Tauri v2 + Svelte 5 + TypeScript**. Desktop (macOS, Windows, Linux) and mobile (Android/iOS) via Tauri.

## Features

- **Parallel Web Crawl** — Concurrent fetching with configurable concurrency, delay, max depth and time limit, domain-restricted frontier, include/exclude patterns, custom headers, and optional HTTP proxy
- **18+ Semantic HTML Audit Checks** — Missing tags, heading hierarchy, ARIA, image alt, form labels, nesting validation
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

> Screenshots coming soon.

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
| **Check semantic HTML**      | Yes     | Run 18+ accessibility and SEO checks                     |
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

## Semantic Audit Checks

| Check                      | Severity | Description                                   |
| -------------------------- | -------- | --------------------------------------------- |
| `missing_html_lang`        | Error    | Missing `lang` attribute on `<html>`          |
| `missing_title`            | Error    | Missing `<title>` tag                         |
| `missing_meta_description` | Warning  | Missing meta description                      |
| `missing_canonical`        | Warning  | Missing canonical link                        |
| `missing_main`             | Warning  | No `<main>` element                           |
| `missing_header`           | Warning  | No `<header>` element                         |
| `missing_footer`           | Warning  | No `<footer>` element                         |
| `missing_nav`              | Warning  | No `<nav>` element                            |
| `missing_h1`               | Warning  | Missing `<h1>` tag                            |
| `multiple_h1`              | Warning  | Multiple `<h1>` tags                          |
| `heading_skip`             | Warning  | Heading level skips (e.g., h1 to h3)          |
| `img_no_alt`               | Error    | Images missing `alt` attribute                |
| `input_no_id`              | Warning  | Form elements missing `id`                    |
| `input_no_label`           | Error    | Inputs without associated `<label>`           |
| `empty_link_text`          | Warning  | Links with no text or aria-label              |
| `missing_aria`             | Warning  | Form controls missing aria-label              |
| `invalid_nesting`          | Error    | Flow content inside phrasing content          |
| `context_nesting`          | Error    | Elements nested outside their allowed context |

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
│   │   │   ├── app/              # App bootstrap (32 commands total)
│   │   │   ├── analytics/        # Dashboard statistics
│   │   │   ├── crawl/            # Crawl engine control
│   │   │   ├── export/           # XLSX/CSV export
│   │   │   ├── pagespeed/        # PageSpeed integration
│   │   │   ├── projects/         # Project CRUD
│   │   │   ├── results/          # Paginated/filtered results
│   │   │   ├── settings/         # Key/value settings store
│   │   │   └── snapshots/        # Crawl snapshots + compare
│   │   ├── crawler/              # Crawl engine
│   │   │   ├── engine.rs         # Parallel crawl with Frontier
│   │   │   ├── parser.rs         # SEO parsing + semantic audit
│   │   │   ├── frontier.rs       # Domain-rotating priority queue
│   │   │   ├── robots.rs         # robots.txt parser
│   │   │   ├── sitemap.rs        # sitemap.xml parser
│   │   │   ├── assets.rs         # Asset inlining (CSS/images)
│   │   │   ├── screenshot.rs     # Headless screenshots
│   │   │   └── db_writer.rs      # Batch DB writer actor
│   │   ├── db/                   # SQLite schema + repositories
│   │   ├── models/               # Data models
│   │   └── nesting_table.rs      # HTML nesting rules
│   ├── gen/                      # Android/iOS generated projects
│   └── icons/                    # App icons (radar, all platforms)
├── messages/                     # i18n translations (EN, ES)
└── .github/workflows/            # CI/CD (GitHub Actions)
```

## Tech Stack

| Layer        | Technology                                                                                |
| ------------ | ----------------------------------------------------------------------------------------- |
| **Backend**  | Rust, Tauri v2, rusqlite (SQLite), tokio, reqwest                                         |
| **Frontend** | Svelte 5, TypeScript, SvelteKit, shadcn-svelte, Chart.js, Lucide icons, Paraglide i18n    |
| **Build**    | Vite, Bun, adapter-static                                                                 |
| **CI/CD**    | GitHub Actions — lint/test, desktop builds (macOS ARM/Intel, Linux, Windows), Android APK |

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
