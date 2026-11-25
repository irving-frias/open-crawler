import { describe, expect, it } from 'vitest';
import { seoCategoryGains, seoGrade, seoScoreColor, seoVariant } from './seo-ui';
import type { SeoCategoryResult } from '#lib/api/types.ts';

function cat(category: string, score: number | null, weight: number): SeoCategoryResult {
  return {
    category,
    score,
    weight,
    passed_weight: 0,
    total_weight: weight,
    passed_checks: 0,
    total_checks: 0,
  };
}

describe('seoScoreColor', () => {
  it('maps score bands to CSS variables', () => {
    expect(seoScoreColor(100)).toBe('var(--success)');
    expect(seoScoreColor(80)).toBe('var(--success)');
    expect(seoScoreColor(79.9)).toBe('var(--warning)');
    expect(seoScoreColor(60)).toBe('var(--warning)');
    expect(seoScoreColor(59.9)).toBe('var(--danger)');
    expect(seoScoreColor(0)).toBe('var(--danger)');
  });
});

describe('seoGrade', () => {
  it('maps score bands to letter grades at the boundaries', () => {
    expect(seoGrade(90)).toBe('A');
    expect(seoGrade(89.9)).toBe('B');
    expect(seoGrade(80)).toBe('B');
    expect(seoGrade(70)).toBe('C');
    expect(seoGrade(60)).toBe('D');
    expect(seoGrade(59.9)).toBe('F');
  });
});

describe('seoVariant', () => {
  it('maps score bands to Badge variants', () => {
    expect(seoVariant(80)).toBe('default');
    expect(seoVariant(60)).toBe('warning');
    expect(seoVariant(59.9)).toBe('destructive');
  });
});

describe('seoCategoryGains', () => {
  it('returns the weighted gain for every imperfect category', () => {
    const gains = seoCategoryGains([cat('meta', 50, 0.25), cat('technical', 100, 0.25)]);
    // present weight = 0.5 → (100 - 50) * 0.25 / 0.5 = 25
    expect(gains.meta).toBeCloseTo(25, 6);
    // A perfect category has nothing to gain.
    expect(gains.technical).toBeUndefined();
  });

  it('skips categories without score (skipped audits) and still normalizes', () => {
    const gains = seoCategoryGains([cat('meta', 40, 0.25), cat('links', null, 0.12)]);
    // Only `meta` has a score → present weight = 0.25 → full deficit = 60.
    expect(gains.meta).toBeCloseTo(60, 6);
    expect(gains.links).toBeUndefined();
  });

  it('excludes skipped categories from the weight denominator', () => {
    const withSkipped = seoCategoryGains([cat('meta', 0, 0.25), cat('links', null, 0.75)]);
    const without = seoCategoryGains([cat('meta', 0, 0.25)]);
    expect(withSkipped.meta).toBeCloseTo(without.meta, 6);
  });

  it('returns an empty object when nothing is scored', () => {
    expect(seoCategoryGains([cat('links', null, 0.12)])).toEqual({});
    expect(seoCategoryGains([])).toEqual({});
  });
});
