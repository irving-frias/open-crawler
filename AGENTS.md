# Open Crawler — Contexto de trabajo

Proyecto Tauri 2 + SvelteKit 5 (Svelte 5 runes) + Rust. Crawler de sitios web con
análisis semántico/SEO. Este archivo guarda el contexto del trabajo en curso para
retomarlo en futuras sesiones.

## Estado actual (al iniciar la mejora)

### Backend (Rust, en `src-tauri/src/`)

- `crawler/parser.rs` — `SeoParser::analyze_semantics()` emite **24 issue types**:
  `missing_html_lang`, `missing_title`, `missing_meta_description`, `missing_canonical`,
  `missing_main`, `missing_header`, `missing_footer`, `missing_nav`, `missing_h1`,
  `multiple_h1`, `heading_skip`, `img_no_alt`, `img_no_dimensions`, `input_no_id`,
  `input_no_label`, `empty_link_text`, `missing_aria`, `invalid_nesting`,
  `context_nesting`, `multiple_main`, `skip_link_missing`, `section_no_accessible_name`,
  `form_no_accessible_name`, `button_no_name`, `duplicate_id`, `aria_current_nav`.
  (constante `MAX_ELEMENT_ISSUES_PER_TYPE = 25`; helper `issue()` con xpath/css/snippet)
- `seo/checks.rs` — `PageExtras::extract(html, url)` (segunda pasada ligera de DOM) y
  `run_all()` con **50+ checks** en 10 categorías: `meta`, `technical`, `social`,
  `accessibility`, `semantic_html`, `performance`, `ai_readability`, `sxo`,
  `security`, `compliance`.
- `seo/audit.rs` — `AuditContext` (url, status, size, load_time, pagespeed_score,
  `response_headers: HashMap<String,String>` lowercase), `CheckResult`
  (id/category/severity/passed/weight/message/guidance/evidence/examples),
  `SeoAuditResult`. `audit_page()` orquesta.
- `seo/score.rs` — `CATEGORY_ORDER` + `CATEGORY_WEIGHTS`, score 0-100 ponderado.
- `seo/priority.rs` — prioridades: error→critical, warning→important, info→minor.
- El audit se almacena en `crawled_pages.seo_audit_json` (JSON).

### Frontend (Svelte, en `src/lib/`)

- `seo-checks.ts` — `DICT` (message/guidance por check), `CHECK_FIXES` (fix/expected),
  `WHY`, y `localizeSeoCheck(id, message, guidance, evidence)`.
- `i18n-issues.ts` — `issueNames`/`issueMessages`/`ISSUE_FIXES` + `translateIssue*`
  para los issue types del parser. `parseIssueParams()` extrae parámetros del mensaje.
- `seo-ui.ts` — helpers de score→color/grado/badge y `seoCategoryGains()`.
- Paneles: `features/seo/SiteSeoPanel.svelte` (audit por sitio) y
  `features/page-detail/PageDetailPanel.svelte` (audit por página).

## Trabajo en curso: mejora de semántica y SEO (sin IA)

Aprobado por el usuario. Objetivo: añadir checks de HTML semántico y SEO basados en
tendencias 2026 (búsqueda generativa AIO/AEO/GEO, extracción por pasajes, freshness,
E-E-A-T, schema completo, ratios técnicos). Todo determinista, **sin IA/NLP**.

### Fase 1 — HTML semántico ✅ HECHO

Nuevos issue types (con `issue()`, cap 25):

| issue_type                        | severity | criterio                                    |
| --------------------------------- | -------- | ------------------------------------------- |
| `article_missing`                 | warning  | hay `<main>` con texto pero sin `<article>` |
| `time_without_datetime`           | warning  | `<time>` sin atributo `datetime`            |
| `figure_without_caption`          | info     | `<figure>` sin `<figcaption>`               |
| `table_without_headers`           | error    | `<table>` sin `<th>`                        |
| `table_without_caption`           | info     | `<table>` sin `<caption>`                   |
| `blockquote_without_attribution`  | warning  | `<blockquote>` sin `cite` ni autor          |
| `text_in_div_instead_of_p`        | warning  | `<div>` con ≥20 palabras de texto directo   |
| `iframe_without_title`            | warning  | `<iframe>` sin `title`                      |
| `video_without_track_or_controls` | warning  | `<video>` sin `controls` ni `<track>`       |

Implementado en `crawler/parser.rs` (issues 25-33, helpers nuevos:
`main_has_substantial_text`, `figures_without_caption`, `tables_without_headers`,
`tables_without_caption`, `blockquotes_without_attribution`,
`videos_without_controls`, `divs_with_direct_text`, `direct_text_words`,
`element_has_child` via `el.select`). Import de `scraper::NodeRef` añadido para
`el.children()`. Tests nuevos en el `mod tests` (5 tests). i18n completo:
messages de paraglide en `messages/{en,es}.json` (`issue.<type>.name|message`) y
`issueNames`/`issueMessages`/`ISSUE_FIXES` en `src/lib/i18n-issues.ts`.

**OJO (nota de sesión):** la capa de herramientas mostró salidas corruptas
(duplicación de contenido, diff mínimo falso). No confiar en `tail`/`read` con
salidas largas; verificar ediciones con `git diff --numstat` y ejecutar tests.

### Fase 2 — Checks de auditoría ✅ HECHO

Nuevos checks en `seo/checks.rs` (`run_all` + `PageExtras`):

| check                          | categoría     | criterio                                       |
| ------------------------------ | ------------- | ---------------------------------------------- |
| `content_to_html_ratio`        | performance   | ratio texto/HTML ≥ 0.08                        |
| `internal_external_ratio`      | meta          | ratio enlaces int/ext ≥ 10:1                   |
| `canonical_self_reference`     | technical     | canonical apunta a la propia URL               |
| `meta_robots_directives`       | technical     | sin nofollow/nosnippet                         |
| `meta_description_topic_match` | meta          | descripción comparte keywords con title        |
| `json_ld_valid`                | sxo           | todo bloque tiene @context + @type             |
| `schema_completeness`          | sxo           | ≥60% de props requeridas por tipo              |
| `faq_accordion_without_schema` | sxo           | details/summary o H2-pregunta sin FAQPage      |
| `author_schema`                | sxo           | Article sin author/Person                      |
| `freshness_dates`              | sxo           | datePublished/dateModified o `<time datetime>` |
| `direct_answer_section`        | sxo           | bloque 40-100 palabras tras H1                 |
| `img_srcset`                   | performance   | todas las imágenes con srcset                  |
| `lazy_loading`                 | performance   | loading explícito en imágenes                  |
| `render_blocking_scripts`      | performance   | sin scripts externos sin async/defer           |
| `autocomplete_inputs`          | accessibility | ≥50% de inputs con autocomplete                |
| `table_headers`                | semantic_html | tablas con `<th>`                              |
| `table_captions`               | semantic_html | tablas con `<caption>`                         |
| `figure_captions`              | semantic_html | figuras con `<figcaption>`                     |
| `iframe_titles`                | semantic_html | iframes con `title`                            |
| `video_accessible`             | semantic_html | videos con controls o `<track>`                |

`PageExtras` nuevos campos: `json_ld_raw`, `img_srcset`, `img_lazy`, `table_total`,
`table_with_headers`, `table_with_caption`, `figure_total`, `figure_with_caption`,
`details_summary`, `video_total`, `video_accessible`, `iframe_total`,
`iframe_with_title`, `input_total`, `autocomplete_inputs`, `text_bytes`,
`html_bytes`, `div_with_direct_text`, `scripts_render_blocking`, `internal_count`,
`external_count`, `meta_robots`, `answer_section_words`, `time_datetime_count`.
Helpers nuevos: `direct_text_words`, `answer_section_words`, `ratio_display`,
`all_json_ld_valid`, `schema_completeness_score`, `json_ld_has_author`,
`json_ld_has_freshness`. `SeoData` ahora deriva `Default`. `SEMANTIC_CHECK_MAP`
ampliado con 5 checks (table/figure/iframe/video). Tests: `mod tests` nuevo en
checks.rs (6 tests). i18n: `DICT`/`WHY`/`CHECK_FIXES` en `src/lib/seo-checks.ts`.

### Fase 3 — Score y UI ✅ HECHO

- Nueva categoría **`semantic_html`** en `seo/score.rs` (CATEGORY_ORDER + CATEGORY_WEIGHTS).
  Pesos recalibrados para sumar 1.0: meta 0.25, technical 0.18, accessibility 0.12,
  semantic_html 0.12, performance 0.10, ai_readability 0.08, sxo 0.08, social 0.07.
- Checks reasignados a `semantic_html`: los 5 semánticos de Fase 2 (table_headers,
  table_captions, figure_captions, iframe_titles, video_accessible) salen de
  `accessibility`; el check de landmarks `semantic_html` sale de `ai_readability`.
- `seo_category_scores` se normaliza automáticamente desde `audit.categories`
  (`db/repos/seo.rs`), sin cambios de esquema. SiteSeoPanel/PageDetailPanel son
  data-driven (solo añadir etiqueta).
- i18n: `DICT`/`CHECK_FIXES`/`WHY` en `src/lib/seo-checks.ts` (17 checks nuevos:
  15 de Fase 2 + table/figure/iframe/video); issue types en `i18n-issues.ts` (Fase 1);
  label `seo.category.semantic_html` en `messages/{en,es}.json` y en los maps de
  PageDetailPanel y SiteSeoPanel.
- Tests ajustados: audit.rs espera 8 categorías; score.rs usa comparación con
  epsilon (precisión de coma flotante). `cargo test --lib`: 122 OK.

### Fase 4 — Export CSV/XLSX ampliado ✅ HECHO

- **`db/repos/export.rs`**: `count_issues` cuenta todas las severidades (antes solo
  error); nuevo `count_seo_rows(project_id)` (suma sobre `json_each` de
  `categories` + `checks` con `passed=0` + `priority_fixes` del `seo_audit_json`);
  `get_result_batch` añade col `redirect_from_url` (índice 26, `duplicate_group_id`
  pasa a 27).
- **`models/crawl_result.rs`**: nuevo campo `redirect_from_url: Option<String>`
  (`#[serde(default)]`). Constructores actualizados: `engine.rs` (lo rellena con
  `response.redirect_from_url`), `results.rs` (light idx 28 / full idx 29),
  `features/results/commands.rs`, test helpers.
- **`features/mod.rs`**: `with_repo` ahora delega en `with_repo_arc(&Arc<RwLock<AppState>>, f)`
  para poder testear los writers sin Tauri.
- **`features/export/commands.rs`** reescrito:
  - Writers (`export_csv_single`, `export_xlsx`) toman `&Arc<RwLock<AppState>>` +
    `emit: &(dyn Fn(&'static str, u64, u64) + Sync + 'static)` (el comando pasa
    closure con `emit_export_progress` → evento `export-progress`).
  - `page_values()` → 33 columnas planas (URL…Redirect From + 8 categorías desde
    `CATEGORY_ORDER` + Keywords/OG/Hreflang JSON). `page_headers()` genera los
    headers en el mismo orden. CSV y XLSX Pages la reusan.
  - XLSX: 5 pases vía `export_page_passes` (Pages, Issues ya sin filtro de
    severidad, **SEO Audit**, **SEO Checks** !passed, **SEO Fixes**) + Links.
    `PagePass {stage, base_name, headers, total_rows, widths, write: fn}`;
    split por `MAX_ROWS_PER_SHEET` reutilizado.
  - `xlsx_str` trunca a 32.767 chars (límite de Excel) para blobs JSON grandes —
    evita "String exceeds Excel's limit of 32,767 characters".
  - `check_elements` (elemento+snippet, ≤3), `category_label`, `parse_audit`.
  - Tests: `mod tests` con `AppState` en memoria + `run_migrations` + inserts de
    proyecto/cfg; `save_redirect_batch` para poblar `redirect_from_url` (no se
    persiste en `save_results_batch`); 4 tests (page_values 33 cols, CSV con
    columnas SEO, nombres de hojas vía `workbook.worksheets()`+`name()`, archivo xlsx).
  - `cargo test --lib`: 126 OK. Clippy limpio (salvo warning pre-existente
    `run_default` en checks.rs).

### Fase 5 — Security y Compliance (headers de respuesta) ✅ HECHO

- **`seo/audit.rs`**: `AuditContext` gana `response_headers: HashMap<String,String>`
  (claves lowercase). Se propaga desde `crawler/fetcher.rs` (reqwest normaliza a
  lowercase) en los 3 call sites: `crawler/engine.rs`, `features/results/commands.rs`,
  `features/seo/commands.rs` (2). Test helpers usan `Default::default()`.
- **6 checks `security`** (categorías ya existían en score.rs, pesos 0.12/0.08):

| check                     | severity | criterio                                  |
| ------------------------- | -------- | ----------------------------------------- |
| `hsts_header`             | warning  | `Strict-Transport-Security` presente      |
| `x_content_type_options`  | warning  | header contiene `nosniff`                 |
| `x_frame_options`         | warning  | `X-Frame-Options` o CSP `frame-ancestors` |
| `content_security_policy` | warning  | `Content-Security-Policy` presente        |
| `referrer_policy`         | info     | `Referrer-Policy` presente                |
| `permissions_policy`      | info     | `Permissions-Policy` presente             |

- **3 checks `compliance`**:

| check                      | severity | criterio                                             |
| -------------------------- | -------- | ---------------------------------------------------- |
| `privacy_policy_available` | warning  | link privacy/cookies/terms/gdpr o schema             |
| `cookie_consent_banner`    | warning  | marcadores CMP/banner en el HTML (`CONSENT_MARKERS`) |
| `data_protection_schema`   | info     | JSON-LD con `privacyPolicy` o `policies`             |

- `PageExtras` nuevos campos: `privacy_link_count` (enlaces cuyo href/texto contiene
  privacy/cookies/gdpr/terms/legal/datenschutz), `consent_banner`. Helper nuevo
  `json_ld_has_privacy` (recorre bloques JSON-LD).
- Tests: audit.rs pasa de 8→10 categorías; export `page_values` 33→35 columnas
  (SAMPLE_AUDIT ampliado con categorías security/compliance; `count_seo_rows` 4→6,
  xlsx processed 7→9); 2 tests nuevos en checks.rs (headers security + compliance).
  `cargo test --lib`: 128 OK. Clippy limpio.
- i18n: `DICT`/`WHY`/`CHECK_FIXES` en `src/lib/seo-checks.ts` (9 checks nuevos);
  labels `seo.category.{security,compliance}` en `messages/{en,es}.json` y en los
  maps de PageDetailPanel y SiteSeoPanel (regenerar paraglide con `bun run build`).

### Fase 6 — Evidence en JSON-LD y categorías skipped (F2.9 + F3.12) ✅ HECHO

- **F2.9**: `CheckResult.examples` poblado para JSON-LD (json_ld_valid, schema_completeness,
  faq_accordion_without_schema, author_schema, freshness_dates, data_protection_schema,
  privacy_policy_available): hasta 3 ejemplos con `{ block, type, evidence, snippet }`.
  Helpers en checks.rs (`json_ld_block_examples`, `json_ld_evidence_of`). Tests nuevos.
- **F3.12**: `CategoryResult.score: Option<f64>` (None = categoría skipped, ej. `links`
  en audit de página). `compute()` emite las 11 categorías de CATEGORY_ORDER (antes 10);
  weighted avg solo suma categorías con `score.is_some()`.
- Persistencia: `save_seo_normalized` omite categorías con score None
  (`seo_category_scores.score` es `REAL NOT NULL`); backfill migración
  `011_seo_overview.sql` filtra `json_extract(c.value,'$.score') IS NOT NULL`
  (espejado en test de `db/repos/mod.rs`).
- Export: `category_score()` con `.and_then(|c| c.score)`; `audit_row` XLSX escribe
  `""` para None; CSV ya mapeaba None→blank.
- Frontend: `types.ts` `SeoCategoryResult.score: number | null`; `seo-ui.ts`
  `seoCategoryGains` con type guard; `PageDetailPanel.svelte` muestra "N/A" +
  `seo-category-na` cuando null. `cargo test --lib`: 141 OK. Commit `9fdfad7`.

### Fase 7 — Site resources: robots.txt, sitemap.xml y hreflang (F4.13) ✅ HECHO

- **`seo/site.rs`** (nuevo): `SiteResource {status, body}`, `SiteResources {robots_txt,
sitemap_xml}`, `origin_of()`, `fetch_site_resources(client, page_url)` cacheado por
  origin (OnceLock<Mutex<HashMap>>, lifetime del proceso), `is_valid_sitemap()`
  (quick_xml: root `urlset`/`sitemapindex`, todas las etiquetas cerradas, EOF sin errores).
- **`seo/audit.rs`**: `AuditContext` gana `site_resources: Option<Arc<SiteResources>>` y
  `audit_page_with_site(seo, html, ctx, client)` async que fetches los recursos (cacheados)
  y delega en `audit_page`. Call sites: `crawler/engine.rs` (`fetch_and_parse` con param
  `site_client: Option<&reqwest::Client>`), `features/results/commands.rs`,
  `features/seo/commands.rs` (single + bulk con `site_client` clonado).
- **3 checks nuevos en `run_all`** (technical):

| check                     | severity | criterio                                                                   |
| ------------------------- | -------- | -------------------------------------------------------------------------- |
| `robots_txt_exists`       | warning  | robots.txt con status 200 y body no vacío (skip sin resources)             |
| `sitemap_xml_valid`       | warning  | sitemap status 200 + `is_valid_sitemap(body)` (skip sin resources)         |
| `hreflang_self_reference` | warning  | si hay hreflang_links, alguno apunta a la propia URL (per-page, sin fetch) |

- Tests: 5 nuevos en checks.rs (pass/fail/skip de site resources + hreflang pass/fail/skip),
  1 corregido en site.rs (well-formedness completa). `cargo test --lib`: 148 OK.
  Clippy/fmt limpios, svelte-check 0, eslint 0, build OK. i18n en seo-checks.ts
  (DICT/WHY/CHECK_FIXES). Commit `21484b0`.

### Verificación

- `cargo test --lib` en `src-tauri/` (tests unit de parser/checks/audit/score/export)
- `bun run check`, `bun run lint`, `bun run build` en la raíz

### No implementar (tendencias 2026)

`llms.txt` (Google no le da trato especial), chunking/rewriting para IA, entity
coverage/NLP, checks que requieran navegador (contraste de color, INP/CLS reales).

## Trabajo en curso: optimización para sitios grandes

Objetivo: renderizado parcial + relleno en segundo plano para sitios con
decenas de miles de páginas. Orden de ejecución: 1→2→4→3→5.

### Fase 1 — Proyección ligera del listado ✅ HECHO

`db/repos/results.rs`: `row_to_result_light` (índices 0-23, sin blobs HTML/JSON)
para `get_results`; `get_page_detail` intacto (full). Test
`test_list_projection_drops_heavy_json_blobs_but_detail_keeps_them`.

### Fase 2 — Agregaciones SQL y comparador paginado ✅ HECHO

- `external_domains` reescrito con agregación SQL (`links.rs`).
- `get_duplicate_groups_page` (`analytics.rs`, agregación SQL + fetch por `IN`) +
  comando `get_duplicate_groups_page`; fix `params_from_iter` con `Vec<Box<dyn ToSql>>`.
- **`compare_crawl_snapshots_page`** (snapshots.rs): diff en SQL puro (new/removed
  con `NOT EXISTS`, changed con JOIN + comparación `ROUND(...,1)`/`IS NOT`),
  `unchanged_count` en SQL; argumentos antes=snapshot_a, después=snapshot_b.
  Modelo `ComparePageResult`, comando `compare_crawls_page` (lib.rs ~232), test
  `test_compare_crawl_snapshots_page_sections` (en `/b` hay que borrarlo de
  `crawled_pages` para que aparezca como removed).
- Frontend `Comparador.svelte`: carga página 1 de las 3 secciones en paralelo
  (`PAGE_SIZE=100`), estado por sección, botón show-more con `compareCrawlsPage`.
  `compareCrawls`/`CompareResult` se mantienen exportados sin uso.
- `orphan_pages`/`dead_end_pages` con `limit=10` en `get_link_analysis`.

### Fase 4 — Índices y page_keywords materializado ✅ HECHO

- Migración `016_perf_indexes.sql` (`migrations/mod.rs`): índices
  `idx_pages_project_{seo_score,load_ms,size_bytes,duplicate_group,title}`,
  `idx_links_project_internal_from`, tabla `page_keywords(project_id, page_id,
keyword, count)` + backfill desde `keywords_json` vía `json_each` (una sola vez).
- Helpers `delete_page_keywords`/`save_page_keywords` (`crawl.rs`), llamados en
  `save_result` y `save_results_batch`; limpieza en `delete_project` y transfer.
- `get_keywords` (analytics.rs) con agregación SQL (`SUM(count)`,
  `COUNT(DISTINCT page_id)`); `duplicate_title` (results.rs) con `EXISTS`.
- `transfer/package.rs`: `copy_page_keywords` + DELETE destino + seed + `counts()` 9 tuplas.
- Tests: `test_page_keywords_materialized_and_aggregated`,
  `test_duplicate_title_filter_uses_existence`,
  `test_page_keywords_backfill_from_keywords_json`, verificación índices 016.

### Fase 3 — SiteTree streaming ✅ HECHO

- **`models/crawl_result.rs`**: `SiteTreeStreamNode` (url, title, status_code,
  depth, issue_count; plano, sin children). Reexportado en `models/mod.rs`.
- **`db/repos/analytics.rs`**: `get_site_tree_pages(project_id, after_url, limit)`
  → `(Vec<SiteTreeStreamNode>, total)`: keyset pagination sobre `url` (usa índice
  `idx_pages_project_url`), `GROUP BY url` + LEFT JOIN agregado de `page_issues`
  para `issue_count`; `COUNT(DISTINCT url)` para total. Test
  `test_site_tree_pages_stream_batches`.
- **`features/results/commands.rs`**: `get_site_tree_stream(app, state, project_id,
batch_size=500)` emite eventos `site-tree-batch` `{project_id, nodes, total}`
  por lote (clones `pid`/`cursor` por iteración para el closure `move`) y devuelve
  el total. Registrado en `lib.rs` tras `get_site_tree_full`. `get_site_tree_full`
  intacto (sigue funcionando como fallback).
- **Frontend**: `SiteTreeStreamNode`/`SiteTreeBatch` en `types.ts`; `getSiteTreeStream`
  en `api/results.ts`. `SiteTree.svelte` consume el stream: acumula nodos planos,
  `buildTree` (ya no aplana children, agrupa por ruta) con rebuild debounced (40ms),
  skeleton solo al inicio, footer `tree-streaming` con `{count} of {total}` (nueva
  clave `tree.streaming` en `messages/{en,es}.json`). Cleanup de listener en el
  `$effect` (unlisten + `treeSeq++`).
- Verificación: `cargo test --lib` 160 OK, clippy limpio, svelte-check 0,
  eslint 0, `bun run build` OK. Regenerar paraglide con
  `npx @inlang/paraglide-js compile --project ./project.inlang --outdir ./src/lib/paraglide`
  o `bun run build`.

### Fase 5 — Show-more paginado + skeletons preservando contenido ✅ HECHO

Objetivo: evitar cargar listas completas en sitios con decenas de miles de
páginas; cargar la página 1 y rellenar bajo demanda con botón "show more",
manteniendo el contenido ya renderizado mientras llegan más datos.

- **Backend `db/repos/links.rs`**: `orphan_pages_page`, `dead_end_pages_page`,
  `top_anchors_page`, `external_domains_page` (públicas, `page` 1-based +
  `page_size`, retornan `(items, total)`). `get_link_analysis` usa page 1 con
  límites 10/10/20/50 como antes. SQL de external domains extraído a
  `external_domain_rows_sql()`.
- **Backend `db/repos/analytics.rs`**: `get_keywords_page(project_id, page,
page_size)` → `(rows, total)` con `COUNT(DISTINCT keyword)`; `get_keywords`
  delega en page 1.
- **Comandos nuevos** en `features/links/commands.rs` y
  `features/analytics/commands.rs` (`get_orphan_pages_page`,
  `get_dead_end_pages_page`, `get_top_anchors_page`, `get_external_domains_page`,
  `get_project_keywords_page`), registrados en `lib.rs`.
- **Frontend**:
  - `api/links.ts`: `getOrphanPagesPage`, `getDeadEndPagesPage`,
    `getTopAnchorsPage`, `getExternalDomainsPage` (tupla `[items, total]`).
  - `api/analytics.ts`: `getDuplicateGroupsPage`, `getProjectKeywordsPage`.
  - `Duplicates.svelte`: página 1 (`PAGE_SIZE=25`) + `showMore()` acumulando
    (dedupe por `group.id`), badge con `total`, skeleton solo si no hay datos,
    botón show-more con `comparator.show_more`.
  - `Keywords.svelte`: igual con `PAGE_SIZE=100` (dedupe por keyword).
  - `LinkAnalysisPanel.svelte`: estado `MoreState<T>` por sección (orphans,
    dead ends, top anchors, external domains); la página 1 viene del
    `LinkAnalysis` y `showMore()` fetches páginas siguientes (lotes 10/10/20/50).
    `hasMore(s, initialFull)` oculta el botón cuando el total inicial es
    conocido o la página inicial no está completa (anchors/domains).
- Tests: `test_paginated_link_lists` (links.rs) y `test_keywords_page_pagination`
  (mod.rs). `cargo test --lib` **162 OK**, clippy limpio, svelte-check 0,
  eslint 0, `bun run build` OK.

### Verificación

- `cargo test --lib` en `src-tauri/`
- `bun run check`, `bun run lint`, `bun run build` en la raíz

## Trabajo en curso: lanzamiento v1.0.0 ✅ PREPARADO

Preparación de release 1.0.0 (Desktop + Android, con auto-updater). El trabajo
anterior de SEO y optimización ya está commiteado (`b3c28dd`, `3147f58`).

### Fase 0 — CI y advisories ✅ HECHO

- `cargo fmt` + `prettier --write` (AGENTS.md, api/results.ts, api/snapshots.ts,
  Comparador.svelte, LinkAnalysisPanel.svelte, i18n-issues.ts, seo-checks.ts,
  seo-ui.ts).
- `lru` 0.12.5 → **0.18.2** (Cargo.toml `lru = "0.18"`), resuelve advisory low.
- `glib` 0.18.5 (RUSTSEC-2024-0429, medium) queda como **warning permitido**:
  transitiva de `gtk 0.18` (GTK3) fijado por Tauri 2.11.5; no hay fix sin GTK4.
  `cargo audit` exit 0 ("18 allowed warnings"). `cargo test --lib` 162 OK.

### Fase 1 — Versión ✅ HECHO

- `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml`, `package.json`: **1.0.0**.

### Fase 2 — Auto-updater ✅ HECHO

- `tauri-plugin-updater = "2"` + `tauri-plugin-process = "2"` (Cargo.toml) y
  registrados en `lib.rs`. `tauri.conf.json` `plugins.updater` (pubkey de
  `.secrets/tauri.key.pub`, endpoint `.../releases/latest/download/latest.json`).
  `capabilities/default.json`: `updater:default` + `process:default`.
- Frontend: `@tauri-apps/plugin-updater@2.10.1` + `@tauri-apps/plugin-process@2.3.1`.
  `src/lib/updater.ts` (`checkForUpdates`, `downloadAndInstall` con progreso,
  `relaunchApp`). `SettingsModal.svelte` con UI de update (checking/available/
  installing %/restart). Check automático en el launcher (`app.svelte.ts`
  `autoCheckUpdates`, una vez por sesión, silencioso). i18n `settings.update` +
  `updater.*` en `messages/{en,es}.json`.
- `tauri-action` genera `latest.json`/`.sig` automáticamente (secrets
  `TAURI_SIGNING_PRIVATE_KEY*` ya configurados).
- **IMPORTANTE**: el bundle necesita `"createUpdaterArtifacts": true` en
  `tauri.conf.json` para que Tauri genere los `.sig`; sin él, tauri-action no
  sube `latest.json` ("Signature not found for the updater JSON").

### Fase 3 — Docs y licencia ✅ HECHO

- `LICENSE` (MIT, Irving Frias 2026), `CHANGELOG.md` (entrada 1.0.0 + 0.3.0),
  `CONTRIBUTING.md`, README actualizado (33 checks semánticos, 97 SEO checks en
  10 categorías, arquitectura con seo/links/schedule/site-map/transfer, sin iOS,
  screenshots placeholder en `docs/screenshots/`). Tag local huérfano `main`
  borrado.

### Fase 4 — Keystore Android de producción ✅ HECHO

- Keystore nuevo en `android/opencrawler.keystore` (alias `opencrawler-prod`,
  password generado aleatorio, validez 10000 días); dev respaldado en
  `android/opencrawler.keystore.dev.bak`. Secrets `ANDROID_*` actualizados vía
  `gh secret set` (base64 + alias + passwords). `ANDROID_SETUP.md` actualizado.

### Fase 5 — Commit y release ✅ HECHO

- Commit de todo el trabajo 1.0, push, `git tag v1.0.0`, push del tag.
- release.yml dispara con tags `v*`: macOS ARM/Intel + Linux + Windows + Android
  (APK/AAB firmados) + latest.json para el updater. `update-readme` regenera la
  tabla de descargas.
- **Incidente resuelto**: la primera release v1.0.0 se publicó sin `latest.json`
  (updater roto, 404 en el endpoint). Causa: faltaba `"createUpdaterArtifacts": true`
  en el bundle de `tauri.conf.json` (issue tauri-action #1098). Fix commit `c38e1d5`;
  se borró tag+release v1.0.0 y se re-pusheó el tag sobre el commit fijado.
  Run `31925399852` success; release re-publicada con 19 assets (latest.json +
  .sig por plataforma) y endpoint `.../latest/download/latest.json` → 200.
- Verificación local del fix: `bun run tauri bundle --target aarch64-apple-darwin
  --bundles app,dmg` con `TAURI_SIGNING_PRIVATE_KEY` (de `.secrets/tauri.key`) y
  password `opencrawler2026` genera `Open Crawler.app.tar.gz.sig`.

**Guardar el password del keystore Android** (`bx067ADsyJeDRluvPazxW6aH9AHXpDkK`)
— es el único respaldo; si se pierde no se pueden publicar actualizaciones del
APK. Guardar también `android/opencrawler.keystore` en un gestor seguro.
