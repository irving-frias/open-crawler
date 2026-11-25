# Línea base de rendimiento (T10)

Medición inicial, 2026-11-24. Entorno: VM de trabajo (4 CPU, 8 GB RAM, disco
local), binario release `open-crawler-mcp`, sitios de prueba locales
(generador en `scripts/perf/gen-site.py`, 10k y 100k páginas, servidos con
`python3 -m http.server`). Repetir tras T13/T14/T19 con los mismos comandos.

Objetivo (UI_UX_PLAN §3.6): con 100k URLs, la UI responde en menos de 100 ms
por interacción y el scroll no baja de 60 fps.

## Cómo reproducir

```sh
# 1. Generar el sitio y servirlo
python3 scripts/perf/gen-site.py /tmp/perf-site-10k 10000 8899
cd /tmp/perf-site-10k && python3 -m http.server 8899 --bind 127.0.0.1 &
# 2. Crawlear (config en /tmp: seed_urls, max_depth 30, delay 0)
open-crawler-mcp crawl --config crawl-10k.json --db perf-10k.db
# 3. Consultas calientes (mediana de 3 corridas)
perf-baseline --db perf-10k.db
# 4. Coste JS de la tabla
bun scripts/perf/measure-table-js.ts
```

## B1/B2. Crawl y base de datos

|                                    | 10k URLs                               | 100k URLs                                       |
| ---------------------------------- | -------------------------------------- | ----------------------------------------------- |
| Páginas crawleadas                 | 10 002 (10 000 + índice + 1 404)       | 100 002 (100 000 + índice + 1 404)              |
| Tiempo total (crawl)               | 64,8 s                                 | 2059,9 s (34,3 min)                             |
| Páginas/s                          | ~154                                   | ~48,6                                           |
| Tiempo total del comando           | 327 s (crawl + snapshot + clusters)    | no aplica: proceso matado tras "CRAWL COMPLETE" |
| Tamaño .db                         | 664 498 176 B (~634 MB, ~66 KB/página) | 6 438 391 808 B (~6,0 GiB, ~64 KB/página)       |
| Post-crawl: `compute_seo_clusters` | ~247 s (1 CPU al 83%)                  | 138,6 s con T13 (antes: ~7 h extrapoladas)      |

Hallazgo: tras el crawl, `compute_seo_clusters` (Jaccard cuadrático, T13)
domina el tiempo total a 10k páginas (247 s de 327 s). El sitio sintético es
el peor caso (todas las páginas comparten el mismo vocabulario de ~40
palabras → un solo bucket gigante, 1 cluster con 10 002 páginas), pero el
comportamiento cuadrático es real y es el objetivo de T13. A 100k el
clustering con el código viejo no se deja terminar en la sesión
(extrapolación ~7 h); se mide el crawl y las consultas matando el proceso
tras el "CRAWL COMPLETE".

Interino 100k (19:15 CST): 30 095 páginas en ~13 min (~38 págs/s, frente a
154 págs/s a 10k) — el rendimiento cae a medida que crece la base (1,9 GB).
La caída es en sí un dato para T14.

Cierre 100k (19:42 CST): el crawl terminó con 100 002 páginas, 0 errores,
en 2059,9 s (~48,6 págs/s). El proceso se mató tras el "CRAWL COMPLETE"
antes de que arrancara el clustering viejo; el snapshot no llegó a crearse
(la tarea es asíncrona y murió con el proceso), así que esta base tiene 0
snapshots y `compare_page` queda sin dato también a 100k. La base creció a
6,0 GiB; el WAL del proceso matado (68 MB) se dejó tal cual.

T13 (medido 2026-11-24 con `perf-baseline --clusters`, sobre copia de la
base): a 10k, 247 s → **8,5 s**; a 100k, ~7 h extrapoladas → **138,6 s**.
En ambos casos el resultado es 0 clusters: con el límite de 400 páginas por
bucket, todos los buckets del sitio sintético se saltan porque su
vocabulario es genérico (compartido por todas las páginas). El mega-cluster
de 10 002 páginas del código viejo no era señal, era ruido; en un sitio
real los clusters se forman por keywords específicas (buckets pequeños) y
los tests de T13 verifican que eso sigue funcionando.

## B3. Consultas calientes (ms, mediana de 3)

| Consulta                        | 10k                        | 100k                   |
| ------------------------------- | -------------------------- | ---------------------- |
| `get_results` pág. 1 (50)       | 1 ms                       | 138 ms *               |
| `get_results` pág. 100 (50)     | 1 ms                       | 339 ms *               |
| `get_results` búsqueda "pagina" | 41 ms                      | 145 255 ms *           |
| `get_filter_options`            | 33 ms                      | 29 288 ms *            |
| `count_seo_rows`                | 146 ms                     | 151 973 ms *           |
| `get_page_detail`               | < 1 ms                     | 11 ms *                |
| `compare_page` (2 snapshots)    | sin dato (1 solo snapshot) | sin dato (0 snapshots) |

\* 100k medido con 1 sola corrida (no mediana de 3): la mediana habría
tardado ~45 min por el I/O. Los números incluyen el coste de leer en frío
una base de 6,0 GiB a ~27 MB/s en este entorno (la caché del SO no retiene
el archivo entero). La lectura correcta es el orden de magnitud, no el
milisegundo: a 100k, la búsqueda de texto (145 s), `count_seo_rows` (152 s)
y las opciones de filtro (29 s) hacen un escaneo completo de la tabla y sus
JSON por llamada. Incluso el listado paginado (138/339 ms) supera el
presupuesto de 100 ms en frío. Es el mandato de T14: estas consultas
necesitan agregados materializados o índices, no más escaneos.

Lectura a 10k: el listado paginado es instantáneo (1 ms, proyección ligera

- índices), pero `count_seo_rows` (146 ms, agregación `json_each` sobre todo
  el `seo_audit_json`) ya supera el presupuesto de 100 ms por interacción: es
  el objetivo de T14. La búsqueda de texto (41 ms) y las opciones de filtro
  (33 ms) son el siguiente coste.

## F1. Bundle de producción

Medido con `bun run build` (2026-11-24):

|                | Bytes                         |
| -------------- | ----------------------------- |
| Build total    | 1 529 692 (~1,46 MB)          |
| JS total       | 1 371 850 (~1,31 MB)          |
| CSS total      | 149 722 (~146 KB)             |
| Chunk JS mayor | 333 KB (`chunks/DcWtQeLG.js`) |
| Archivos       | 95                            |

## F2. Tiempo a interactivo

No medible en este entorno: el Chromium del sandbox está bloqueado por
política (solo lo maneja el runtime) y webkit2gtk no expone CDP. Para medirlo
en la máquina objetivo: abrir la app, DevTools → Performance, recargar con
caché vacía y anotar LCP + "tiempo a interactivo" (long tasks < 50 ms).
Criterio del plan (§3.4): por debajo de 2 s.

## F3. Coste JS de la tabla (bun, módulos reales, 50 filas × 8 issues)

Mediana de 21 corridas, 2026-11-24:

| Operación                         | ms    |
| --------------------------------- | ----- |
| `parseIssues` frío (50 filas)     | 0,262 |
| `parseIssues` caliente (50 filas) | 0,007 |
| `getIssueCounts` (50 filas)       | 0,008 |
| Variantes de score (50 filas)     | 0,026 |
| Mapeo completo de filas (50)      | 0,014 |

Conclusión: los hot paths JS de la tabla son despreciables (< 0,3 ms por
página de 50 filas); la memoización de `issue-cache.ts` funciona. El coste
por interacción lo dominan las consultas del backend (B3) y el render DOM.
El FPS de scroll en navegador real se mide con el profiler en la máquina
objetivo (DevTools → Performance, scroll de 10 s en la tabla de resultados,
anotar FPS medio y mínimo).

## T14. Después de la auditoría + índices (100k, 1 corrida, en frío)

Migración 021 (`project_seo_counts` materializado + 3 índices de cobertura)
y `INDEXED BY idx_pages_search` en `get_results` con búsqueda. Medido sobre
copia de la base 100k con la migración aplicada (la migración tarda 469 s
en esta base: backfill de los contadores + 3 índices; coste único).

| Consulta                        | Antes (100k) | Después (100k)                                                                                                                                           |
| ------------------------------- | ------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `count_seo_rows`                | 151 973 ms   | **7 ms** (lectura de 1 fila; backfill verificado: 1 100 022 + 4 721 058 + 1 200 024 = 7 021 104)                                                         |
| `get_results` búsqueda "pagina" | 145 255 ms   | **328 ms** (COUNT desde el índice; la página con `INDEXED BY`, antes 126 s solo la página)                                                               |
| `get_filter_options`            | 29 288 ms    | 18 831 ms (el DISTINCT de estados ya es rápido; quedan los DISTINCT de severidad/tipo sobre el índice de 4,7 M de `page_issues`, ~10 s cada uno en frío) |
| `get_results` p1 / p100         | 138 / 339 ms | 147 / 365 ms (sin cambio, dentro del ruido)                                                                                                              |

Residual T14: `get_filter_options` sigue por encima del presupuesto en
frío por los escaneos del índice de `page_issues`; hacerlos instantáneos
exigiría materializar los valores distintos por proyecto (misma técnica
que los contadores SEO). En caliente / SSD local son < 0,5 s.
