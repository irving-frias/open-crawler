# Open Crawler

SEO audit tool similar to Screaming Frog, built with **Rust + Tauri v2 + Svelte 5 + TypeScript**.

## Features

- **Parallel Web Crawl** — Concurrent fetching with configurable concurrency, domain-rotating frontier
- **17+ Semantic HTML Audit Checks** — Missing tags, heading hierarchy, ARIA, image alt, form labels, nesting validation
- **HTML Nesting Validation** — Flow/phrasing content rules with static table + caninclude API fallback
- **Sitemap Discovery** — Automatic robots.txt and sitemap.xml parsing (gzip supported)
- **Resume Interrupted Crawls** — Persistent crawl state with automatic recovery
- **Multi-Project Support** — Manage multiple crawl projects independently
- **Export** — XLSX (formatted, multi-sheet) and CSV with severity coloring
- **Search & Filter** — Real-time search by URL/title/H1 with highlighting
- **Issue Dashboard** — Severity breakdown, issue type counts, filterable
- **i18n** — English and Spanish (Paraglide)
- **Responsive Design** — Works on desktop and mobile (sidebar drawer)

## Screenshots

> Screenshots coming soon.

## Installation

### Pre-built (Recommended)

Download the latest release for your platform from [Releases](https://github.com/irving-frias/open-crawler/releases):

| Platform | Installer |
|----------|-----------|
| macOS (Apple Silicon) | `.dmg` |
| macOS (Intel) | `.dmg` |
| Windows (x64) | `.msi` |
| Linux (x64) | `.deb` |

### From Source

**Prerequisites:**
- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) 22+
- [pnpm](https://pnpm.io/) 8+

```bash
git clone https://github.com/irving-frias/open-crawler.git
cd open-crawler
pnpm install
cargo tauri dev
```

## Usage

1. **Create a project** — Click "New project" in the sidebar and enter a name
2. **Configure crawl** — Enter a seed URL and adjust settings (max depth, time limit, etc.)
3. **Start crawling** — Click "Start Crawl" and watch results stream in real-time
4. **View results** — Browse pages, expand issues, view page details with HTML preview
5. **Export** — Click "Export XLSX/CSV" to download formatted results

## Configuration

| Setting | Default | Description |
|---------|---------|-------------|
| **Max Depth** | 10 | How deep to crawl from the seed URL (1 = seed page only) |
| **Time Limit** | 3600s | Maximum crawl duration in seconds |
| **Respect robots.txt** | Yes | Honor crawl-delay and disallow rules |
| **Check sitemap.xml** | Yes | Discover URLs from sitemaps referenced in robots.txt |
| **Check semantic HTML** | Yes | Run 17+ accessibility and SEO checks |

## Semantic Audit Checks

| Check | Severity | Description |
|-------|----------|-------------|
| `missing_html_lang` | Error | Missing `lang` attribute on `<html>` |
| `missing_title` | Error | Missing `<title>` tag |
| `missing_meta_description` | Warning | Missing meta description |
| `missing_canonical` | Warning | Missing canonical link |
| `missing_main` | Warning | No `<main>` element |
| `missing_h1` | Warning | Missing `<h1>` tag |
| `multiple_h1` | Warning | Multiple `<h1>` tags |
| `heading_skip` | Warning | Heading level skips (e.g., h1 to h3) |
| `img_no_alt` | Error | Images missing `alt` attribute |
| `input_no_id` | Warning | Form elements missing `id` |
| `input_no_label` | Error | Inputs without associated `<label>` |
| `empty_link_text` | Warning | Links with no text or aria-label |
| `missing_aria` | Warning | Form controls missing aria-label |
| `invalid_nesting` | Error | Flow content inside phrasing content |

## Architecture

```
open-crawler/
├── src/                          # Frontend (Svelte 5 + TypeScript)
│   ├── lib/components/           # UI components
│   │   ├── ResultsTable.svelte   # Searchable results table
│   │   ├── PageDetailPanel.svelte # Full page details
│   │   ├── SemanticDashboard.svelte # Issue dashboard
│   │   ├── SettingsModal.svelte  # Settings
│   │   └── Toast.svelte          # Notifications
│   └── routes/                   # SvelteKit routes
├── src-tauri/                    # Backend (Rust + Tauri v2)
│   └── src/
│       ├── crawler/              # Crawl engine
│       │   ├── engine.rs         # Parallel crawl with Frontier
│       │   ├── parser.rs         # SEO parsing + semantic audit
│       │   ├── frontier.rs       # Domain-rotating priority queue
│       │   ├── robots.rs         # robots.txt parser
│       │   ├── sitemap.rs        # sitemap.xml parser
│       │   ├── assets.rs         # Asset inlining (CSS/images)
│       │   ├── screenshot.rs     # Chrome headless screenshots
│       │   └── db_writer.rs      # Batch DB writer actor
│       ├── commands/mod.rs       # Tauri commands (21 total)
│       ├── models/               # Data models
│       └── db/                   # SQLite schema + repository
├── messages/                     # i18n translations (EN, ES)
└── .github/workflows/            # CI/CD (GitHub Actions)
```

## Tech Stack

| Layer | Technology |
|-------|-----------|
| **Backend** | Rust, Tauri v2, rusqlite (SQLite), tokio, reqwest |
| **Frontend** | Svelte 5, TypeScript, SvelteKit, Paraglide i18n |
| **Build** | Vite, pnpm, adapter-static |
| **CI/CD** | GitHub Actions (4-platform: macOS ARM/Intel, Linux, Windows) |

## Development

```bash
# Install dependencies
pnpm install

# Start dev server (frontend only)
pnpm dev

# Start Tauri dev (full app)
cargo tauri dev

# Build for production
cargo tauri build

# Run tests
cargo test --lib

# Lint
cargo clippy -- -D warnings
```

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/my-feature`)
3. Make your changes
4. Run tests (`cargo test --lib`) and lint (`cargo clippy -- -D warnings`)
5. Commit your changes
6. Push to the branch and open a Pull Request

## License

[MIT](LICENSE)
