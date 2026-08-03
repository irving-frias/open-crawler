# FEATURES.md — Mapa de la arquitectura modular

Open-crawler se organiza por **dominios** (features) en los tres niveles: backend Rust, capa API TS y frontend Svelte. Cada feature es un módulo autocontenido que habla con el resto vía la API tipada, no con imports cruzados.

## Backend (Rust, Tauri)

### `src-tauri/src/features/`
Un directorio por dominio, con dos archivos cada uno:

- `mod.rs` — declara el módulo (`pub mod commands;`).
- `commands.rs` — comandos Tauri (`#[tauri::command]`). Sin acceso directo al DB; delegar en los repos.

Dominios: `analytics`, `app`, `crawl`, `export`, `pagespeed`, `projects`, `results`, `settings`, `snapshots`.

`features/mod.rs` expone el helper `with_repo(&state, |repo| ...)` que centraliza el lock de `CrawlRepo` y la inyección del repo, evitando repetir `state.crawl_repo.lock()` en cada comando.

`src-tauri/src/commands/mod.rs` es un registro **puro**: solo re-exporta los comandos por feature. No contiene lógica.

### `src-tauri/src/db/repos/`
Un archivo por dominio (`projects.rs`, `crawl.rs`, `results.rs`, `analytics.rs`, `export.rs`, `pagespeed.rs`, `settings.rs`, `snapshots.rs`). Cada repo encapsula consultas SQL sobre un `SqlitePool` inyectado. `mod.rs` define `CrawlRepo` (con `conn` privada) y tipos compartidos como `CrawlSessionInfo`, además de tests.

### Registro de comandos
`src-tauri/src/lib.rs` → `invoke_handler` usa una **lista explícita** de comandos agrupada por feature. Nota: el glob `generate_handler![commands::*]` **no compila** en Tauri v2; la lista debe ser explícita.

### Convenciones
- Los comandos reciben `State<AppState>` y delegan en repos; nunca tocan SQL directamente.
- Los tipos de entrada/salida de comandos son los contratos de la API TS (ver `types.ts`).

## Capa API (TypeScript)

### `src/lib/api/`
Espejo tipado del backend. Un módulo por dominio (`projects.ts`, `crawl.ts`, `results.ts`, ...) + `types.ts` con los tipos de contrato.

- `types.ts` — tipos TS manuales, espejo de los structs Rust (sin generación automática).
- `index.ts` — re-exporta todo con exports **namespace** y **flat**.
- Regla de oro: **cero `invoke(` en `src/`** (fuera de `src/lib/api/`).

## Frontend (Svelte 5, runes)

### `src/lib/features/`
Un directorio por dominio UI: `project/`, `crawl-controls/`, `results/`, `export/`, `page-detail/`, `dashboard/`, `semantic/`, `site-tree/`, `comparator/`, `duplicates/`, `keywords/`, `settings/`.

Reglas:
- Los componentes feature **no se importan entre sí** a través de `features/`; solo `+page.svelte` y los shells (o `$lib/...` compartido) los montan.
- Los componentes grandes se descomponen en subcarpetas, p. ej. `page-detail/sections/LinksSection.svelte` y `page-detail/sections/PreviewSection.svelte`.
- `src/lib/components/` queda reservado a genéricos: `charts/` y `ui/`.

### Capa shell (estado global)
- `src/lib/app.svelte.ts` — `createAppShell()` construye todo el estado/crawl/loadResults/export/listeners Tauri como un `$state` proxy con métodos en el objeto devuelto. Se instala con `setAppShell()` en `+page.svelte` y se obtiene con `getAppShell()` (clave `Symbol`). `projectsBase` es un `$state` local fuera del proxy (no referenciable dentro de closures como `useOptimistic`).
- `src/lib/tabs.ts` — registro de tabs: `TAB_DEFS` con loaders **lazy** por tab y `loadTabComponent()` con caché. `AnyComponent = any` por compatibilidad de tipos Svelte 5.
- `src/lib/use-optimistic.svelte.ts` — delete optimista (`useOptimistic<Project, OptimisticAction<Project>>`).

### Lazy-loading
Los paneles pesados (p. ej. `PageDetailPanel`, `SettingsModal`, tabs) se montan solo cuando se necesita, normalmente con `{@const Comp = components.x}` + `<Comp .../>` (en Svelte 5 `<svelte:component>` está deprecado y rompe `pnpm check`).

## Verificación
CI (`.github/workflows/ci.yml`): `pnpm build` + `cargo check` + `clippy -D warnings`.

- Backend: `cd src-tauri && cargo check && cargo clippy --all-targets && cargo test --lib`.
- Frontend: `pnpm check && pnpm build` (Node 22: `source ~/.nvm/nvm.sh && nvm use 22`).
