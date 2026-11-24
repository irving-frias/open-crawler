# Changelog

All notable changes to this project are documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.1] - 2026-11-24

Patch release after three months paused. No new features.

### Fixed

- Clippy warning in OBEX Unicode decoding (`src-tauri/src/transfer/obex/codec.rs`): `chunks_exact(2)` replaced with `as_chunks::<2>()`.
- `svelte-check` in a fresh clone failed with missing Paraglide modules until a build had run; documented order is build first, then check.

### Changed

- Dependency patches (no major upgrades, no GTK4 migration): Tauri 2.11.5 -> 2.12.7 with matching `tauri-plugin-*` crates, `lru` 0.18.2 -> 0.18.5, Tokio 1.53.1 -> 1.54.1; frontend patches within the same majors (Svelte 5.59.1, Vite 6.4.6, TypeScript 5.9.3, Paraglide 2.29.3).
- `glib` stays at 0.18.5 (allowed warning, RUSTSEC-2024-0429, GTK3 pinned by Tauri).

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

[1.0.1]: https://github.com/irving-frias/open-crawler/releases/tag/v1.0.1
[1.0.0]: https://github.com/irving-frias/open-crawler/releases/tag/v1.0.0
[0.3.0]: https://github.com/irving-frias/open-crawler/releases/tag/0.3.0
