// Mide el coste JS de los hot paths de la tabla de resultados (F3, T10):
// parseIssues (frío y caliente), getIssueCounts, variantes de score y el
// mapeo completo de 50 filas (lo que hace el `$derived rows` de
// ResultsTable.svelte en cada filtro/paginación). Importa los módulos reales;
// bun ejecuta TS directamente. Uso:
//   bun scripts/perf/measure-table-js.ts
// Imprime JSON con milisegundos (mediana de 21 corridas por caso).

import { parseIssues, getIssueCounts } from '#lib/features/results/issue-cache.ts';
import { seoVariant, seoGrade } from '#lib/seo-ui.ts';
// readabilityVariant/readabilityLabel son funciones locales triviales de
// ResultsTable.svelte (umbrales); no se importan, su coste es despreciable.

const SEVERITIES = ['error', 'warning', 'info'] as const;
const TYPES = [
  'missing_title',
  'missing_h1',
  'img_no_alt',
  'missing_meta_description',
  'heading_skip',
];

function issueJson(n: number, nonce: number): string {
  const issues = [];
  for (let i = 0; i < n; i++) {
    issues.push({
      issue_type: TYPES[(i + nonce) % TYPES.length],
      severity: SEVERITIES[(i + nonce) % 3],
      message: `Issue ${i} en la página ${nonce}`,
      xpath: `/html/body/main/div[${i + 1}]`,
      element: '<div>',
      snippet: 'texto de ejemplo',
    });
  }
  return JSON.stringify(issues);
}

function median(v: number[]): number {
  const s = [...v].sort((a, b) => a - b);
  return s[Math.floor(s.length / 2)];
}

function timeIt(fn: () => void, runs = 21): number {
  const samples: number[] = [];
  for (let r = 0; r < runs; r++) {
    const t0 = performance.now();
    fn();
    samples.push(performance.now() - t0);
  }
  return Math.round(median(samples) * 1000) / 1000;
}

// 50 filas como una página de la tabla, con 8 issues cada una (realista).
const rows = Array.from({ length: 50 }, (_, i) => ({
  json: issueJson(8, i),
  seo: 40 + ((i * 7) % 60),
  readability: 30 + ((i * 11) % 65),
}));

const parseCold = timeIt(() => {
  for (let i = 0; i < 50; i++) parseIssues(issueJson(8, 1000 + i));
});
const parseWarm = timeIt(() => {
  for (const r of rows) parseIssues(r.json);
});
const parsed = rows.map((r) => parseIssues(r.json));
const counts = timeIt(() => {
  for (const p of parsed) getIssueCounts(p);
});
const variants = timeIt(() => {
  for (const r of rows) {
    seoVariant(r.seo);
    seoGrade(r.seo);
  }
});
const rowMapping = timeIt(() => {
  for (const r of rows) {
    const issues = parseIssues(r.json);
    getIssueCounts(issues);
    seoVariant(r.seo);
    seoGrade(r.seo);
  }
});

console.log(
  JSON.stringify({
    rows: 50,
    issuesPerRow: 8,
    ms: {
      parseIssues_cold_50rows: parseCold,
      parseIssues_warm_50rows: parseWarm,
      getIssueCounts_50rows: counts,
      scoreVariants_50rows: variants,
      rowMapping_50rows: rowMapping,
    },
  })
);
