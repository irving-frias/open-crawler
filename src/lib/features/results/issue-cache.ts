// Shared, memoized parsing of the backend `semantic_issues_json` payloads.
//
// Both the results table (per-row counts + expandable detail) and the filter
// bar (available severities / issue types) parse the exact same JSON strings on
// every refresh and every keystroke. The backend stores the serialized payload
// verbatim, so identical strings produce identical arrays; memoizing by string
// avoids re-running JSON.parse hundreds of times per render.
//
// Svelte 5 discourages module-level reactive state, but this is a pure
// memoization function with its own cache, not shared reactive state.
const MAX_CACHE_SIZE = 5000;
const EVICT_BATCH = 250;
const cache = new Map<string, any[]>();

export function parseIssues(issuesJson: string | null | undefined): any[] {
  if (!issuesJson) return [];
  const hit = cache.get(issuesJson);
  if (hit) return hit;
  let parsed: any[];
  try {
    parsed = JSON.parse(issuesJson);
    if (!Array.isArray(parsed)) parsed = [];
  } catch {
    parsed = [];
  }
  if (cache.size >= MAX_CACHE_SIZE) {
    for (const oldest of cache.keys()) {
      cache.delete(oldest);
      if (cache.size < MAX_CACHE_SIZE - EVICT_BATCH) break;
    }
  }
  cache.set(issuesJson, parsed);
  return parsed;
}

export function getIssueCounts(issues: any[]): { errors: number; warnings: number; infos: number } {
  let errors = 0;
  let warnings = 0;
  let infos = 0;
  for (const issue of issues) {
    if (issue.severity === 'error') errors++;
    else if (issue.severity === 'warning') warnings++;
    else infos++;
  }
  return { errors, warnings, infos };
}
