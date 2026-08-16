# Contributing to Open Crawler

Thanks for your interest in contributing! Open Crawler is a Tauri 2 + Svelte 5
desktop crawler with a Rust backend and a TypeScript/Svelte frontend.

## Development setup

**Prerequisites**

- [Rust](https://rustup.rs/) (stable)
- [Bun](https://bun.sh/) 1.3+
- Tauri system dependencies for your platform (see [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/))

```bash
bun install
bun tauri dev
```

## Codebase map

- `src-tauri/src/features/` — Tauri commands grouped by domain; they delegate to
  repos and never touch SQL directly.
- `src-tauri/src/db/repos/` — one repository per domain over a SQLite pool.
- `src-tauri/src/crawler/` — crawl engine, parser, robots/sitemap, assets.
- `src-tauri/src/seo/` — audit checks, scoring, priorities, site resources.
- `src/lib/api/` — typed TS wrappers mirroring the Rust commands (`types.ts` is
  the contract; no `invoke(` outside `src/lib/api/`).
- `src/lib/features/` — one directory per UI domain; feature components do not
  import each other directly.
- `messages/` — i18n (EN/ES); edit the JSON then regenerate Paraglide with
  `bun run build` (generated files live in `src/lib/paraglide/` and are
  git-ignored).

## Before submitting a change

Run the full verification locally:

```bash
# Backend
cd src-tauri && cargo fmt --check && cargo clippy -- -D warnings && cargo test --lib

# Frontend
bun run check
bun run lint
bun run format:check
bun run build
```

CI runs all of the above plus `cargo audit` and `bun audit`.

## Conventions

- SQL migrations live in `src-tauri/src/db/migrations/*.sql` (transactional,
  idempotent, tracked in `schema_migrations`). Add one new numbered file per
  schema change.
- Commands are registered explicitly in `src-tauri/src/lib.rs` (a
  `generate_handler!` glob does not compile with Tauri v2).
- Backend structs are the source of truth for the TS contract in
  `src/lib/api/types.ts`.
- New i18n keys go in both `messages/en.json` and `messages/es.json`; run
  `bun run build` afterwards to regenerate Paraglide.

## License

By contributing you agree that your contributions are licensed under the
[MIT License](LICENSE).
