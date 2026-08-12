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

### Verificación

- `cargo test --lib` en `src-tauri/` (tests unit de parser/checks/audit/score/export)
- `bun run check`, `bun run lint`, `bun run build` en la raíz

### No implementar (tendencias 2026)

`llms.txt` (Google no le da trato especial), chunking/rewriting para IA, entity
coverage/NLP, checks que requieran navegador (contraste de color, INP/CLS reales).
