import type { StatusClass } from './shared.js';

export type StatusFilter = StatusClass | 'all';

// Filter state for the site tree, keeping the current status and search
// context across reloads instead of starting from zero.
export const siteMapFilters = $state<{ status: StatusFilter; search: string }>({
  status: 'all',
  search: '',
});

export function resetSiteMapFilters(): void {
  siteMapFilters.status = 'all';
  siteMapFilters.search = '';
}
