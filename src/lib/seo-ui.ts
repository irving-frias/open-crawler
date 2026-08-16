import type { SeoCategoryResult } from '$lib/api/types';

/**
 * Shared presentation helpers for SEO scores. Centralizes the score → color /
 * grade / badge mapping that was previously duplicated in PageDetailPanel,
 * ResultsTable and SiteSeoPanel.
 */

export function seoScoreColor(score: number): string {
  if (score >= 80) return 'var(--success)';
  if (score >= 60) return 'var(--warning)';
  return 'var(--danger)';
}

export function seoGrade(score: number): string {
  if (score >= 90) return 'A';
  if (score >= 80) return 'B';
  if (score >= 70) return 'C';
  if (score >= 60) return 'D';
  return 'F';
}

export function seoVariant(score: number): 'default' | 'warning' | 'destructive' {
  if (score >= 80) return 'default';
  if (score >= 60) return 'warning';
  return 'destructive';
}

/**
 * Potential overall-score increase if a failing category were raised to 100.
 * Mirrors the backend weighted average: overall = Σ(score_c · weight_c) / Σ(weight_c)
 * over the categories with a score, so fixing every check of category `c`
 * moves the overall score by `(100 - score_c) · weight_c / present_weight`.
 * Skipped categories (score: null) are excluded.
 */
export function seoCategoryGains(categories: SeoCategoryResult[]): Record<string, number> {
  const scored = categories.filter(
    (c): c is SeoCategoryResult & { score: number } => c.score !== null
  );
  const presentWeight = scored.reduce((acc, c) => acc + c.weight, 0);
  if (presentWeight <= 0) return {};
  const gains: Record<string, number> = {};
  for (const cat of scored) {
    if (cat.score >= 100) continue;
    const gain = ((100 - cat.score) * cat.weight) / presentWeight;
    gains[cat.category] = gain;
  }
  return gains;
}
