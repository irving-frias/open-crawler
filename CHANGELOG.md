# Changelog

All notable changes to this project are documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2026-08-16

First stable release. SEO audit tool for web crawlers with semantic analysis,
site-wide SEO auditing and secure multi-device data sharing.

### Added

- **Auto-updater** (`tauri-plugin-updater`): signed update bundles published to
  GitHub Releases; the app checks for updates on launch (launcher window) and
  offers an explicit check button in Settings.
- **Site-wide SEO audit** — a new "SEO Insights" experience built on top of the
  per-page audit engine:
  - 50+ deterministic audit checks across 10 categories (meta, technical,
    social, accessibility, semantic_html, performance, ai_readability, sxo,
    security, compliance).
  - Robots.txt and sitemap.xml checks (status + well-formedness), plus
    hreflang self-reference validation.
  - JSON-LD validation, schema completeness scoring, FAQ/author/freshness
    checks and per-block evidence/examples.
  - Security headers (HSTS, CSP, X-Frame-Options, nosniff, referrer, permissions
    policy) and compliance signals (privacy policy, cookie consent, data
    protection schema).
  - Weighted category scoring (0-100), priorities and skipped-category handling.
- **Semantic HTML checks (parser)** — 33 issue types: article/time/figure/table/
  blockquote/iframe/video semantics, text-in-div, headings, ARIA, forms, empty
  tags, Open Graph `og:locale`, web app manifest, sectioned content and
  external sources cited.
- **Performance for large sites** — light result projection, SQL-aggregated
  duplicate/keyword/link reports, keyset-paginated site tree streaming,
  materialized `page_keywords` table, performance indexes, and incremental
  "show more" loading across list views.
- **Export** — expanded CSV/XLSX with 35 flat columns per page plus SEO Audit,
  SEO Checks and SEO Fixes sheets, redirect tracking and live progress events.

### Fixed

- Validators for empty tags (`empty_heading`, `empty_paragraph`,
  `empty_content_tag`).
- Cross-project configuration re-keying on `.ocproj` import.
- Duplicate i18n keys and Android build configuration.

### Security

- Upgraded `lru` to 0.18 (addresses a Stacked Borrows soundness advisory).
- Remaining RustSec findings (e.g. `glib`, transitive via GTK3) are warnings,
  not vulnerabilities; a GTK4 migration is tracked for a future release.

### Platform

- Desktop: macOS (Apple Silicon + Intel), Windows (MSI/exe), Linux
  (deb/rpm/AppImage).
- Android: arm64 APK and AAB, signed with a production keystore.
- iOS: not built yet (README no longer claims support).

## [0.3.0] - 2026-08-12

Third milestone release. 10 desktop platforms + Android APK/AAB.

### Added

- Link analysis (rel tokens, sponsored/ugc/internal flags, aggregate report).
- SEO security and compliance checks from persisted response headers.
- JSON-LD evidence/examples and skipped audit categories as N/A.
- Robots.txt, sitemap.xml and hreflang self-reference checks.
- Response header persistence in crawl results.

### Fixed

- Clippy `-D warnings` issues in link analysis ordering.

[1.0.0]: https://github.com/irving-frias/open-crawler/releases/tag/v1.0.0
[0.3.0]: https://github.com/irving-frias/open-crawler/releases/tag/0.3.0
