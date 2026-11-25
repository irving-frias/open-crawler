import { describe, expect, it } from 'vitest';
import { isLazyTab, TAB_DEFS, TAB_IDS } from './tabs';

describe('tabs', () => {
  it('keeps TAB_DEFS in sync with TAB_IDS and without duplicates', () => {
    expect(TAB_DEFS.map((t) => t.id)).toEqual([...TAB_IDS]);
    expect(new Set(TAB_IDS).size).toBe(TAB_IDS.length);
  });

  it('loads every tab lazily except the results table', () => {
    expect(isLazyTab('results')).toBe(false);
    for (const id of TAB_IDS.filter((id) => id !== 'results')) {
      expect(isLazyTab(id)).toBe(true);
    }
  });
});
