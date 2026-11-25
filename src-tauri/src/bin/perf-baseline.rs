//! `perf-baseline`: mide las consultas calientes de la UI contra una base
//! ya crawleada y escribe los tiempos (mediana de N corridas) como JSON.
//!
//! Uso:
//!   perf-baseline --db <ruta.db> [--project <id>] [--runs 3]
//!   perf-baseline --db <ruta.db> [--project <id>] --clusters
//!
//! La base se abre sin migraciones y sin writes (solo SELECT). Sin caché de
//! resultados: se mide el SQL crudo, no el cache. El JSON incluye el número
//! de páginas del proyecto y el tamaño del archivo .db para contexto.
//!
//! `--clusters` mide `compute_seo_clusters` sobre una COPIA de la base
//! (la función borra y reescribe los clusters, así que nunca toca la
//! base original) e imprime el tiempo y el número de clusters.
//!
//! Herramienta de T10 (línea base de rendimiento, UI_UX_PLAN §3.6). Se
//! re-ejecuta tras T13/T14/T19 para comparar.

use std::path::PathBuf;
use std::time::Instant;

use open_crawler_lib::db::CrawlRepo;
use rusqlite::Connection;
use serde_json::json;

fn median(mut v: Vec<u128>) -> u128 {
    v.sort_unstable();
    v[v.len() / 2]
}

/// Ejecuta `f` N veces y devuelve la mediana en milisegundos.
fn time_it<F: FnMut() -> usize>(mut f: F, runs: usize) -> u128 {
    let mut samples = Vec::with_capacity(runs);
    for _ in 0..runs {
        let t0 = Instant::now();
        let _n = f();
        samples.push(t0.elapsed().as_millis());
    }
    median(samples)
}

fn main() {
    let mut db: Option<PathBuf> = None;
    let mut project: Option<String> = None;
    let mut runs: usize = 3;
    let mut clusters = false;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--db" => db = args.next().map(PathBuf::from),
            "--project" => project = args.next(),
            "--runs" => runs = args.next().and_then(|s| s.parse().ok()).unwrap_or(3),
            "--clusters" => clusters = true,
            _ => {}
        }
    }
    let db_path = db.expect("--db <ruta.db> es obligatorio");
    if clusters {
        // compute_seo_clusters escribe: medir sobre una copia.
        let copy_path = db_path.with_extension("clusters-copy.db");
        std::fs::copy(&db_path, &copy_path).expect("no se pudo copiar la base");
        let conn = Connection::open(&copy_path).expect("no se pudo abrir la copia");
        let project_id = match project {
            Some(p) => p,
            None => conn
                .query_row(
                    "SELECT id FROM projects ORDER BY created_at DESC LIMIT 1",
                    [],
                    |r| r.get::<_, String>(0),
                )
                .expect("la base no tiene proyectos"),
        };
        let repo = CrawlRepo::new(&conn, None);
        let t0 = Instant::now();
        let count = repo
            .compute_seo_clusters(&project_id)
            .expect("falló compute_seo_clusters");
        println!(
            "{}",
            json!({
                "clusters_secs": t0.elapsed().as_secs_f64(),
                "cluster_count": count,
                "db_copy": copy_path.to_string_lossy(),
            })
        );
        return;
    }
    // Apertura normal (no se emite ningún write: solo SELECT). Las
    // migraciones no corren: la base ya existe y viene de un crawl.
    let conn = Connection::open(&db_path).expect("no se pudo abrir la base");
    let project_id = match project {
        Some(p) => p,
        None => conn
            .query_row(
                "SELECT id FROM projects ORDER BY created_at DESC LIMIT 1",
                [],
                |r| r.get::<_, String>(0),
            )
            .expect("la base no tiene proyectos"),
    };
    let pages: u64 = conn
        .query_row(
            "SELECT COUNT(*) FROM crawled_pages WHERE project_id = ?1",
            rusqlite::params![project_id],
            |r| r.get::<_, i64>(0),
        )
        .unwrap_or(0) as u64;
    let db_bytes = std::fs::metadata(&db_path).map(|m| m.len()).unwrap_or(0);

    let repo = CrawlRepo::new(&conn, None);
    let page_id: Option<String> = conn
        .query_row(
            "SELECT id FROM crawled_pages WHERE project_id = ?1 LIMIT 1",
            rusqlite::params![project_id],
            |r| r.get(0),
        )
        .ok();
    let snapshots: Vec<String> = {
        let mut stmt = conn
            .prepare(
                "SELECT id FROM crawl_snapshots WHERE project_id = ?1 \
                 ORDER BY snapshot_time DESC LIMIT 2",
            )
            .expect("preparar snapshots");
        let rows = stmt
            .query_map(rusqlite::params![project_id], |r| r.get::<_, String>(0))
            .expect("consultar snapshots");
        rows.filter_map(|r| r.ok()).collect()
    };

    let results_p1 = time_it(
        || {
            repo.get_results(
                &project_id,
                1,
                50,
                None,
                None,
                None,
                None,
                None,
                None,
                false,
                false,
                false,
                false,
            )
            .map(|(rows, _)| rows.len())
            .unwrap_or(0)
        },
        runs,
    );
    let results_p100 = time_it(
        || {
            repo.get_results(
                &project_id,
                100,
                50,
                None,
                None,
                None,
                None,
                None,
                None,
                false,
                false,
                false,
                false,
            )
            .map(|(rows, _)| rows.len())
            .unwrap_or(0)
        },
        runs,
    );
    let results_search = time_it(
        || {
            repo.get_results(
                &project_id,
                1,
                50,
                None,
                Some("pagina"),
                None,
                None,
                None,
                None,
                false,
                false,
                false,
                false,
            )
            .map(|(rows, _)| rows.len())
            .unwrap_or(0)
        },
        runs,
    );
    let filter_options = time_it(
        || repo.get_filter_options(&project_id).map(|_| 1).unwrap_or(0),
        runs,
    );
    let seo_rows = time_it(
        || {
            repo.count_seo_rows(&project_id)
                .map(|n| n as usize)
                .unwrap_or(0)
        },
        runs,
    );
    let page_detail = match &page_id {
        Some(id) => {
            let id = id.clone();
            Some(time_it(
                || repo.get_page_detail(&id).map(|_| 1).unwrap_or(0),
                runs,
            ))
        }
        None => None,
    };
    let compare = if snapshots.len() == 2 {
        let (a, b) = (snapshots[0].clone(), snapshots[1].clone());
        Some(time_it(
            || {
                repo.compare_crawl_snapshots_page(&a, &b, "new", 1, 100)
                    .map(|_| 1)
                    .unwrap_or(0)
            },
            runs,
        ))
    } else {
        None
    };

    println!(
        "{}",
        json!({
            "db": db_path.to_string_lossy(),
            "project_id": project_id,
            "pages": pages,
            "db_bytes": db_bytes,
            "runs": runs,
            "ms": {
                "get_results_p1": results_p1,
                "get_results_p100": results_p100,
                "get_results_search": results_search,
                "get_filter_options": filter_options,
                "count_seo_rows": seo_rows,
                "get_page_detail": page_detail,
                "compare_page": compare,
            }
        })
    );
}
