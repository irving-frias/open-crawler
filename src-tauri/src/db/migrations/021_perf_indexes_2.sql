-- T14: contadores SEO materializados por proyecto + índices de cobertura
-- para las consultas pesadas restantes (medidas en docs/PERF_BASELINE.md).
--
-- `count_seo_rows` sumaba 3 agregados sobre millones de filas
-- (seo_category_scores 1,1 M, seo_check_issues 4,7 M a 100k, más un SUM
-- sobre crawled_pages que leía páginas de overflow por fila). Con los
-- contadores, la lectura es una sola fila; los contadores se mantienen en
-- save_seo_normalized_batch / delete_seo_normalized, en la misma
-- transacción que los datos que cuentan.
CREATE TABLE IF NOT EXISTS project_seo_counts (
    project_id       TEXT NOT NULL PRIMARY KEY,
    category_count   INTEGER NOT NULL DEFAULT 0,
    check_issue_count INTEGER NOT NULL DEFAULT 0,
    priority_fix_total INTEGER NOT NULL DEFAULT 0
);

-- Backfill desde los datos ya normalizados (coste único en la actualización).
INSERT OR REPLACE INTO project_seo_counts
    (project_id, category_count, check_issue_count, priority_fix_total)
SELECT
    p.id,
    (SELECT COUNT(*) FROM seo_category_scores s WHERE s.project_id = p.id),
    (SELECT COUNT(*) FROM seo_check_issues i WHERE i.project_id = p.id),
    (SELECT COALESCE(SUM(c.seo_priority_fix_count), 0)
     FROM crawled_pages c WHERE c.project_id = p.id)
FROM projects p;

-- SUM(seo_priority_fix_count) sin tocar la tabla: la columna vive al final
-- del registro (ALTER en la 011) y en filas de ~66 KB cae en overflow.
CREATE INDEX IF NOT EXISTS idx_pages_project_priority_fix
    ON crawled_pages(project_id, seo_priority_fix_count);

-- DISTINCT status_code de get_filter_options sin lecturas a la tabla.
CREATE INDEX IF NOT EXISTS idx_pages_project_status
    ON crawled_pages(project_id, status_code);

-- Búsqueda de texto (url/title/h1 LIKE '%...%') + ORDER BY crawl_timestamp:
-- el COUNT se resuelve desde el índice solo; la página lleva
-- `INDEXED BY idx_pages_search` en get_results porque el planificador
-- prefiere idx_pages_project_ts (evita el sort) y acaba leyendo la tabla
-- por fila para evaluar los LIKE (126 s a 100k); con el índice forzado,
-- la página baja a ~3,5 s en frío (escaneo del índice + 50 lecturas).
-- crawl_timestamp va segundo para que el índice también sirva el ORDER BY
-- sin temp b-tree.
CREATE INDEX IF NOT EXISTS idx_pages_search
    ON crawled_pages(project_id, crawl_timestamp, url, title, h1);
