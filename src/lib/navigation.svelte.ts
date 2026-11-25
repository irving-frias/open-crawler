import { toast } from 'svelte-sonner';
import { m } from '#lib/paraglide/messages.js';
import type { AppShell, TabValue } from '#lib/app.svelte.ts';
import type { FilterState } from '#lib/features/results/FilterBar.svelte';
import { getPageIdByUrl } from '#lib/api/results.ts';

/**
 * Context navigation (T17, UI_UX_PLAN §4): deep links with context, a
 * shared history that restores the exact view on back, and per-project
 * persistence — without touching app.svelte.ts (T9 file). Everything here
 * drives the app only through its public AppShell API.
 */

export interface NavContext {
  tab: TabValue;
  projectId: string;
  filters: FilterState;
  page: number;
  search: string;
  detailPageId: string;
}

export interface NavTarget {
  tab: TabValue;
  projectId?: string;
  status?: number[];
  severity?: string[];
  issueType?: string | null;
  search?: string;
  page?: number;
  url?: string;
  snapshotA?: string;
  snapshotB?: string;
}

const HISTORY_LIMIT = 50;
const STORAGE_PREFIX = 'oc-nav:';

const DEFAULT_FILTERS: FilterState = {
  statusCodes: [],
  severities: [],
  depth: undefined,
  missingTitle: false,
  duplicateTitle: false,
  noindexOnly: false,
  is404: false,
  issueType: '',
};

export function defaultFilters(): FilterState {
  return cloneFilters(DEFAULT_FILTERS);
}

function cloneFilters(f: FilterState): FilterState {
  return {
    statusCodes: [...f.statusCodes],
    severities: [...f.severities],
    depth: f.depth,
    missingTitle: f.missingTitle,
    duplicateTitle: f.duplicateTitle,
    noindexOnly: f.noindexOnly,
    is404: f.is404,
    issueType: f.issueType,
  };
}

function filtersEqual(a: FilterState, b: FilterState): boolean {
  return (
    a.issueType === b.issueType &&
    a.depth === b.depth &&
    a.missingTitle === b.missingTitle &&
    a.duplicateTitle === b.duplicateTitle &&
    a.noindexOnly === b.noindexOnly &&
    a.is404 === b.is404 &&
    a.statusCodes.join(',') === b.statusCodes.join(',') &&
    a.severities.join(',') === b.severities.join(',')
  );
}

/** Deep-link text scheme (T17 §4.1): `results?project=X&status=404&...`. */
export function buildLink(target: NavTarget): string {
  const params = new URLSearchParams();
  if (target.tab === 'results') {
    if (target.projectId) params.set('project', target.projectId);
    for (const s of target.status ?? []) params.append('status', String(s));
    for (const s of target.severity ?? []) params.append('severity', s);
    if (target.issueType) params.set('issue', target.issueType);
    if (target.search) params.set('search', target.search);
    if (target.page && target.page > 1) params.set('page', String(target.page));
    return `results?${params.toString()}`;
  }
  if (target.tab === 'comparator') {
    if (target.projectId) params.set('project', target.projectId);
    if (target.snapshotA) params.set('a', target.snapshotA);
    if (target.snapshotB) params.set('b', target.snapshotB);
    return `compare?${params.toString()}`;
  }
  if (target.url) {
    if (target.projectId) params.set('project', target.projectId);
    params.set('url', target.url);
    return `detail?${params.toString()}`;
  }
  if (target.projectId) params.set('project', target.projectId);
  params.set('tab', target.tab);
  return `tab?${params.toString()}`;
}

export function parseLink(link: string): NavTarget | null {
  const [view, query = ''] = link.split('?');
  const params = new URLSearchParams(query);
  const projectId = params.get('project') ?? undefined;
  if (view === 'results') {
    return {
      tab: 'results',
      projectId,
      status: params
        .getAll('status')
        .map(Number)
        .filter((n) => !Number.isNaN(n)),
      severity: params.getAll('severity'),
      issueType: params.get('issue') ?? null,
      search: params.get('search') ?? undefined,
      page: Number(params.get('page') ?? 1) || 1,
    };
  }
  if (view === 'detail') {
    const url = params.get('url');
    if (!url) return null;
    return { tab: 'results', projectId, url };
  }
  if (view === 'compare') {
    return {
      tab: 'comparator',
      projectId,
      snapshotA: params.get('a') ?? undefined,
      snapshotB: params.get('b') ?? undefined,
    };
  }
  if (view === 'tab') {
    const tab = params.get('tab') as TabValue | null;
    if (!tab) return null;
    return { tab, projectId };
  }
  return null;
}

let app: AppShell | null = null;
/** Last tab set programmatically: tab-change effects ignore it. */
let lastAppliedTab: TabValue | null = null;
let boundProjectId = '';
let saveTimer: ReturnType<typeof setTimeout> | null = null;

function currentContext(): NavContext {
  if (!app) {
    return {
      tab: 'results',
      projectId: '',
      filters: cloneFilters(DEFAULT_FILTERS),
      page: 1,
      search: '',
      detailPageId: '',
    };
  }
  return {
    tab: app.activeTab,
    projectId: app.selectedProjectId,
    filters: cloneFilters(app.activeFilters),
    page: app.currentPage,
    search: app.debouncedSearch,
    detailPageId: app.detailPageId,
  };
}

function persist() {
  if (!app || !app.selectedProjectId) return;
  if (saveTimer) clearTimeout(saveTimer);
  saveTimer = setTimeout(() => {
    try {
      localStorage.setItem(
        `${STORAGE_PREFIX}${app!.selectedProjectId}`,
        JSON.stringify(currentContext())
      );
    } catch {
      // Storage full or unavailable: navigation still works in-session.
    }
  }, 300);
}

function restore(projectId: string): NavContext | null {
  try {
    const raw = localStorage.getItem(`${STORAGE_PREFIX}${projectId}`);
    if (!raw) return null;
    const ctx = JSON.parse(raw) as NavContext;
    if (!ctx || typeof ctx.tab !== 'string') return null;
    return ctx;
  } catch {
    return null;
  }
}

function setTab(tab: TabValue) {
  if (!app) return;
  lastAppliedTab = tab;
  app.activeTab = tab;
}

function applyContext(ctx: NavContext) {
  if (!app) return;
  if (ctx.projectId && ctx.projectId !== app.selectedProjectId) {
    app.selectProject(ctx.projectId);
  }
  if (!filtersEqual(ctx.filters, app.activeFilters)) {
    app.handleFilterChange(cloneFilters(ctx.filters));
  }
  if (ctx.search !== app.debouncedSearch) {
    app.onSearchInput(ctx.search);
  }
  setTab(ctx.tab);
  if (ctx.detailPageId !== app.detailPageId) {
    app.detailPageId = ctx.detailPageId;
  }
  // Restoring a search triggers a debounced load of page 1; only
  // restore the page explicitly when there is no search to race with.
  if (!ctx.search && ctx.page > 1) {
    void app.loadResults(ctx.page);
  } else if (!ctx.search && ctx.page === 1 && app.currentPage !== 1) {
    void app.loadResults(1);
  }
}

function pushHistory() {
  history.push(currentContext());
  if (history.length > HISTORY_LIMIT) history.shift();
}

// Reactive so components (breadcrumbs, back button) update on navigate.
const history = $state<NavContext[]>([]);

export const navigation = {
  get historyLength(): number {
    return history.length;
  },

  get current(): NavContext {
    return currentContext();
  },

  /** Bind once from +page.svelte after the app shell is created. */
  bind(shell: AppShell) {
    app = shell;
    if (!shell.selectedProjectId) return;
    // Idempotent per project: re-runs when the user switches projects.
    if (boundProjectId === shell.selectedProjectId) return;
    boundProjectId = shell.selectedProjectId;
    const saved = restore(shell.selectedProjectId);
    if (saved) applyContext(saved);
  },

  /**
   * Record a tab switch made from the tab UI (ResultsView effect, which
   * tracks the previous tab itself). Programmatic tab sets are ignored:
   * they already pushed their own history entry.
   */
  noteTabChange(prevTab: TabValue, newTab: TabValue) {
    if (!app) return;
    if (newTab === lastAppliedTab) return;
    history.push({ ...currentContext(), tab: prevTab });
    if (history.length > HISTORY_LIMIT) history.shift();
    lastAppliedTab = newTab;
    persist();
  },

  /** Record the current context as the latest state (filter/search edits). */
  noteContextChange() {
    persist();
  },

  goToResults(
    opts: {
      status?: number[];
      severity?: string[];
      issueType?: string | null;
      search?: string;
      page?: number;
    } = {}
  ) {
    if (!app) return;
    pushHistory();
    const filters = cloneFilters(app.activeFilters);
    if (opts.status !== undefined) filters.statusCodes = opts.status;
    if (opts.severity !== undefined) filters.severities = opts.severity;
    if (opts.issueType !== undefined) filters.issueType = opts.issueType ?? '';
    if (opts.search !== undefined) {
      app.onSearchInput(opts.search);
    }
    if (opts.status !== undefined || opts.severity !== undefined || opts.issueType !== undefined) {
      app.handleFilterChange(filters);
    }
    setTab('results');
    if (opts.page && opts.page > 1) void app.loadResults(opts.page);
    persist();
  },

  /** Dashboard issue card → results filtered by that issue type. */
  filterByIssueType(issueType: string | null) {
    if (!app) return;
    pushHistory();
    // handleFilterIssueType switches to the results tab itself; mark it
    // as programmatic so the tab-change effect does not push twice.
    lastAppliedTab = 'results';
    app.handleFilterIssueType(issueType);
    persist();
  },

  /** Clear the issue-type filter chip in the results toolbar. */
  clearIssueFilter() {
    if (!app) return;
    app.handleFilterIssueType(null);
    persist();
  },

  /** Clear every result filter and the search (Esc with nothing to close). */
  clearAllFilters() {
    if (!app) return;
    app.handleFilterChange(cloneFilters(DEFAULT_FILTERS));
    if (app.debouncedSearch) app.onSearchInput('');
    persist();
  },

  goToTab(tab: TabValue) {
    if (!app) return;
    if (tab === app.activeTab) return;
    pushHistory();
    setTab(tab);
    persist();
  },

  goToDetail(pageId: string) {
    if (!app) return;
    if (pageId === app.detailPageId) return;
    pushHistory();
    app.openDetail(pageId);
    persist();
  },

  async goToDetailByUrl(url: string) {
    if (!app) return;
    try {
      const pageId = await getPageIdByUrl(app.selectedProjectId, url);
      if (!pageId) {
        toast.error(m['nav.detail_missing']());
        return;
      }
      this.goToDetail(pageId);
    } catch (e) {
      console.error('[Nav] Failed to resolve URL to page:', e);
      toast.error(m['nav.detail_missing']());
    }
  },

  /** Close the detail overlay, restoring the context it was opened from. */
  closeDetail() {
    if (!app) return;
    if (history.length > 0) {
      const prev = history.pop()!;
      applyContext({ ...prev, detailPageId: '' });
    } else {
      app.detailPageId = '';
    }
    persist();
  },

  goBack() {
    if (!app || history.length === 0) return false;
    const prev = history.pop()!;
    applyContext(prev);
    persist();
    return true;
  },

  /** Open a serialized deep link (alerts, shared links). */
  openLink(link: string) {
    const target = parseLink(link);
    if (!target || !app) return false;
    if (target.url) {
      void this.goToDetailByUrl(target.url);
      return true;
    }
    if (target.tab === 'results') {
      this.goToResults({
        status: target.status,
        severity: target.severity,
        issueType: target.issueType,
        search: target.search,
        page: target.page,
      });
      return true;
    }
    this.goToTab(target.tab);
    return true;
  },
};
