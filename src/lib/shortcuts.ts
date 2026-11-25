import { m } from '#lib/paraglide/messages.js';
import type { AppShell, TabValue } from '#lib/app.svelte.ts';
import { navigation } from '#lib/navigation.svelte.ts';

/**
 * Keyboard shortcuts (T18, UI_UX_PLAN §2). The plan puts the global
 * handler in app.svelte.ts, but that file is a T9 file; this module owns
 * the same behaviour using only the public AppShell API and the
 * navigation store (T17), and +page.svelte mounts it.
 */

export interface ShortcutAction {
  id: string;
  label: () => string;
  hint: string;
  keywords: string;
  group: 'crawl' | 'view' | 'export' | 'project' | 'settings';
  run: () => void;
}

const VIEW_TABS: { tab: TabValue; label: () => string; key: string }[] = [
  { tab: 'results', label: () => m['nav.results'](), key: 'r' },
  { tab: 'overview', label: () => m['tabs.overview'](), key: '' },
  { tab: 'dashboard', label: () => m['tabs.issues_dashboard'](), key: 'd' },
  { tab: 'site_tree', label: () => m['tabs.site_tree'](), key: 's' },
  { tab: 'comparator', label: () => m['tabs.comparator'](), key: 'c' },
  { tab: 'duplicates', label: () => m['tabs.duplicates'](), key: '' },
  { tab: 'keywords', label: () => m['tabs.keywords'](), key: '' },
  { tab: 'schedule', label: () => m['tabs.schedule'](), key: '' },
  { tab: 'seo', label: () => m['tabs.seo'](), key: '' },
  { tab: 'links', label: () => m['tabs.links'](), key: '' },
  { tab: 'extraction', label: () => m['tabs.extraction'](), key: '' },
];

export function getActions(app: AppShell): ShortcutAction[] {
  const actions: ShortcutAction[] = [];

  actions.push({
    id: 'crawl:start',
    label: () => m['palette.start_crawl'](),
    hint: 'Ctrl Enter',
    keywords: 'crawl rastreo iniciar start',
    group: 'crawl',
    run: () => app.handleStartCrawl(),
  });

  for (const view of VIEW_TABS) {
    actions.push({
      id: `view:${view.tab}`,
      label: () => `${m['palette.go_to']()} ${view.label()}`,
      hint: view.key ? `g ${view.key}` : '',
      keywords: `ir vista ${view.tab} ${view.label()}`,
      group: 'view',
      run: () => navigation.goToTab(view.tab),
    });
  }

  const exports: { format: 'xlsx' | 'csv' | 'sitemap'; label: () => string }[] = [
    { format: 'xlsx', label: () => m['palette.export_xlsx']() },
    { format: 'csv', label: () => m['palette.export_csv']() },
    { format: 'sitemap', label: () => m['palette.export_sitemap']() },
  ];
  for (const e of exports) {
    actions.push({
      id: `export:${e.format}`,
      label: e.label,
      hint: '',
      keywords: `exportar export ${e.format}`,
      group: 'export',
      run: () => void app.exportFull(e.format),
    });
  }

  for (const p of app.projects.optimistic) {
    actions.push({
      id: `project:${p.id}`,
      label: () => `${m['palette.switch_project']()} ${p.name}`,
      hint: '',
      keywords: `proyecto project ${p.name}`,
      group: 'project',
      run: () => app.selectProject(p.id),
    });
  }

  actions.push({
    id: 'settings:open',
    label: () => m['palette.open_settings'](),
    hint: 'g p',
    keywords: 'ajustes configuracion settings',
    group: 'settings',
    run: () => {
      app.settingsModalOpen = true;
    },
  });

  actions.push({
    id: 'settings:toggle-js',
    label: () => (app.renderJs ? m['palette.disable_js']() : m['palette.enable_js']()),
    hint: '',
    keywords: 'javascript js renderizado render',
    group: 'settings',
    run: () => {
      app.renderJs = !app.renderJs;
    },
  });

  return actions;
}

/** Case-insensitive subsequence (fuzzy) match. */
export function fuzzyMatch(query: string, text: string): boolean {
  const q = query.trim().toLowerCase();
  if (!q) return true;
  const t = text.toLowerCase();
  let qi = 0;
  for (let ti = 0; ti < t.length && qi < q.length; ti++) {
    if (t[ti] === q[qi]) qi++;
  }
  return qi === q.length;
}

export function focusResultsSearch(): boolean {
  const input = document.querySelector<HTMLInputElement>('.search-bar input');
  if (!input) return false;
  input.focus();
  input.select();
  return true;
}

export interface GlobalShortcutHandlers {
  openPalette: () => void;
  closePalette: () => boolean;
  openHelp: () => void;
  closeHelp: () => boolean;
}

function isTypingTarget(e: KeyboardEvent): boolean {
  const t = e.target as HTMLElement | null;
  if (!t) return false;
  return (
    t.tagName === 'INPUT' ||
    t.tagName === 'TEXTAREA' ||
    t.tagName === 'SELECT' ||
    t.isContentEditable
  );
}

/**
 * Install the global key handler. Returns a cleanup function.
 * Rules (§2): no shortcut steals keys while typing (except Esc and
 * Ctrl/Cmd+K); `g` sequences have a 1 s window.
 */
export function installGlobalShortcuts(
  app: AppShell,
  handlers: GlobalShortcutHandlers
): () => void {
  let pendingG = false;
  let gTimer: ReturnType<typeof setTimeout> | null = null;

  function clearG() {
    pendingG = false;
    if (gTimer) {
      clearTimeout(gTimer);
      gTimer = null;
    }
  }

  function onKeydown(e: KeyboardEvent) {
    const mod = e.ctrlKey || e.metaKey;
    const typing = isTypingTarget(e);

    if (mod && (e.key === 'k' || e.key === 'K')) {
      e.preventDefault();
      handlers.openPalette();
      return;
    }

    if (e.key === 'Escape') {
      clearG();
      if (handlers.closePalette()) return;
      if (handlers.closeHelp()) return;
      if (app.detailPageId) {
        navigation.closeDetail();
        return;
      }
      if (app.settingsModalOpen) {
        app.settingsModalOpen = false;
        return;
      }
      if (app.deleteDialogOpen) {
        app.closeDelete();
        return;
      }
      navigation.clearAllFilters();
      return;
    }

    if (typing || mod || e.altKey) return;

    if (pendingG) {
      clearG();
      const key = e.key.toLowerCase();
      const view = VIEW_TABS.find((v) => v.key === key);
      if (view) {
        navigation.goToTab(view.tab);
        return;
      }
      if (key === 'p') {
        app.settingsModalOpen = true;
        return;
      }
      return;
    }

    if (e.key === 'g' || e.key === 'G') {
      pendingG = true;
      gTimer = setTimeout(clearG, 1000);
      return;
    }

    if (e.key === '/') {
      e.preventDefault();
      if (app.activeTab !== 'results') navigation.goToTab('results');
      // The results tab renders after the tab switch; retry briefly.
      if (!focusResultsSearch()) {
        setTimeout(() => focusResultsSearch(), 150);
      }
      return;
    }

    if (e.key === '?') {
      handlers.openHelp();
    }
  }

  window.addEventListener('keydown', onKeydown);
  return () => {
    window.removeEventListener('keydown', onKeydown);
    clearG();
  };
}
