export const TAB_IDS = [
  'results',
  'overview',
  'dashboard',
  'site_tree',
  'comparator',
  'duplicates',
  'keywords',
  'schedule',
  'seo',
] as const;

export type TabId = (typeof TAB_IDS)[number];
export type LazyTabId = Exclude<TabId, 'results'>;

export interface TabDef {
  id: TabId;
}

export const TAB_DEFS: TabDef[] = [
  { id: 'results' },
  { id: 'overview' },
  { id: 'dashboard' },
  { id: 'site_tree' },
  { id: 'comparator' },
  { id: 'duplicates' },
  { id: 'keywords' },
  { id: 'schedule' },
  { id: 'seo' },
];

type AnyComponent = any;

const loaders: Record<LazyTabId, () => Promise<{ default: AnyComponent }>> = {
  overview: () => import('$lib/features/dashboard/Dashboard.svelte'),
  dashboard: () => import('$lib/features/semantic/SemanticDashboard.svelte'),
  site_tree: () => import('$lib/features/site-tree/SiteTree.svelte'),
  comparator: () => import('$lib/features/comparator/Comparador.svelte'),
  duplicates: () => import('$lib/features/duplicates/Duplicates.svelte'),
  keywords: () => import('$lib/features/keywords/Keywords.svelte'),
  schedule: () => import('$lib/features/schedule/SchedulePanel.svelte'),
  seo: () => import('$lib/features/seo/SiteSeoPanel.svelte'),
};

const cache: Partial<Record<LazyTabId, AnyComponent>> = {};

export function loadTabComponent(id: LazyTabId): Promise<AnyComponent> {
  const cached = cache[id];
  if (cached) return Promise.resolve(cached);
  return loaders[id]().then((m) => {
    cache[id] = m.default;
    return m.default;
  });
}

export function isLazyTab(id: TabId): id is LazyTabId {
  return id !== 'results';
}
