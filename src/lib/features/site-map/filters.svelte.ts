import type { StatusClass } from './shared.js';

export type StatusFilter = StatusClass | 'all';

// Filter state shared by the site tree and the site graph, so switching tabs
// keeps the current status and search context instead of starting from zero.
export const siteMapFilters = $state<{ status: StatusFilter; search: string }>({
  status: 'all',
  search: '',
});

export function resetSiteMapFilters(): void {
  siteMapFilters.status = 'all';
  siteMapFilters.search = '';
}
