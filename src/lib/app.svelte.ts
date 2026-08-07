import { listen } from '@tauri-apps/api/event';
import { toast } from 'svelte-sonner';
import * as api from '$lib/api';
import type {
  ExportPackageInfo,
  ImportConflictMode,
  ImportSummary,
  TransferInfo,
  TransferProgress,
} from '$lib/api/transfer';
import type { Project } from '$lib/api/types';
import { applyTheme, applyUiStyle } from '$lib/theme.js';
import { useOptimistic, type OptimisticAction } from '$lib/use-optimistic.svelte.js';
import { m } from '$lib/paraglide/messages.js';
import { notify } from '$lib/utils';
import { getContext, setContext } from 'svelte';
import type { FilterState } from '$lib/features/results/FilterBar.svelte';
import {
  parseP2PCode,
  startP2PSender,
  receiveP2P as receivePeerPackage,
  type P2PProgress,
} from '$lib/features/transfer/p2p';

const APP_SHELL_KEY = Symbol('open-crawler-app-shell');

export type CrawlStatus = 'idle' | 'running' | 'paused' | 'completed' | 'error';
export type TabValue = 'results' | 'overview' | 'dashboard' | 'site_tree' | 'comparator' | 'duplicates' | 'keywords' | 'schedule';

export interface CrawlProgressState {
  crawled: number;
  queued: number;
  current: string;
  errors: number;
}

export interface ResultsState {
  items: any[];
  total: number;
  page: number;
  page_size: number;
}

export interface ResumableInfo {
  pages_crawled: number;
  queue_remaining: number;
  elapsed_secs: number;
}

export interface ExportProgress {
  running: boolean;
  percent: number;
  stage: string;
}

export interface CrawlFormConfig {
  seedUrl: string;
  maxDepth: number;
  maxCrawlTime: number;
  respectRobots: boolean;
  checkSitemap: boolean;
  checkSemantics: boolean;
  proxyUrl: string;
  proxyUser: string;
  proxyPass: string;
  cookies: string;
  siteUser: string;
  sitePass: string;
}

export interface AppFields {
  selectedProjectId: string;
  newProjectName: string;
  renamingProjectId: string;
  renamingName: string;
  seedUrl: string;
  maxDepth: number;
  respectRobots: boolean;
  checkSitemap: boolean;
  checkSemantics: boolean;
  maxCrawlTime: number;
  proxyUrl: string;
  proxyUser: string;
  proxyPass: string;
  cookies: string;
  siteUser: string;
  sitePass: string;
  seedUrlsByProject: Record<string, string>;
  configByProject: Record<string, CrawlFormConfig>;
  status: CrawlStatus;
  progress: CrawlProgressState;
  results: ResultsState;
  error: string;
  sitemapInfo: string;
  activeFilters: FilterState;
  currentPage: number;
  pageSize: number;
  pageSizeSelect: string;
  resumableInfo: ResumableInfo | null;
  showResumeDialog: boolean;
  streamedCount: number;
  siteFavicon: string;
  exportProgress: ExportProgress;
  transferDialogOpen: boolean;
  transferBusy: boolean;
  transferProgress: TransferProgress;
  lastPackage: ExportPackageInfo | null;
  activeTransfer: TransferInfo | null;
  shareImporting: boolean;
  shareChecked: boolean;
  p2pBusy: boolean;
  p2pPeerId: string;
  p2pStatus: string;
  p2pProgress: P2PProgress;
  p2pError: string;
  btBusy: boolean;
  btStatus: string;
  btError: string;
  expandedIssueUrl: string;
  activeTab: TabValue;
  detailPageId: string;
  searchQuery: string;
  debouncedSearch: string;
  resultsLoading: boolean;
  settingsModalOpen: boolean;
  notificationsEnabled: boolean;
  deletePendingId: string | null;
  deleteDialogOpen: boolean;
  initialized: boolean;
}

export type ProjectsStore = ReturnType<typeof useOptimistic<Project, OptimisticAction<Project>>>;

export interface AppShell extends AppFields {
  projects: ProjectsStore;
  getSelectedProject(): Project | undefined;
  totalPages(): number;
  formatDuration(secs: number): string;
  selectProject(id: string): void;
  createProject(): Promise<void>;
  startRename(id: string, currentName: string): Promise<void>;
  confirmRename(): Promise<void>;
  cancelRename(): Promise<void>;
  requestDelete(id: string): void;
  closeDelete(): void;
  deleteProject(id: string): Promise<void>;
  handleStartCrawl(): void;
  startCrawl(resume?: boolean): Promise<void>;
  stopCrawl(): Promise<void>;
  loadResults(page?: number): Promise<void>;
  handleFilterChange(filters: FilterState): void;
  goToPage(page: number): void;
  changePageSize(newSize: number): void;
  handleFilterIssueType(issueType: string | null): void;
  openDetail(pageId: string): void;
  onSearchInput(query: string): void;
  exportFull(format: 'xlsx' | 'csv'): Promise<void>;
  exportPackage(includeCredentials: boolean, lightweight: boolean, shareAfter: boolean): Promise<ExportPackageInfo | null>;
  exportAndStartWifi(includeCredentials: boolean, lightweight: boolean, minutes?: number): Promise<void>;
  exportAndShare(includeCredentials: boolean, lightweight: boolean): Promise<void>;
  exportAndStartP2P(includeCredentials: boolean, lightweight: boolean): Promise<void>;
  stopP2P(): void;
  receiveP2P(code: string, mode: ImportConflictMode): Promise<void>;
  sendViaObex(addr: string, includeCredentials: boolean, lightweight: boolean): Promise<void>;
  importPackage(filePath: string, mode: ImportConflictMode): Promise<ImportSummary>;
  importSharedIntent(mode: ImportConflictMode): Promise<ImportSummary | null>;
  startTransferServer(filePath: string, minutes?: number): Promise<TransferInfo | null>;
  stopTransferServer(): Promise<void>;
  downloadTransfer(url: string, dest: string): Promise<void>;
}

export function formatDuration(secs: number): string {
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  const s = secs % 60;
  if (h > 0) return `${h}h ${m}m ${s}s`;
  if (m > 0) return `${m}m ${s}s`;
  return `${s}s`;
}

export function createAppShell(): AppShell {
  let projectsBase = $state<Project[]>([]);
  let p2pSender: ReturnType<typeof startP2PSender> | null = null;
  const projects = useOptimistic<Project, OptimisticAction<Project>>(
    () => projectsBase,
    (list, action) => (action.type === 'delete' ? list.filter((p) => p.id !== action.id) : list)
  );

  const state = $state({
    selectedProjectId: '',
    newProjectName: '',
    renamingProjectId: '',
    renamingName: '',
    seedUrl: '',
    maxDepth: 10,
    respectRobots: true,
    checkSitemap: true,
    checkSemantics: true,
    maxCrawlTime: 3600,
    proxyUrl: '',
    proxyUser: '',
    proxyPass: '',
    cookies: '',
    siteUser: '',
    sitePass: '',
    seedUrlsByProject: {} as Record<string, string>,
    configByProject: {} as Record<string, CrawlFormConfig>,
    status: 'idle' as CrawlStatus,
    progress: { crawled: 0, queued: 0, current: '', errors: 0 } as CrawlProgressState,
    results: { items: [], total: 0, page: 1, page_size: 50 } as ResultsState,
    error: '',
    sitemapInfo: '',
    activeFilters: {
      statusCodes: [],
      severities: [],
      depth: undefined,
      missingTitle: false,
      duplicateTitle: false,
      noindexOnly: false,
      is404: false,
      issueType: '',
    } as FilterState,
    currentPage: 1,
    pageSize: 50,
    pageSizeSelect: '50',
    resumableInfo: null as ResumableInfo | null,
    showResumeDialog: false,
    streamedCount: 0,
    siteFavicon: '',
    exportProgress: { running: false, percent: 0, stage: '' } as ExportProgress,
    transferDialogOpen: false,
    transferBusy: false,
    transferProgress: { stage: '', processed: 0, total: 0, percent: 0 } as TransferProgress,
    lastPackage: null as ExportPackageInfo | null,
    activeTransfer: null as TransferInfo | null,
    shareImporting: false,
    shareChecked: false,
    p2pBusy: false,
    p2pPeerId: '',
    p2pStatus: '',
    p2pProgress: { transferred: 0, total: 0, percent: 0 } as P2PProgress,
    p2pError: '',
    btBusy: false,
    btStatus: '',
    btError: '',
    expandedIssueUrl: '',
    activeTab: 'results' as TabValue,
    detailPageId: '',
    searchQuery: '',
    debouncedSearch: '',
    resultsLoading: false,
    settingsModalOpen: false,
    notificationsEnabled: true,
    deletePendingId: null as string | null,
    deleteDialogOpen: false,
    initialized: false,
  }) as AppShell;

  state.projects = projects;

  // ==================== PERSISTENCE ====================

  const PERSIST_KEY = 'open-crawler:ui-state-v1';

  function readPersisted(): Record<string, unknown> {
    try {
      if (typeof localStorage === 'undefined') return {};
      return JSON.parse(localStorage.getItem(PERSIST_KEY) || '{}') as Record<string, unknown>;
    } catch {
      return {};
    }
  }

  const persisted = readPersisted();
  if (typeof persisted.selectedProjectId === 'string' && persisted.selectedProjectId) {
    state.selectedProjectId = persisted.selectedProjectId;
  }
  if (persisted.seedUrlsByProject && typeof persisted.seedUrlsByProject === 'object') {
    state.seedUrlsByProject = { ...(persisted.seedUrlsByProject as Record<string, string>) };
  }
  if (persisted.configByProject && typeof persisted.configByProject === 'object') {
    state.configByProject = { ...(persisted.configByProject as Record<string, CrawlFormConfig>) };
  }
  state.seedUrl = state.seedUrlsByProject[state.selectedProjectId] ?? (typeof persisted.seedUrl === 'string' ? persisted.seedUrl : '');
  const initialConfig = state.configByProject[state.selectedProjectId];
  if (initialConfig) {
    applyFormConfig(initialConfig);
    state.seedUrl = initialConfig.seedUrl;
  }
  if (typeof persisted.pageSize === 'number' && persisted.pageSize > 0) state.pageSize = persisted.pageSize;
  if (typeof persisted.maxDepth === 'number') state.maxDepth = persisted.maxDepth;
  if (typeof persisted.maxCrawlTime === 'number') state.maxCrawlTime = persisted.maxCrawlTime;
  if (typeof persisted.respectRobots === 'boolean') state.respectRobots = persisted.respectRobots;
  if (typeof persisted.checkSitemap === 'boolean') state.checkSitemap = persisted.checkSitemap;
  if (typeof persisted.checkSemantics === 'boolean') state.checkSemantics = persisted.checkSemantics;
  if (persisted.activeTab) state.activeTab = persisted.activeTab as TabValue;
  if (persisted.lastPackage && typeof persisted.lastPackage === 'object') {
    state.lastPackage = persisted.lastPackage as ExportPackageInfo;
  }
  const persistedActiveTransfer =
    persisted.activeTransfer && typeof persisted.activeTransfer === 'object'
      ? (persisted.activeTransfer as TransferInfo)
      : null;

  $effect(() => {
    const snapshot = {
      selectedProjectId: state.selectedProjectId,
      seedUrlsByProject: state.seedUrlsByProject,
      seedUrl: state.seedUrl,
      configByProject: state.configByProject,
      pageSize: state.pageSize,
      maxDepth: state.maxDepth,
      maxCrawlTime: state.maxCrawlTime,
      respectRobots: state.respectRobots,
      checkSitemap: state.checkSitemap,
      checkSemantics: state.checkSemantics,
      activeTab: state.activeTab,
      lastPackage: state.lastPackage,
      activeTransfer: state.activeTransfer,
    };
    try {
      localStorage.setItem(PERSIST_KEY, JSON.stringify(snapshot));
    } catch {
      // Storage may be unavailable (private mode, quota) — keep running in-memory.
    }
  });

  let exportHideTimer: ReturnType<typeof setTimeout> | null = null;
  let searchDebounceTimer: ReturnType<typeof setTimeout> | null = null;
  let progressPollTimer: ReturnType<typeof setInterval> | null = null;
  let batchRefreshTimer: ReturnType<typeof setTimeout> | null = null;
  let resultsRequestSeq = 0;
  let streamRefreshing = false;
  let streamRefreshQueued = false;
  let transferRestored = false;
  const unlistenFns: (() => void)[] = [];
  let disposed = false;

  $effect(() => {
    state.pageSizeSelect = String(state.pageSize);
  });

  $effect(() => {
    setupListeners();
    loadProjects();
    loadSettings();
    return () => {
      disposed = true;
      unlistenFns.forEach((fn) => fn());
      unlistenFns.length = 0;
      if (progressPollTimer) clearInterval(progressPollTimer);
      progressPollTimer = null;
      if (batchRefreshTimer) clearTimeout(batchRefreshTimer);
      batchRefreshTimer = null;
      if (searchDebounceTimer) clearTimeout(searchDebounceTimer);
      searchDebounceTimer = null;
      if (exportHideTimer) clearTimeout(exportHideTimer);
      exportHideTimer = null;
    };
  });

  async function setupListeners() {
    // Guard against listener leaks: `listen()` resolves asynchronously, so the
    // effect cleanup may run before the subscriptions are registered. Register
    // only while the shell is alive and immediately drop any that resolve after
    // it has been disposed (e.g. HMR / re-runs).
    const register = (p: Promise<() => void>) => {
      p.then((un) => {
        if (disposed) un();
        else unlistenFns.push(un);
      }).catch((e) => {
        console.error('[Crawler] Failed to subscribe:', e);
      });
    };

    // Catch up with pending refreshes when the window becomes visible again.
    document.addEventListener('visibilitychange', handleVisibilityChange);
    unlistenFns.push(() =>
      document.removeEventListener('visibilitychange', handleVisibilityChange)
    );

    // On Android, packages shared into the app are only available once the
    // webview is focused again — drain the share-target queue each time.
    register(listen<any>('tauri://focus', () => {
      checkIncomingShare();
    }));

    register(listen<any>('crawl-started', (event) => {
      console.log('[Crawler] crawl-started:', event.payload);
      if (state.notificationsEnabled) {
        notify(m['notifications.crawl_started'](), m['notifications.crawl_started_desc']());
      }
    }));

    register(listen<any>('crawl-progress', (event) => {
      const p = event.payload;
      if (p.project_id !== state.selectedProjectId) return;
      state.progress.crawled = p.urls_crawled;
      state.progress.queued = p.urls_queued;
      state.progress.current = p.current_url;
      state.progress.errors = p.errors;
    }));

    register(listen<any>('crawl-batch', (event) => {
      const p = event.payload;
      if (p.project_id !== state.selectedProjectId) return;
      if (p.count) state.streamedCount += p.count;
      scheduleResultsRefresh();
    }));

    register(listen<any>('crawl-complete', (event) => {
      const p = event.payload;
      if (p.project_id !== state.selectedProjectId) return;
      state.status = 'completed';
      state.resumableInfo = null;
      state.streamedCount = 0;
      loadResults(1);
      if (state.notificationsEnabled) {
        notify(m['notifications.crawl_complete'](), m['notifications.crawl_complete_desc']());
      }
    }));

    register(listen<any>('crawl-error', (event) => {
      const p = event.payload;
      if (p.project_id !== state.selectedProjectId) return;
      state.status = 'error';
      state.error = p.error || String(p);
      if (state.notificationsEnabled) {
        notify(m['notifications.crawl_error'](), p.error || String(p));
      }
    }));

    register(listen<any>('crawl-stopped', (event) => {
      const p = event.payload;
      if (p.project_id !== state.selectedProjectId) return;
      state.status = 'idle';
      state.streamedCount = 0;
      checkResumable();
      if (state.notificationsEnabled) {
        notify(m['notifications.crawl_stopped'](), m['notifications.crawl_stopped_desc']());
      }
    }));

    register(listen<any>('sitemap-discovered', (event) => {
      const p = event.payload;
      state.sitemapInfo = p.fallback
        ? m['sitemap.fallback']()
        : m['sitemap.found']({
            count: String(p.urls_found),
            sitemaps: String(p.sitemaps_checked),
          });
    }));
  }

  // ==================== PROJECTS ====================

  async function loadProjects() {
    try {
      const data = await api.projects.listProjects();
      projectsBase = data;
      if (data.length > 0) {
        const id =
          state.selectedProjectId && data.some((p) => p.id === state.selectedProjectId)
            ? state.selectedProjectId
            : data[0].id;
        if (id !== state.selectedProjectId) {
          selectProject(id);
        } else if (state.results.items.length === 0) {
          // Selection was restored from localStorage: activate its content
          // without re-selecting (avoids the empty flash on reload).
          restoreProjectConfig(state.selectedProjectId);
          loadResults(1);
          checkResumable();
          checkRunningCrawl();
        }
      } else {
        state.selectedProjectId = '';
      }
    } catch (e) {
      console.error('[Projects] Failed to load:', e);
    } finally {
      state.initialized = true;
      if (!transferRestored) {
        transferRestored = true;
        restoreTransferServer();
      }
      checkIncomingShare();
    }
  }

  async function loadSettings() {
    try {
      const settings = await api.settings.getSettings();
      if (settings.page_size) {
        state.pageSize = parseInt(settings.page_size, 10);
      }
      if (settings.notifications_enabled !== undefined) {
        state.notificationsEnabled = settings.notifications_enabled === 'true';
      }
      if (settings.theme) {
        applyTheme(settings.theme);
      }
      if (settings.ui_style) {
        applyUiStyle(settings.ui_style);
      }
    } catch (e) {
      // Settings may not exist yet, use defaults
    }
  }

  function captureFormConfig(): CrawlFormConfig {
    return {
      seedUrl: state.seedUrl,
      maxDepth: state.maxDepth,
      maxCrawlTime: state.maxCrawlTime,
      respectRobots: state.respectRobots,
      checkSitemap: state.checkSitemap,
      checkSemantics: state.checkSemantics,
      proxyUrl: state.proxyUrl,
      proxyUser: state.proxyUser,
      proxyPass: state.proxyPass,
      cookies: state.cookies,
      siteUser: state.siteUser,
      sitePass: state.sitePass,
    };
  }

  function applyFormConfig(cfg: CrawlFormConfig) {
    state.maxDepth = cfg.maxDepth ?? 10;
    state.maxCrawlTime = cfg.maxCrawlTime ?? 3600;
    state.respectRobots = cfg.respectRobots ?? true;
    state.checkSitemap = cfg.checkSitemap ?? true;
    state.checkSemantics = cfg.checkSemantics ?? true;
    state.proxyUrl = cfg.proxyUrl ?? '';
    state.proxyUser = cfg.proxyUser ?? '';
    state.proxyPass = cfg.proxyPass ?? '';
    state.cookies = cfg.cookies ?? '';
    state.siteUser = cfg.siteUser ?? '';
    state.sitePass = cfg.sitePass ?? '';
  }

  // Restores the last crawl's settings from the backend when there is no local
  // snapshot for the project yet (first time opening it after a reload).
  function restoreProjectConfig(id: string) {
    if (state.configByProject[id] || !id) return;
    api.crawl
      .getLastCrawlConfig(id)
      .then((cfg) => {
        if (!cfg || disposed) return;
        const restored: CrawlFormConfig = {
          seedUrl: cfg.seed_urls?.[0] ?? '',
          maxDepth: cfg.max_depth ?? 10,
          maxCrawlTime: cfg.max_crawl_time_secs ?? 3600,
          respectRobots: cfg.respect_robots ?? true,
          checkSitemap: cfg.check_sitemap ?? true,
          checkSemantics: cfg.check_semantics ?? true,
          proxyUrl: cfg.proxy?.url ?? '',
          proxyUser: cfg.proxy?.username ?? '',
          proxyPass: cfg.proxy?.password ?? '',
          cookies: cfg.cookies?.join('\n') ?? '',
          siteUser: cfg.site_auth?.username ?? '',
          sitePass: cfg.site_auth?.password ?? '',
        };
        state.configByProject[id] = restored;
        if (state.selectedProjectId === id) {
          applyFormConfig(restored);
          if (!state.seedUrl) state.seedUrl = restored.seedUrl;
        }
      })
      .catch((e) => console.error('[Crawler] Failed to restore last crawl config:', e));
  }

  function selectProject(id: string) {
    if (state.selectedProjectId && state.selectedProjectId !== id) {
      state.seedUrlsByProject[state.selectedProjectId] = state.seedUrl;
      state.configByProject[state.selectedProjectId] = captureFormConfig();
    }
    state.selectedProjectId = id;
    const local = state.configByProject[id];
    if (local) {
      applyFormConfig(local);
      state.seedUrl = local.seedUrl;
    } else {
      state.seedUrl = state.seedUrlsByProject[id] ?? '';
    }
    state.status = 'idle';
    state.progress = { crawled: 0, queued: 0, current: '', errors: 0 };
    state.results = { items: [], total: 0, page: 1, page_size: 50 };
    state.currentPage = 1;
    state.error = '';
    state.sitemapInfo = '';
    state.resumableInfo = null;
    state.showResumeDialog = false;
    state.streamedCount = 0;
    state.expandedIssueUrl = '';
    state.activeFilters.issueType = '';
    state.activeTab = 'results';
    state.detailPageId = '';
    restoreProjectConfig(id);
    loadResults(1);
    checkResumable();
    checkRunningCrawl();
  }

  async function checkRunningCrawl() {
    if (!state.selectedProjectId || disposed) return;
    try {
      const runningIds = await api.crawl.getRunningCrawls();
      if (disposed) return;
      if (runningIds.includes(state.selectedProjectId)) {
        state.status = 'running';
        pollCrawlProgress();
      }
    } catch (e) {
      console.error('[Crawler] Failed to check running crawls:', e);
    }
  }

  async function pollCrawlProgress() {
    if (disposed) return;
    if (progressPollTimer) clearInterval(progressPollTimer);
    progressPollTimer = setInterval(async () => {
      if (disposed || state.status !== 'running' || !state.selectedProjectId) {
        clearInterval(progressPollTimer!);
        progressPollTimer = null;
        return;
      }
      try {
        const p = await api.crawl.getCrawlStatus(state.selectedProjectId);
        if (p) {
          state.progress.crawled = p.urls_crawled;
          state.progress.queued = p.urls_queued;
          state.progress.current = p.current_url;
          state.progress.errors = p.errors;
        } else {
          state.status = 'completed';
          clearInterval(progressPollTimer!);
          progressPollTimer = null;
          loadResults(1);
        }
      } catch (e) {
        console.error('[Crawler] Failed to poll progress:', e);
      }
    }, 1000);
  }

  async function checkResumable() {
    if (!state.selectedProjectId) return;
    try {
      const info = await api.crawl.checkResumableCrawl(state.selectedProjectId);
      state.resumableInfo = info;
    } catch (e) {
      console.error('[Crawler] Failed to check resumable:', e);
    }
  }

  async function createProject() {
    if (!state.newProjectName.trim()) return;
    try {
      const project = await api.projects.createProject(state.newProjectName.trim());
      state.newProjectName = '';
      await loadProjects();
      selectProject(project.id);
    } catch (e) {
      state.error = String(e);
    }
  }

  async function startRename(id: string, currentName: string) {
    state.renamingProjectId = id;
    state.renamingName = currentName;
  }

  async function confirmRename() {
    if (!state.renamingName.trim()) return;
    try {
      await api.projects.renameProject(state.renamingProjectId, state.renamingName.trim());
      state.renamingProjectId = '';
      state.renamingName = '';
      await loadProjects();
    } catch (e) {
      state.error = String(e);
    }
  }

  async function cancelRename() {
    state.renamingProjectId = '';
    state.renamingName = '';
  }

  function requestDelete(id: string) {
    state.deletePendingId = id;
    state.deleteDialogOpen = true;
  }

  function closeDelete() {
    state.deleteDialogOpen = false;
    state.deletePendingId = null;
  }

  async function deleteProject(id: string) {
    closeDelete();
    const wasSelected = state.selectedProjectId === id;
    const prevSelected = state.selectedProjectId;
    const prevResults = state.results;
    const prevStatus = state.status;
    const prevProgress = state.progress;

    const action: OptimisticAction<Project> = { id, type: 'delete' };

    if (wasSelected) {
      state.selectedProjectId = '';
      state.results = { items: [], total: 0, page: 1, page_size: 50 };
      state.status = 'idle';
      state.progress = { crawled: 0, queued: 0, current: '', errors: 0 };
    }
    projects.add(action);

    try {
      await api.projects.deleteProject(id);
      projectsBase = projectsBase.filter((p) => p.id !== id);
    } catch (e) {
      if (wasSelected) {
        state.selectedProjectId = prevSelected;
        state.results = prevResults;
        state.status = prevStatus;
        state.progress = prevProgress;
        loadResults(1);
        checkResumable();
      }
      state.error = String(e);
      toast.error(m['errors.delete_failed']());
    } finally {
      projects.settle(action);
    }
  }

  // ==================== CRAWL ====================

  async function startCrawl(resume: boolean = false) {
    if (!state.selectedProjectId) return;
    state.configByProject[state.selectedProjectId] = captureFormConfig();
    state.seedUrlsByProject[state.selectedProjectId] = state.seedUrl;
    try {
      state.status = 'running';
      state.error = '';
      state.sitemapInfo = '';
      state.showResumeDialog = false;
      state.streamedCount = 0;
      state.currentPage = 1;
      state.results = { items: [], total: 0, page: 1, page_size: state.pageSize };
      api.crawl.getFavicon(state.seedUrl.trim()).then((icon) => {
        if (icon) state.siteFavicon = icon;
      }).catch(() => {
        state.siteFavicon = '';
      });
      await api.crawl.startCrawl(state.selectedProjectId, {
        seed_urls: [state.seedUrl],
        max_depth: state.maxDepth,
        respect_robots: state.respectRobots,
        check_sitemap: state.checkSitemap,
        check_semantics: state.checkSemantics,
        max_crawl_time_secs: state.maxCrawlTime,
        cookies: state.cookies
          .split('\n')
          .map((c) => c.trim())
          .filter((c) => c.length > 0),
        site_auth:
          state.siteUser.trim() || state.sitePass
            ? { username: state.siteUser, password: state.sitePass }
            : null,
        proxy: state.proxyUrl.trim()
          ? {
              url: state.proxyUrl.trim(),
              username: state.proxyUser.trim() || null,
              password: state.proxyPass || null,
            }
          : null,
      });
    } catch (e) {
      state.status = 'error';
      state.error = String(e);
      toast.error(String(e));
    }
  }

  function handleStartCrawl() {
    if (state.resumableInfo) {
      state.showResumeDialog = true;
    } else {
      startCrawl(false);
    }
  }

  async function stopCrawl() {
    if (!state.selectedProjectId) return;
    try {
      await api.crawl.stopCrawl(state.selectedProjectId);
      toast.info(m['progress.title']() + ': stopped');
    } catch (e) {
      state.error = String(e);
      toast.error(String(e));
    }
  }

  // ==================== RESULTS ====================

  function scheduleResultsRefresh() {
    if (state.activeTab !== 'results') return;
    // Skip polling while the window is hidden: the backend only emits
    // crawl-batch while a crawl is active, so nothing is missed by batching
    // the refresh into a single catch-up call when the tab becomes visible.
    if (document.visibilityState === 'hidden') return;
    if (batchRefreshTimer) return;
    batchRefreshTimer = setTimeout(() => {
      batchRefreshTimer = null;
      runStreamRefresh();
    }, 1000);
  }

  function handleVisibilityChange() {
    if (document.visibilityState === 'visible') {
      scheduleResultsRefresh();
    }
  }

  async function runStreamRefresh() {
    if (streamRefreshing) {
      streamRefreshQueued = true;
      return;
    }
    streamRefreshing = true;
    try {
      await loadResults(state.currentPage, { silent: true });
    } finally {
      streamRefreshing = false;
      if (streamRefreshQueued) {
        streamRefreshQueued = false;
        scheduleResultsRefresh();
      }
    }
  }

  async function loadResults(page: number = 1, opts: { silent?: boolean } = {}) {
    if (!state.selectedProjectId) return;
    const seq = ++resultsRequestSeq;
    state.currentPage = page;
    if (!opts.silent) state.resultsLoading = true;
    try {
      const data = await api.results.getResults({
        projectId: state.selectedProjectId,
        page,
        pageSize: state.pageSize,
        semanticIssueType: state.activeFilters.issueType || null,
        search: state.debouncedSearch || null,
        statusFilter: state.activeFilters.statusCodes.length > 0 ? state.activeFilters.statusCodes : null,
        severityFilter: state.activeFilters.severities.length > 0 ? state.activeFilters.severities : null,
        depthFilter: state.activeFilters.depth,
        missingTitle: state.activeFilters.missingTitle || null,
        duplicateTitle: state.activeFilters.duplicateTitle || null,
        noindexOnly: state.activeFilters.noindexOnly || null,
        is404: state.activeFilters.is404 || null,
      });
      if (seq !== resultsRequestSeq) return;
      if (
        page === state.results.page &&
        data.total === state.results.total &&
        data.items.length === state.results.items.length
      ) {
        return;
      }
      state.results = data;
    } catch (e) {
      if (seq === resultsRequestSeq) console.error('[Crawler] Failed to load results:', e);
    } finally {
      if (seq === resultsRequestSeq && !opts.silent) state.resultsLoading = false;
    }
  }

  function handleFilterChange(filters: FilterState) {
    state.activeFilters = filters;
    loadResults(1);
  }

  function goToPage(page: number) {
    if (page < 1 || page > state.totalPages()) return;
    loadResults(page);
  }

  function changePageSize(newSize: number) {
    state.pageSize = newSize;
    state.currentPage = 1;
    loadResults(1);
  }

  function handleFilterIssueType(issueType: string | null) {
    const type = issueType || '';
    if (type) {
      // Clicking an issue from the dashboard resets the remaining filters so
      // the results list matches the dashboard counts, and navigates there.
      state.activeFilters = {
        statusCodes: [],
        severities: [],
        depth: undefined,
        missingTitle: false,
        duplicateTitle: false,
        noindexOnly: false,
        is404: false,
        issueType: type,
      };
      state.activeTab = 'results';
      state.searchQuery = '';
      state.debouncedSearch = '';
    } else {
      state.activeFilters.issueType = '';
    }
    state.currentPage = 1;
    loadResults(1);
  }

  function openDetail(pageId: string) {
    state.detailPageId = pageId;
  }

  function onSearchInput(query: string) {
    state.searchQuery = query;
    if (searchDebounceTimer) clearTimeout(searchDebounceTimer);
    searchDebounceTimer = setTimeout(() => {
      state.debouncedSearch = query;
      state.currentPage = 1;
      loadResults(1);
    }, 400);
  }

  // ==================== EXPORT ====================

  async function exportFull(format: 'xlsx' | 'csv') {
    if (!state.selectedProjectId || state.exportProgress.running) return;
    try {
      const mobile = await api.crawl.isMobile();
      const ext = format === 'xlsx' ? 'xlsx' : 'csv';
      const defaultName = `crawl-results-${state.selectedProjectId}.${ext}`;

      // Show the native save dialog on every platform. On Android this opens the
      // SAF picker (ACTION_CREATE_DOCUMENT) so the user can choose the destination.
      const { save } = await import('@tauri-apps/plugin-dialog');
      const picked = await save({
        defaultPath: defaultName,
        filters: [
          {
            name: format === 'xlsx' ? 'Excel' : 'CSV',
            extensions: [ext],
          },
        ],
      });
      if (!picked) return;
      const path = mobile || picked.toLowerCase().endsWith(`.${ext}`) ? picked : `${picked}.${ext}`;

      state.exportProgress = { running: true, percent: 0, stage: '' };
      const unlisten = await listen<{ stage: string; percent: number }>('export-progress', (event) => {
        state.exportProgress = {
          running: true,
          percent: Math.min(event.payload.percent, 100),
          stage: event.payload.stage,
        };
      });

      try {
        await api.exportApi.exportFull(state.selectedProjectId, path, format);
        state.exportProgress = { running: false, percent: 100, stage: '' };
        toast.success(mobile ? m['export.shared']() : `Exported to ${path.split(/[/\\]/).pop()}`);
        if (state.notificationsEnabled) {
          notify(m['notifications.export_complete'](), m['notifications.export_complete_desc']());
        }
        if (exportHideTimer) clearTimeout(exportHideTimer);
        exportHideTimer = setTimeout(() => {
          state.exportProgress = { running: false, percent: 0, stage: '' };
        }, 1500);
      } finally {
        unlisten();
      }
    } catch (e) {
      state.exportProgress = { running: false, percent: 0, stage: '' };
      state.error = String(e);
      toast.error(String(e));
    }
  }

  // ==================== TRANSFER (package export / import / WiFi) ====================

  async function exportPackage(includeCredentials: boolean, lightweight: boolean, shareAfter: boolean) {
    if (!state.selectedProjectId || state.transferBusy) return null;
    state.transferBusy = true;
    try {
      const projectId = state.selectedProjectId;
      const mobile = await api.crawl.isMobile();
      const defaultName = `open-crawler-${projectId}.ocproj`;

      let filePath: string;
      if (mobile) {
        filePath = defaultName;
      } else {
        const { save } = await import('@tauri-apps/plugin-dialog');
        const picked = await save({
          defaultPath: defaultName,
          filters: [{ name: 'Open Crawler Package', extensions: ['ocproj'] }],
        });
        if (!picked) return null;
        filePath = picked.toLowerCase().endsWith('.ocproj') ? picked : `${picked}.ocproj`;
      }

      const info = await api.transferApi.exportPackage(
        [projectId],
        filePath,
        lightweight,
        includeCredentials,
        shareAfter,
        false
      );
      state.lastPackage = info;
      toast.success(m['transfer.exported']({ name: info.file_name }));
      return info;
    } catch (e) {
      state.error = String(e);
      toast.error(String(e));
      return null;
    } finally {
      state.transferBusy = false;
    }
  }

  // Direct share: export to a managed transfers dir (no save dialog / share
  // sheet) and immediately start the LAN WiFi server so it can be scanned.
  async function exportAndStartWifi(includeCredentials: boolean, lightweight: boolean, minutes?: number) {
    if (!state.selectedProjectId || state.transferBusy) return;
    state.transferBusy = true;
    try {
      const projectId = state.selectedProjectId;
      const name = `open-crawler-${projectId}.ocproj`;
      const info = await api.transferApi.exportPackage(
        [projectId],
        name,
        lightweight,
        includeCredentials,
        false,
        true
      );
      state.lastPackage = info;
      const transfer = await api.transferApi.startTransferServer(info.path, minutes ?? 15);
      state.activeTransfer = transfer;
      toast.success(m['transfer.exported']({ name: info.file_name }));
    } catch (e) {
      state.error = String(e);
      toast.error(String(e));
    } finally {
      state.transferBusy = false;
    }
  }

  // Direct share to a nearby device: on mobile this uses the system share
  // sheet (Bluetooth / Nearby); on macOS it opens the native share sheet
  // (AirDrop). Desktop exports silently to the transfers dir first so there
  // is no save dialog.
  async function exportAndShare(includeCredentials: boolean, lightweight: boolean) {
    if (!state.selectedProjectId || state.transferBusy) return;
    const mobile = await api.crawl.isMobile();
    if (mobile) {
      // Mobile: exportPackage handles the busy flag and opens the native
      // share sheet (shareAfter=true) after writing the package.
      await exportPackage(includeCredentials, lightweight, true);
      return;
    }
    state.transferBusy = true;
    try {
      const projectId = state.selectedProjectId;
      const name = `open-crawler-${projectId}.ocproj`;
      const info = await api.transferApi.exportPackage(
        [projectId],
        name,
        lightweight,
        includeCredentials,
        false,
        true
      );
      state.lastPackage = info;
      await api.transferApi.openShareSheet(info.path);
      toast.success(m['transfer.exported']({ name: info.file_name }));
    } catch (e) {
      state.error = String(e);
      toast.error(String(e));
    } finally {
      state.transferBusy = false;
    }
  }

  // OBEX Object Push over Bluetooth (Windows/Linux). Silent export, then the
  // package is streamed to the given device address over RFCOMM.
  async function sendViaObex(addr: string, includeCredentials: boolean, lightweight: boolean) {
    if (!state.selectedProjectId || state.btBusy || !addr.trim()) return;
    state.btBusy = true;
    state.btStatus = 'exporting';
    state.btError = '';
    try {
      const projectId = state.selectedProjectId;
      const name = `open-crawler-${projectId}.ocproj`;
      const info = await api.transferApi.exportPackage(
        [projectId],
        name,
        lightweight,
        includeCredentials,
        false,
        true
      );
      state.lastPackage = info;
      state.btStatus = 'sending';
      const unlisten = await listen<TransferProgress>('transfer-progress', (event) => {
        if (event.payload.stage !== 'bluetooth') return;
        state.transferProgress = {
          stage: event.payload.stage,
          processed: event.payload.processed,
          total: event.payload.total,
          percent: Math.min(event.payload.percent, 100),
        };
      });
      try {
        await api.transferApi.btSend(addr.trim(), info.path);
      } finally {
        unlisten();
      }
      state.btStatus = 'complete';
      toast.success(m['transfer.bt.sent']({ name: info.file_name }));
    } catch (e) {
      state.btError = String(e);
      state.btStatus = 'error';
      toast.error(String(e));
    } finally {
      state.btBusy = false;
    }
  }

  // Direct share over the internet (P2P): silent export + WebRTC sender peer.
  async function exportAndStartP2P(includeCredentials: boolean, lightweight: boolean) {
    if (!state.selectedProjectId || state.p2pBusy) return;
    state.p2pBusy = true;
    state.p2pStatus = 'exporting';
    state.p2pError = '';
    try {
      const projectId = state.selectedProjectId;
      const name = `open-crawler-${projectId}.ocproj`;
      const info = await api.transferApi.exportPackage(
        [projectId],
        name,
        lightweight,
        includeCredentials,
        false,
        true
      );
      state.lastPackage = info;
      p2pSender = startP2PSender(
        info.path,
        info.file_name,
        info.size_bytes,
        (status) => (state.p2pStatus = status),
        (p) => (state.p2pProgress = p),
        (message) => {
          state.p2pError = message;
          state.p2pStatus = 'error';
          toast.error(message);
        }
      );
      state.p2pPeerId = p2pSender.id;
      state.p2pStatus = 'ready';
      // Also serve the `/receive` browser page on the LAN so receivers on the
      // same network don't need the app.
      try {
        const transfer = await api.transferApi.startTransferServer(info.path, 15);
        state.activeTransfer = transfer;
      } catch (e) {
        console.warn('P2P: could not start LAN receiver page:', e);
      }
      toast.success(m['transfer.exported']({ name: info.file_name }));
    } catch (e) {
      state.p2pError = String(e);
      state.p2pStatus = 'error';
      toast.error(String(e));
    } finally {
      state.p2pBusy = false;
    }
  }

  function stopP2P() {
    p2pSender?.dispose();
    p2pSender = null;
    state.p2pPeerId = '';
    state.p2pStatus = '';
    state.p2pError = '';
    state.p2pProgress = { transferred: 0, total: 0, percent: 0 };
    if (state.activeTransfer) {
      void api.transferApi.stopTransferServer().catch(() => {});
      state.activeTransfer = null;
    }
  }

  // Receiver side: connects to the sender's peer id, writes the incoming
  // package to a temp file and imports it using the chosen conflict mode.
  async function receiveP2P(code: string, mode: ImportConflictMode) {
    if (state.p2pBusy) return;
    const peerId = parseP2PCode(code);
    if (!peerId) {
      toast.error(m['transfer.p2p.invalid_code']());
      return;
    }
    state.p2pBusy = true;
    state.p2pStatus = 'connecting';
    state.p2pError = '';
    let dest = '';
    try {
      const { appDataDir, join } = await import('@tauri-apps/api/path');
      const fs = await import('@tauri-apps/plugin-fs');
      const dir = await appDataDir();
      const transfersDir = await join(dir, 'transfers');
      await fs.mkdir(transfersDir, { recursive: true });
      dest = await join(transfersDir, `p2p-incoming-${Date.now()}.ocproj`);
      await receivePeerPackage(
        peerId,
        dest,
        (s) => (state.p2pStatus = s),
        (p) => (state.p2pProgress = p),
        (message) => {
          state.p2pError = message;
          state.p2pStatus = 'error';
          toast.error(message);
        }
      );
      await importPackage(dest, mode);
    } catch (e) {
      state.p2pError = String(e);
      state.p2pStatus = 'error';
      toast.error(String(e));
    } finally {
      if (dest) {
        try {
          const fs = await import('@tauri-apps/plugin-fs');
          await fs.remove(dest);
        } catch {
          // best-effort cleanup
        }
      }
      state.p2pBusy = false;
      state.p2pStatus = '';
      state.p2pError = '';
      state.p2pProgress = { transferred: 0, total: 0, percent: 0 };
    }
  }

  async function importPackage(filePath: string, mode: ImportConflictMode) {
    if (state.transferBusy) return { imported: [], skipped: [], warnings: [] } as ImportSummary;
    state.transferBusy = true;
    try {
      const summary = await api.transferApi.importPackage(filePath, mode);
      await loadProjects();
      if (summary.imported.length > 0 && !state.selectedProjectId) {
        selectProject(summary.imported[0].id);
      }
      toast.success(
        m['transfer.imported']({ count: summary.imported.length, skipped: summary.skipped.length })
      );
      for (const w of summary.warnings) toast.warning(w);
      return summary;
    } catch (e) {
      state.error = String(e);
      toast.error(String(e));
      return { imported: [], skipped: [], warnings: [] } as ImportSummary;
    } finally {
      state.transferBusy = false;
    }
  }

  // Imports a package received through the system share sheet (Bluetooth /
  // Nearby / files) on mobile. Returns null when the queue is empty.
  async function importSharedIntent(mode: ImportConflictMode) {
    if (state.shareImporting) return null;
    state.shareImporting = true;
    try {
      const summary = await api.transferApi.importSharedIntent(mode);
      if (!summary) return null;
      await loadProjects();
      if (summary.imported.length > 0 && !state.selectedProjectId) {
        selectProject(summary.imported[0].id);
      }
      toast.success(
        m['transfer.imported']({ count: summary.imported.length, skipped: summary.skipped.length })
      );
      for (const w of summary.warnings) toast.warning(w);
      return summary;
    } catch (e) {
      state.error = String(e);
      toast.error(String(e));
      return null;
    } finally {
      state.shareImporting = false;
    }
  }

  // Polls the share-target queue once per focus (and once at startup). The
  // intent may be queued before the webview is ready, so this drains any that
  // arrived while the app was closed or in the background.
  function checkIncomingShare() {
    if (!state.initialized || state.shareChecked) return;
    state.shareChecked = true;
    importSharedIntent('skip');
  }

  async function startTransferServer(filePath: string, minutes?: number) {
    if (state.transferBusy) return null;
    state.transferBusy = true;
    try {
      const info = await api.transferApi.startTransferServer(filePath, minutes);
      state.activeTransfer = info;
      return info;
    } catch (e) {
      state.error = String(e);
      toast.error(String(e));
      return null;
    } finally {
      state.transferBusy = false;
    }
  }

  async function stopTransferServer() {
    try {
      await api.transferApi.stopTransferServer();
      state.activeTransfer = null;
    } catch (e) {
      state.error = String(e);
      toast.error(String(e));
    }
  }

  // Re-open the WiFi server that was active when the app closed, so a transfer
  // still works after a restart (no need to export again).
  async function restoreTransferServer() {
    if (!state.lastPackage || !persistedActiveTransfer) return;
    try {
      const minutes = Math.max(1, Math.round(persistedActiveTransfer.expires_in_secs / 60));
      const info = await api.transferApi.startTransferServer(state.lastPackage.path, minutes);
      state.activeTransfer = info;
    } catch (e) {
      console.warn('[Transfer] Could not restore active WiFi server:', e);
    }
  }

  async function downloadTransfer(url: string, dest: string) {
    if (state.transferBusy) return;
    state.transferBusy = true;
    state.transferProgress = { stage: 'download', processed: 0, total: 0, percent: 0 };
    const unlisten = await listen<TransferProgress>('transfer-progress', (event) => {
      state.transferProgress = {
        stage: event.payload.stage,
        processed: event.payload.processed,
        total: event.payload.total,
        percent: Math.min(event.payload.percent, 100),
      };
    });
    try {
      await api.transferApi.downloadTransfer(url, dest);
    } finally {
      unlisten();
      state.transferBusy = false;
    }
  }

  // ==================== SELECTORS ====================

  function getSelectedProject() {
    return state.projects.optimistic.find((p) => p.id === state.selectedProjectId);
  }

  state.getSelectedProject = getSelectedProject;
  state.totalPages = () => Math.ceil(state.results.total / state.pageSize);
  state.formatDuration = formatDuration;
  state.selectProject = selectProject;
  state.createProject = createProject;
  state.startRename = startRename;
  state.confirmRename = confirmRename;
  state.cancelRename = cancelRename;
  state.requestDelete = requestDelete;
  state.closeDelete = closeDelete;
  state.deleteProject = deleteProject;
  state.handleStartCrawl = handleStartCrawl;
  state.startCrawl = startCrawl;
  state.stopCrawl = stopCrawl;
  state.loadResults = loadResults;
  state.handleFilterChange = handleFilterChange;
  state.goToPage = goToPage;
  state.changePageSize = changePageSize;
  state.handleFilterIssueType = handleFilterIssueType;
  state.openDetail = openDetail;
  state.onSearchInput = onSearchInput;
  state.exportFull = exportFull;
  state.exportPackage = exportPackage;
  state.exportAndStartWifi = exportAndStartWifi;
  state.exportAndShare = exportAndShare;
  state.exportAndStartP2P = exportAndStartP2P;
  state.stopP2P = stopP2P;
  state.receiveP2P = receiveP2P;
  state.sendViaObex = sendViaObex;
  state.importPackage = importPackage;
  state.importSharedIntent = importSharedIntent;
  state.startTransferServer = startTransferServer;
  state.stopTransferServer = stopTransferServer;
  state.downloadTransfer = downloadTransfer;

  return state;
}

export function setAppShell() {
  return setContext(APP_SHELL_KEY, createAppShell());
}

export function getAppShell() {
  return getContext<AppShell>(APP_SHELL_KEY);
}
