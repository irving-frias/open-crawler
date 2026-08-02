<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import { toast } from 'svelte-sonner';
  import ResultsTable from '$lib/components/ResultsTable.svelte';
  import FilterBar, { type FilterState } from '$lib/components/FilterBar.svelte';
import { m } from '$lib/paraglide/messages.js';
import { notify } from '$lib/utils';
import { Settings, Plus, Pencil, Trash2, X, ChevronLeft, ChevronRight, ChevronsLeft, ChevronsRight, RefreshCw, Download, SearchX, FileSpreadsheet, FileText, ChevronDown } from 'lucide-svelte';
  import { Button } from '$lib/components/ui/button/index.js';
  import { Input } from '$lib/components/ui/input/index.js';
  import { Label } from '$lib/components/ui/label/index.js';
  import { Checkbox } from '$lib/components/ui/checkbox/index.js';
  import * as Select from '$lib/components/ui/select/index.js';
  import { Progress } from '$lib/components/ui/progress/index.js';
  import { Badge } from '$lib/components/ui/badge/index.js';
  import { Skeleton } from '$lib/components/ui/skeleton/index.js';
  import * as Tabs from '$lib/components/ui/tabs/index.js';
  import * as AlertDialog from '$lib/components/ui/alert-dialog/index.js';
  import * as Popover from '$lib/components/ui/popover/index.js';

  let PageDetailPanel = $state<typeof import('$lib/components/PageDetailPanel.svelte').default | null>(null);
  let SiteTree = $state<typeof import('$lib/components/SiteTree.svelte').default | null>(null);
  let SemanticDashboard = $state<typeof import('$lib/components/SemanticDashboard.svelte').default | null>(null);
  let SettingsModal = $state<typeof import('$lib/components/SettingsModal.svelte').default | null>(null);

  $effect(() => {
    if (detailPageId && !PageDetailPanel) {
      import('$lib/components/PageDetailPanel.svelte').then(m => PageDetailPanel = m.default);
    }
  });

  $effect(() => {
    if (activeTab === 'dashboard' && !SemanticDashboard) {
      import('$lib/components/SemanticDashboard.svelte').then(m => SemanticDashboard = m.default);
    }
    if (activeTab === 'site_tree' && !SiteTree) {
      import('$lib/components/SiteTree.svelte').then(m => SiteTree = m.default);
    }
  });

  $effect(() => {
    if (settingsModalOpen && !SettingsModal) {
      import('$lib/components/SettingsModal.svelte').then(m => SettingsModal = m.default);
    }
  });

  // Project state
  let projects = $state<any[]>([]);
  let selectedProjectId = $state('');
  let newProjectName = $state('');
  let renamingProjectId = $state('');
  let renamingName = $state('');

  // Crawl config
  let seedUrl = $state('');
  let maxDepth = $state(10);
  let respectRobots = $state(true);
  let renderJs = $state(false);
  let checkSitemap = $state(true);
  let checkSemantics = $state(true);
  let maxCrawlTime = $state(3600);
  let proxyUrl = $state('');
  let proxyUser = $state('');
  let proxyPass = $state('');

  let seedUrlsByProject = $state<Record<string, string>>({});

  // Crawl state
  let status = $state<'idle' | 'running' | 'paused' | 'completed' | 'error'>('idle');
  let progress = $state({ crawled: 0, queued: 0, current: '', errors: 0 });
  let results = $state<any>({ items: [], total: 0, page: 1, page_size: 50 });
  let error = $state('');
  let sitemapInfo = $state('');

  // Filters
  let activeFilters = $state<FilterState>({
    statusCodes: [],
    severities: [],
    depth: undefined,
    missingTitle: false,
    duplicateTitle: false,
    noindexOnly: false,
    is404: false,
  });

  // Pagination
  let currentPage = $state(1);
  let pageSize = $state(50);
  let pageSizeSelect = $state('50');
  let totalPages = $derived(Math.ceil(results.total / pageSize));

  const progressPct = $derived(
    progress.crawled > 0
      ? Math.min((progress.crawled / (progress.crawled + progress.queued)) * 100, 100)
      : 0
  );

  $effect(() => {
    pageSizeSelect = String(pageSize);
  });

  // Resume state
  let resumableInfo = $state<any>(null);
  let showResumeDialog = $state(false);

  // Streamed count during live crawl
  let streamedCount = $state(0);

  // Export progress
  let exportProgress = $state<{ running: boolean; percent: number; stage: string }>({
    running: false,
    percent: 0,
    stage: '',
  });
  let exportHideTimer: ReturnType<typeof setTimeout> | null = null;

  // Expanded issue row
  let expandedIssueUrl = $state('');

  // Tab + filter state
  let activeTab = $state<'results' | 'dashboard' | 'site_tree'>('results');
  let semanticFilter = $state('');

  // Detail panel
  let detailPageId = $state('');

  // Search
  let searchQuery = $state('');
  let debouncedSearch = $state('');
  let searchDebounceTimer: ReturnType<typeof setTimeout> | null = null;

  // Results loading state
  let resultsLoading = $state(false);

  function onSearchInput(query: string) {
    searchQuery = query;
    if (searchDebounceTimer) clearTimeout(searchDebounceTimer);
    searchDebounceTimer = setTimeout(() => {
      debouncedSearch = query;
      currentPage = 1;
      loadResults(1);
    }, 400);
  }

  // Settings modal
  let settingsModalOpen = $state(false);

  // Notifications
  let notificationsEnabled = $state(true);

  // Delete confirmation
  let deletePendingId = $state<string | null>(null);
  let deleteDialogOpen = $state(false);

  let unlistenFns: UnlistenFn[] = [];

  // ==================== PROJECTS ====================

  async function loadProjects() {
    try {
      const data = await invoke<any[]>('list_projects');
      projects = data;
      if (data.length > 0 && !selectedProjectId) {
        selectProject(data[0].id);
      }
    } catch (e) {
      console.error('[Projects] Failed to load:', e);
    }
  }

  async function loadSettings() {
    try {
      const settings = await invoke<Record<string, string>>('get_settings');
      if (settings.page_size) {
        pageSize = parseInt(settings.page_size, 10);
      }
      if (settings.notifications_enabled !== undefined) {
        notificationsEnabled = settings.notifications_enabled === 'true';
      }
    } catch (e) {
      // Settings may not exist yet, use defaults
    }
  }

  function selectProject(id: string) {
    if (selectedProjectId && selectedProjectId !== id) {
      seedUrlsByProject[selectedProjectId] = seedUrl;
    }
    selectedProjectId = id;
    seedUrl = seedUrlsByProject[id] ?? '';
    status = 'idle';
    progress = { crawled: 0, queued: 0, current: '', errors: 0 };
    results = { items: [], total: 0, page: 1, page_size: 50 };
    currentPage = 1;
    error = '';
    sitemapInfo = '';
    resumableInfo = null;
    showResumeDialog = false;
    streamedCount = 0;
    expandedIssueUrl = '';
    semanticFilter = '';
    activeTab = 'results';
    detailPageId = '';
    loadResults(1);
    checkResumable();
    checkRunningCrawl();
  }

  async function checkRunningCrawl() {
    if (!selectedProjectId) return;
    try {
      const runningIds = await invoke<string[]>('get_running_crawls');
      if (runningIds.includes(selectedProjectId)) {
        status = 'running';
        // Poll progress while running
        pollCrawlProgress();
      }
    } catch (e) {
      console.error('[Crawler] Failed to check running crawls:', e);
    }
  }

  let progressPollTimer: any = null;

  async function pollCrawlProgress() {
    if (progressPollTimer) clearInterval(progressPollTimer);
    progressPollTimer = setInterval(async () => {
      if (status !== 'running' || !selectedProjectId) {
        clearInterval(progressPollTimer);
        progressPollTimer = null;
        return;
      }
      try {
        const p = await invoke<any | null>('get_crawl_status', {
          projectId: selectedProjectId,
        });
        if (p) {
          progress.crawled = p.urls_crawled;
          progress.queued = p.urls_queued;
          progress.current = p.current_url;
          progress.errors = p.errors;
        } else {
          // Crawl finished while we were away
          status = 'completed';
          clearInterval(progressPollTimer);
          progressPollTimer = null;
          loadResults(1);
        }
      } catch (e) {
        console.error('[Crawler] Failed to poll progress:', e);
      }
    }, 1000);
  }

  async function checkResumable() {
    if (!selectedProjectId) return;
    try {
      const info = await invoke<any | null>('check_resumable_crawl', {
        projectId: selectedProjectId,
      });
      resumableInfo = info;
    } catch (e) {
      console.error('[Crawler] Failed to check resumable:', e);
    }
  }

  async function createProject() {
    if (!newProjectName.trim()) return;
    try {
      const project = await invoke<any>('create_project', {
        request: { name: newProjectName.trim() },
      });
      newProjectName = '';
      await loadProjects();
      selectProject(project.id);
    } catch (e) {
      error = String(e);
    }
  }

  async function startRename(id: string, currentName: string) {
    renamingProjectId = id;
    renamingName = currentName;
  }

  async function confirmRename() {
    if (!renamingName.trim()) return;
    try {
      await invoke('rename_project', {
        request: { id: renamingProjectId, name: renamingName.trim() },
      });
      renamingProjectId = '';
      renamingName = '';
      await loadProjects();
    } catch (e) {
      error = String(e);
    }
  }

  async function cancelRename() {
    renamingProjectId = '';
    renamingName = '';
  }

  function requestDelete(id: string) {
    deletePendingId = id;
    deleteDialogOpen = true;
  }

  function closeDelete() {
    deleteDialogOpen = false;
    deletePendingId = null;
  }

  async function deleteProject(id: string) {
    closeDelete();
    try {
      await invoke('delete_project', { id });
      if (selectedProjectId === id) {
        selectedProjectId = '';
        results = { items: [], total: 0, page: 1, page_size: 50 };
      }
      await loadProjects();
    } catch (e) {
      error = String(e);
    }
  }

  // ==================== CRAWL ====================

  async function setupListeners() {
    const un0 = await listen<any>('crawl-started', (event) => {
      console.log('[Crawler] crawl-started:', event.payload);
      if (notificationsEnabled) {
        notify(m['notifications.crawl_started'](), m['notifications.crawl_started_desc']());
      }
    });
    unlistenFns.push(un0);

    const un1 = await listen<any>('crawl-progress', (event) => {
      const p = event.payload;
      if (p.project_id !== selectedProjectId) return;
      progress.crawled = p.urls_crawled;
      progress.queued = p.urls_queued;
      progress.current = p.current_url;
      progress.errors = p.errors;
    });
    unlistenFns.push(un1);

    // Real-time batch results - just count, don't mix with paginated results
    const un_batch = await listen<any>('crawl-batch', (event) => {
      const p = event.payload;
      if (p.project_id !== selectedProjectId) return;
      if (p.items && p.items.length > 0) {
        streamedCount += p.items.length;
      }
    });
    unlistenFns.push(un_batch);

    const un2 = await listen<any>('crawl-complete', (event) => {
      const p = event.payload;
      if (p.project_id !== selectedProjectId) return;
      status = 'completed';
      resumableInfo = null;
      streamedCount = 0;
      loadResults(1);
      if (notificationsEnabled) {
        notify(m['notifications.crawl_complete'](), m['notifications.crawl_complete_desc']());
      }
    });
    unlistenFns.push(un2);

    const un3 = await listen<any>('crawl-error', (event) => {
      const p = event.payload;
      if (p.project_id !== selectedProjectId) return;
      status = 'error';
      error = p.error || String(p);
      if (notificationsEnabled) {
        notify(m['notifications.crawl_error'](), p.error || String(p));
      }
    });
    unlistenFns.push(un3);

    const un4 = await listen<any>('crawl-stopped', (event) => {
      const p = event.payload;
      if (p.project_id !== selectedProjectId) return;
      status = 'idle';
      streamedCount = 0;
      checkResumable();
      if (notificationsEnabled) {
        notify(m['notifications.crawl_stopped'](), m['notifications.crawl_stopped_desc']());
      }
    });
    unlistenFns.push(un4);

    const un5 = await listen<any>('sitemap-discovered', (event) => {
      const p = event.payload;
      sitemapInfo = p.fallback
        ? m['sitemap.fallback']()
        : m['sitemap.found']({
            count: String(p.urls_found),
            sitemaps: String(p.sitemaps_checked),
          });
    });
    unlistenFns.push(un5);
  }

  $effect(() => {
    setupListeners();
    loadProjects();
    loadSettings();
    return () => {
      unlistenFns.forEach((fn) => fn());
      unlistenFns = [];
      if (progressPollTimer) clearInterval(progressPollTimer);
    };
  });

  async function startCrawl(resume: boolean = false) {
    if (!selectedProjectId) return;
    try {
      status = 'running';
      error = '';
      sitemapInfo = '';
      showResumeDialog = false;
      streamedCount = 0;
      currentPage = 1;
      results = { items: [], total: 0, page: 1, page_size: pageSize };
      await invoke('start_crawl', {
        projectId: selectedProjectId,
        config: {
          seed_urls: [seedUrl],
          max_depth: maxDepth,
          respect_robots: respectRobots,
          render_js: renderJs,
          check_sitemap: checkSitemap,
          check_semantics: checkSemantics,
          max_crawl_time_secs: maxCrawlTime,
          proxy: proxyUrl.trim()
            ? {
                url: proxyUrl.trim(),
                username: proxyUser.trim() || null,
                password: proxyPass || null,
              }
            : null,
        },
      });
    } catch (e) {
      status = 'error';
      error = String(e);
      toast.error(String(e));
    }
  }

  function handleStartCrawl() {
    if (resumableInfo) {
      showResumeDialog = true;
    } else {
      startCrawl(false);
    }
  }

  async function stopCrawl() {
    if (!selectedProjectId) return;
    try {
      await invoke('stop_crawl', { projectId: selectedProjectId });
      toast.info(m["progress.title"]() + ': stopped');
    } catch (e) {
      error = String(e);
      toast.error(String(e));
    }
  }

  async function loadResults(page: number = 1) {
    if (!selectedProjectId) return;
    try {
      currentPage = page;
      resultsLoading = true;
      const data = await invoke('get_results', {
        projectId: selectedProjectId,
        page: currentPage,
        pageSize: pageSize,
        semanticIssueType: semanticFilter || null,
        search: debouncedSearch || null,
        statusFilter: activeFilters.statusCodes.length > 0 ? activeFilters.statusCodes : null,
        severityFilter: activeFilters.severities.length > 0 ? activeFilters.severities : null,
        depthFilter: activeFilters.depth,
        missingTitle: activeFilters.missingTitle || null,
        duplicateTitle: activeFilters.duplicateTitle || null,
        noindexOnly: activeFilters.noindexOnly || null,
        is404: activeFilters.is404 || null,
      });
      results = data;
    } catch (e) {
      console.error('[Crawler] Failed to load results:', e);
    } finally {
      resultsLoading = false;
    }
  }

  function handleFilterChange(filters: FilterState) {
    activeFilters = filters;
    loadResults(1);
  }

  function goToPage(page: number) {
    if (page < 1 || page > totalPages) return;
    loadResults(page);
  }

  function changePageSize(newSize: number) {
    pageSize = newSize;
    currentPage = 1;
    loadResults(1);
  }

  function handleFilterIssueType(issueType: string | null) {
    semanticFilter = issueType || '';
    currentPage = 1;
    loadResults(1);
  }

  function openDetail(pageId: string) {
    detailPageId = pageId;
  }

  function getPageNumbers(): (number | '...')[] {
    const pages: (number | '...')[] = [];
    const total = totalPages;
    const current = currentPage;

    if (total <= 7) {
      for (let i = 1; i <= total; i++) pages.push(i);
    } else {
      pages.push(1);
      if (current > 3) pages.push('...');
      const start = Math.max(2, current - 1);
      const end = Math.min(total - 1, current + 1);
      for (let i = start; i <= end; i++) pages.push(i);
      if (current < total - 2) pages.push('...');
      pages.push(total);
    }

    return pages;
  }

  async function exportFull(format: 'xlsx' | 'csv') {
    if (!selectedProjectId || exportProgress.running) return;
    try {
      const mobile = await invoke<boolean>('is_mobile');
      const ext = format === 'xlsx' ? 'xlsx' : 'csv';
      const defaultName = `crawl-results-${selectedProjectId}.${ext}`;

      let path: string;
      if (mobile) {
        path = defaultName;
      } else {
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
        path = picked.toLowerCase().endsWith(`.${ext}`) ? picked : `${picked}.${ext}`;
      }

      exportProgress = { running: true, percent: 0, stage: '' };
      const unlisten = await listen<{ stage: string; percent: number }>('export-progress', (event) => {
        exportProgress = {
          running: true,
          percent: Math.min(event.payload.percent, 100),
          stage: event.payload.stage,
        };
      });

      try {
        await invoke('export_full', {
          projectId: selectedProjectId,
          filePath: path,
          format,
        });
        exportProgress = { running: false, percent: 100, stage: '' };
        toast.success(mobile ? m['export.shared']() : `Exported to ${path.split(/[/\\]/).pop()}`);
        if (notificationsEnabled) {
          notify(m['notifications.export_complete'](), m['notifications.export_complete_desc']());
        }
        if (exportHideTimer) clearTimeout(exportHideTimer);
        exportHideTimer = setTimeout(() => {
          exportProgress = { running: false, percent: 0, stage: '' };
        }, 1500);
      } finally {
        unlisten();
      }
    } catch (e) {
      exportProgress = { running: false, percent: 0, stage: '' };
      error = String(e);
      toast.error(String(e));
    }
  }

  function getSelectedProject() {
    return projects.find((p) => p.id === selectedProjectId);
  }

  function formatDuration(secs: number): string {
    const h = Math.floor(secs / 3600);
    const m = Math.floor((secs % 3600) / 60);
    const s = secs % 60;
    if (h > 0) return `${h}h ${m}m ${s}s`;
    if (m > 0) return `${m}m ${s}s`;
    return `${s}s`;
  }
</script>

<div class="app-layout">
  <!-- Header -->
  <header class="app-header">
    <div class="header-left">
      <h1 class="logo">{m["app.title"]()}</h1>
      <Button
        variant="ghost"
        size="xs"
        class="btn-settings"
        onclick={() => settingsModalOpen = true}
        aria-label={m["settings.title"]()}
        title={m["settings.title"]()}
      >
        <Settings class="size-4" />
      </Button>
    </div>

    <div class="header-center">
      <div class="project-create">
        <Input
          type="text"
          bind:value={newProjectName}
          placeholder={m["sidebar.new_project_placeholder"]()}
          onkeydown={(e) => e.key === 'Enter' && createProject()}
        />
        <Button
          size="icon"
          onclick={createProject}
          disabled={!newProjectName.trim()}
          aria-label={m["sidebar.new_project_placeholder"]()}
          title={m["sidebar.new_project_placeholder"]()}
        >
          <Plus class="size-4" />
        </Button>
      </div>
    </div>

    <nav class="project-list-header">
      {#each projects as project (project.id)}
        <div
          class="project-chip"
          class:selected={project.id === selectedProjectId}
          role="button"
          tabindex="0"
          onclick={() => selectProject(project.id)}
          onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') selectProject(project.id); }}
        >
          {#if renamingProjectId === project.id}
            <Input
              type="text"
              bind:value={renamingName}
              onkeydown={(e) => {
                if (e.key === 'Enter') confirmRename();
                if (e.key === 'Escape') cancelRename();
              }}
              onblur={confirmRename}
              class="h-8 px-2 py-1 text-sm"
              onclick={(e) => e.stopPropagation()}
            />
          {:else}
            <span class="project-avatar">{project.name.trim().charAt(0).toUpperCase() || '?'}</span>
            <span class="project-name">{project.name}</span>
            <div class="project-actions">
              <Button
                variant="ghost"
                size="xs"
                class="btn-mini"
                title={m["sidebar.rename"]()}
                aria-label={m["sidebar.rename"]()}
                onclick={(e) => {
                  e.stopPropagation();
                  startRename(project.id, project.name);
                }}
              >
                <Pencil class="size-3" />
              </Button>
              <Button
                variant="ghost"
                size="xs"
                class="hover:bg-destructive hover:text-white"
                title={m["sidebar.delete"]()}
                aria-label={m["sidebar.delete"]()}
                onclick={(e) => {
                  e.stopPropagation();
                  requestDelete(project.id);
                }}
              >
                <Trash2 class="size-3" />
              </Button>
            </div>
          {/if}
        </div>
      {/each}
      {#if projects.length === 0}
        <div class="empty-projects">{m["sidebar.no_projects"]()}</div>
      {/if}
    </nav>
  </header>

  <!-- Main content -->
  <main class="main-content">
    {#if !selectedProjectId}
      <div class="no-project">
        <SearchX class="no-project-icon" />
        <h2>{m["app.select_project"]()}</h2>
        <p>{m["app.select_project_hint"]()}</p>
      </div>
    {:else}
      <div class="project-header">
        <h2>{getSelectedProject()?.name}</h2>
      </div>

      {#if error}
        <div class="error">{error}</div>
      {/if}

      <!-- Resume Dialog -->
      {#if showResumeDialog && resumableInfo}
        <AlertDialog.Root bind:open={showResumeDialog}>
          <AlertDialog.Content class="max-w-md">
            <AlertDialog.Header>
              <AlertDialog.Title>{m["resume.title"]()}</AlertDialog.Title>
              <AlertDialog.Description>
                {m["resume.found"]({ pages: resumableInfo.pages_crawled, urls: resumableInfo.queue_remaining, time: formatDuration(resumableInfo.elapsed_secs) })}
              </AlertDialog.Description>
            </AlertDialog.Header>
            <AlertDialog.Footer>
              <Button variant="outline" onclick={() => { showResumeDialog = false; startCrawl(false); }}>
                {m["resume.fresh_btn"]()}
              </Button>
              <Button variant="outline" onclick={() => showResumeDialog = false}>
                {m["resume.cancel"]()}
              </Button>
              <Button onclick={() => startCrawl(true)}>
                {m["resume.resume_btn"]()}
              </Button>
            </AlertDialog.Footer>
          </AlertDialog.Content>
        </AlertDialog.Root>
      {/if}

      <section class="config-section">
        <h2>{m["config.title"]()}</h2>

        <div class="form-group">
          <Label for="seed">{m["config.seed_url"]()}</Label>
          <Input
            id="seed"
            type="url"
            bind:value={seedUrl}
            placeholder={m["config.seed_url_placeholder"]()}
            disabled={status === 'running'}
          />
        </div>

        <div class="form-row">
          <div class="form-group">
            <Label for="maxDepth">{m["config.max_depth"]()}</Label>
            <Input id="maxDepth" type="number" bind:value={maxDepth} min="1" max="100" disabled={status === 'running'} />
          </div>
          <div class="form-group">
            <Label for="maxTime">{m["config.time_limit"]()}</Label>
            <Input id="maxTime" type="number" bind:value={maxCrawlTime} min="0" max="86400" disabled={status === 'running'} />
          </div>
        </div>

        <div class="form-group">
          <Label for="proxyUrl">{m["config.proxy_url"]()}</Label>
          <Input
            id="proxyUrl"
            type="text"
            bind:value={proxyUrl}
            placeholder={m["config.proxy_url_placeholder"]()}
            disabled={status === 'running'}
          />
        </div>
        {#if proxyUrl.trim()}
          <div class="form-row">
            <div class="form-group">
              <Label for="proxyUser">{m["config.proxy_user"]()}</Label>
              <Input id="proxyUser" type="text" bind:value={proxyUser} autocomplete="off" disabled={status === 'running'} />
            </div>
            <div class="form-group">
              <Label for="proxyPass">{m["config.proxy_pass"]()}</Label>
              <Input id="proxyPass" type="password" bind:value={proxyPass} autocomplete="off" disabled={status === 'running'} />
            </div>
          </div>
        {/if}

        <div class="form-group checkboxes">
          <label class="checkbox-label">
            <Checkbox bind:checked={respectRobots} disabled={status === 'running'} />
            <span>{m["config.respect_robots"]()}</span>
          </label>
          <label class="checkbox-label">
            <Checkbox bind:checked={renderJs} disabled={status === 'running'} />
            <span>{m["config.render_js"]()}</span>
          </label>
          <label class="checkbox-label">
            <Checkbox bind:checked={checkSitemap} disabled={status === 'running'} />
            <span>{m["config.check_sitemap"]()}</span>
          </label>
          <label class="checkbox-label">
            <Checkbox bind:checked={checkSemantics} disabled={status === 'running'} />
            <span>{m["config.check_semantics"]()}</span>
          </label>
        </div>

        <div class="actions">
          {#if status === 'idle' || status === 'completed' || status === 'error'}
            <Button onclick={handleStartCrawl} disabled={!seedUrl}>
              {resumableInfo ? m["config.resume"]() : m["config.start"]()}
            </Button>
          {:else if status === 'running'}
            <Button variant="destructive" onclick={stopCrawl}>{m["config.stop"]()}</Button>
          {/if}
          <Button variant="outline" class="gap-1.5" onclick={() => loadResults(currentPage)}>
            <RefreshCw class="size-4" />
            {m["config.refresh"]()}
          </Button>
          {#if results.items.length > 0}
            <Popover.Root>
              <Popover.Trigger>
                {#snippet child({ props })}
                  <Button
                    variant="outline"
                    class="gap-1.5"
                    {...props}
                    disabled={exportProgress.running}
                  >
                    <Download class="size-4" />
                    {m["settings.export"]()}
                    <ChevronDown class="size-3.5" />
                  </Button>
                {/snippet}
              </Popover.Trigger>
              <Popover.Content align="end" class="w-44 p-1">
                <Button
                  variant="ghost"
                  class="w-full justify-start gap-2"
                  disabled={exportProgress.running}
                  onclick={() => exportFull('xlsx')}
                >
                  <FileSpreadsheet class="size-4" />
                  {m["export.xlsx"]()}
                </Button>
                <Button
                  variant="ghost"
                  class="w-full justify-start gap-2"
                  disabled={exportProgress.running}
                  onclick={() => exportFull('csv')}
                >
                  <FileText class="size-4" />
                  {m["export.csv"]()}
                </Button>
              </Popover.Content>
            </Popover.Root>
          {/if}
        </div>

        {#if resumableInfo && status === 'idle'}
          <div class="resume-hint">
            {m["resume.hint"]({ pages: resumableInfo.pages_crawled, urls: resumableInfo.queue_remaining })}
          </div>
        {/if}
      </section>

      {#if status === 'running'}
        <section class="progress-section">
          <div class="progress-head">
            <h2>{m["progress.title"]()}</h2>
            <span class="progress-pct">{Math.round(progressPct)}%</span>
          </div>
          <Progress
            value={progressPct}
            class="h-2 transition-all duration-300"
          />
          <div class="progress-stats">
            <span>{m["progress.crawled"]({ count: progress.crawled.toString() })}</span>
            <span>{m["progress.queued"]({ count: progress.queued.toString() })}</span>
            <span>{m["progress.errors"]({ count: progress.errors.toString() })}</span>
            <span class="current-url">{progress.current || '...'}</span>
          </div>
          {#if streamedCount > 0}
            <div class="streamed-info">
              {m["progress.streamed"]({ count: streamedCount.toString() })}
            </div>
          {/if}
        </section>
      {/if}

      {#if sitemapInfo}
        <div class="sitemap-info">{sitemapInfo}</div>
      {/if}

      {#if results.items.length > 0 || streamedCount > 0 || resultsLoading}
        <section class="results-section">
          <Tabs.Root bind:value={activeTab} class="mb-4">
            <Tabs.List>
              <Tabs.Trigger value="results">
                {m["tabs.results"]({ count: results.total.toLocaleString() })}
              </Tabs.Trigger>
              <Tabs.Trigger value="dashboard">
                {m["tabs.issues_dashboard"]()}
              </Tabs.Trigger>
              <Tabs.Trigger value="site_tree">
                {m["tabs.site_tree"]()}
              </Tabs.Trigger>
            </Tabs.List>
          </Tabs.Root>

          {#if activeTab === 'results'}
            <div class="results-toolbar">
              <div class="page-size-selector">
                <Label for="pageSizeSelect">{m["results.page_size_show"]()}</Label>
                <Select.Root
                  type="single"
                  bind:value={pageSizeSelect}
                  onValueChange={(v) => {
                    if (v) changePageSize(parseInt(v, 10));
                  }}
                >
                  <Select.Trigger id="pageSizeSelect" class="w-20">
                    {pageSizeSelect}
                  </Select.Trigger>
                  <Select.Content>
                    {#each ['25', '50', '100', '200'] as size (size)}
                      <Select.Item value={size}>{size}</Select.Item>
                    {/each}
                  </Select.Content>
                </Select.Root>
              </div>
              {#if semanticFilter}
                <Badge variant="secondary" class="gap-1.5 px-3 py-1">
                  {m["results.filtered_by"]({ type: semanticFilter.replace(/_/g, ' ') })}
                  <Button
                    variant="ghost"
                    size="xs"
                    class="btn-clear-filter size-5"
                    onclick={() => handleFilterIssueType(null)}
                    aria-label={m["results.clear_filter"]()}
                    title={m["results.clear_filter"]()}
                  >
                    <X class="size-3" />
                  </Button>
                </Badge>
              {/if}
            </div>

            <FilterBar
              items={results.items}
              totalResults={results.total}
              onFilter={handleFilterChange}
            />

            {#if resultsLoading}
              <div class="results-skeleton">
                <Skeleton class="h-10 w-full" />
                <Skeleton class="h-10 w-full" />
                <Skeleton class="h-10 w-full" />
                <Skeleton class="h-10 w-full" />
                <Skeleton class="h-10 w-3/4" />
              </div>
            {:else}
              <ResultsTable
                bind:expandedUrl={expandedIssueUrl}
                items={results.items}
                onDetail={openDetail}
                searchQuery={debouncedSearch}
                onSearch={onSearchInput}
              />

              {#if totalPages > 1}
                <div class="pagination">
                  <span class="pagination-info">
                    {m["results.showing"]({ from: ((currentPage - 1) * pageSize + 1).toString(), to: Math.min(currentPage * pageSize, results.total).toString(), total: results.total.toLocaleString() })}
                  </span>
                  <div class="pagination-controls">
                    <Button
                      variant="outline"
                      size="icon"
                      class="btn-edge size-9"
                      onclick={() => goToPage(1)}
                      disabled={currentPage === 1}
                      aria-label="First page"
                    >
                      <ChevronsLeft class="size-4" />
                    </Button>
                    <Button
                      variant="outline"
                      size="icon"
                      class="size-9"
                      onclick={() => goToPage(currentPage - 1)}
                      disabled={currentPage === 1}
                      aria-label="Previous page"
                    >
                      <ChevronLeft class="size-4" />
                    </Button>
                    {#each getPageNumbers() as pageNum}
                      {#if pageNum === '...'}
                        <span class="page-ellipsis">&hellip;</span>
                      {:else}
                        <Button
                          variant={pageNum === currentPage ? 'default' : 'outline'}
                          size="icon"
                          class="btn-page size-9"
                          onclick={() => goToPage(pageNum)}
                          aria-current={pageNum === currentPage ? 'page' : undefined}
                        >
                          {pageNum}
                        </Button>
                      {/if}
                    {/each}
                    <Button
                      variant="outline"
                      size="icon"
                      class="size-9"
                      onclick={() => goToPage(currentPage + 1)}
                      disabled={currentPage === totalPages}
                      aria-label="Next page"
                    >
                      <ChevronRight class="size-4" />
                    </Button>
                    <Button
                      variant="outline"
                      size="icon"
                      class="btn-edge size-9"
                      onclick={() => goToPage(totalPages)}
                      disabled={currentPage === totalPages}
                      aria-label="Last page"
                    >
                      <ChevronsRight class="size-4" />
                    </Button>
                  </div>
                  <span class="pagination-page">
                    {m["results.page_of"]({ current: currentPage.toString(), total: totalPages.toString() })}
                  </span>
                </div>
              {/if}
            {/if}
          {:else if activeTab === 'dashboard'}
            {#if SemanticDashboard}
              <SemanticDashboard
                projectId={selectedProjectId}
                onFilterIssueType={handleFilterIssueType}
                bind:activeFilter={semanticFilter}
              />
            {/if}
          {:else if activeTab === 'site_tree'}
            {#if SiteTree}
              <SiteTree projectId={selectedProjectId} />
            {/if}
          {/if}
        </section>
      {/if}
    {/if}
  </main>
</div>

{#if PageDetailPanel}
  <PageDetailPanel bind:pageId={detailPageId} onClose={() => detailPageId = ''} />
{/if}

{#if SettingsModal}
  <SettingsModal bind:open={settingsModalOpen} />
{/if}

{#if exportProgress.running || exportProgress.percent === 100}
  <div class="export-progress-bar" aria-live="polite">
    <div class="progress-head">
      <span class="export-title">{m["export.progress"]()}</span>
      <span class="progress-pct">{Math.round(exportProgress.percent)}%</span>
    </div>
    <Progress value={exportProgress.percent} class="h-2 transition-all duration-300" />
    <div class="progress-stats">
      <span>
        {#if exportProgress.stage === 'pages'}
          {m["export.stage.pages"]()}
        {:else if exportProgress.stage === 'issues'}
          {m["export.stage.issues"]()}
        {:else if exportProgress.stage === 'links'}
          {m["export.stage.links"]()}
        {:else}
          {'…'}
        {/if}
      </span>
    </div>
  </div>
{/if}

<AlertDialog.Root bind:open={deleteDialogOpen} onOpenChange={(o) => { if (!o) closeDelete(); }}>
  <AlertDialog.Content class="max-w-md">
    <AlertDialog.Header>
      <AlertDialog.Title>{m["dialog.delete_title"]()}</AlertDialog.Title>
      <AlertDialog.Description>
        {m["dialog.delete_confirm"]()}
      </AlertDialog.Description>
    </AlertDialog.Header>
    <AlertDialog.Footer>
      <Button variant="outline" onclick={closeDelete}>
        {m["settings.cancel"]()}
      </Button>
      <Button variant="destructive" onclick={() => deleteProject(deletePendingId!)}>
        {m["sidebar.delete"]()}
      </Button>
    </AlertDialog.Footer>
  </AlertDialog.Content>
</AlertDialog.Root>

<style>
  .app-layout {
    display: flex;
    flex-direction: column;
    height: 100vh;
    height: 100dvh;
    overflow: hidden;
  }

  /* Header */
  .app-header {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 12px;
    padding: calc(10px + env(safe-area-inset-top)) 16px 10px;
    background: var(--bg-sidebar);
    border-bottom: 1px solid var(--border);
    z-index: 20;
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .header-center {
    display: flex;
    align-items: center;
    flex: 1;
    min-width: 180px;
  }

  .project-create {
    display: flex;
    gap: 6px;
    width: 100%;
    max-width: 420px;
  }

  .project-list-header {
    display: flex;
    gap: 8px;
    align-items: center;
    overflow-x: auto;
    overflow-y: hidden;
    scroll-behavior: smooth;
    scroll-snap-type: x proximity;
    padding: 6px 2px;
    -webkit-overflow-scrolling: touch;
    scrollbar-width: thin;
    scrollbar-color: var(--border-muted) transparent;
  }

  .project-list-header::-webkit-scrollbar {
    height: 6px;
  }

  .project-list-header::-webkit-scrollbar-track {
    background: transparent;
    border-radius: 3px;
  }

  .project-list-header::-webkit-scrollbar-thumb {
    background: var(--border-muted);
    border-radius: 3px;
  }

  .project-list-header::-webkit-scrollbar-thumb:hover {
    background: var(--text-muted);
  }

  .project-chip {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 10px 16px;
    min-height: 44px;
    border-radius: var(--radius-lg);
    cursor: pointer;
    transition: background var(--transition-base), border-color var(--transition-base), transform var(--transition-fast);
    border-left: 4px solid transparent;
    background: var(--bg-card);
    border: 1px solid var(--border);
    scroll-snap-align: start;
    flex-shrink: 0;
    font-size: 0.9rem;
  }

  .project-chip:hover {
    background: var(--bg-hover);
    transform: translateY(-1px);
  }

  .project-chip:active {
    transform: translateY(0);
  }

  .project-chip:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 1px;
  }

  .project-chip.selected {
    background: var(--bg-hover);
    border-left: 4px solid var(--accent);
    box-shadow: 0 0 0 1px var(--accent-subtle);
  }

  .project-name {
    font-size: 0.9rem;
    color: var(--text);
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 180px;
  }

  .project-avatar {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 26px;
    height: 26px;
    border-radius: 8px;
    background: var(--accent-gradient);
    color: #fff;
    font-size: 0.72rem;
    font-weight: 700;
    text-transform: uppercase;
    flex-shrink: 0;
  }

  .project-actions {
    display: none;
    gap: 6px;
  }

  .project-chip:hover .project-actions {
    display: flex;
  }

  .empty-projects {
    padding: 10px 12px;
    text-align: center;
    color: var(--text-muted);
    font-size: 0.85rem;
  }

  .logo {
    font-size: 1.1rem;
    font-weight: 700;
    background: var(--accent-gradient);
    -webkit-background-clip: text;
    background-clip: text;
    -webkit-text-fill-color: transparent;
    margin: 0;
    white-space: nowrap;
  }

  /* Main content */
  .main-content {
    flex: 1;
    overflow-y: auto;
    padding: var(--space-md);
    padding-bottom: calc(var(--space-md) + env(safe-area-inset-bottom));
    overscroll-behavior: contain;
  }

  .no-project {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 6px;
    color: var(--text-muted);
  }

  :global(.no-project-icon) {
    width: 48px;
    height: 48px;
    color: var(--border-muted);
    margin-bottom: 8px;
  }

  .no-project h2 {
    color: var(--text-secondary);
  }

  .project-header h2 {
    font-size: 1.4rem;
    color: var(--text);
    margin: 0;
  }

  .error {
    background: var(--danger-subtle);
    border: 1px solid var(--danger);
    color: var(--danger);
    padding: 12px 16px;
    border-radius: var(--radius-lg);
  }

  .sitemap-info {
    background: var(--success-subtle);
    border: 1px solid var(--success);
    color: var(--success);
    padding: 10px 16px;
    border-radius: var(--radius-lg);
    font-size: 0.9rem;
  }

  /* Resume hint */
  .resume-hint {
    margin-top: 12px;
    padding: 8px 12px;
    background: var(--bg-hover);
    border-radius: 6px;
    font-size: 0.85rem;
    color: var(--warning);
  }

  section {
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: var(--radius-xl);
    padding: var(--space-lg);
    box-shadow: var(--shadow-xs);
  }

  h2 {
    font-size: 1.15rem;
    margin-bottom: 16px;
    color: var(--text);
  }

  .form-group {
    margin-bottom: 16px;
  }

  .form-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;
  }

  label {
    display: block;
    margin-bottom: 6px;
    font-size: 0.9rem;
    color: var(--text-secondary);
  }

  .checkboxes {
    display: flex;
    gap: 24px;
    flex-wrap: wrap;
  }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
  }

  .actions {
    display: flex;
    gap: 12px;
    margin-top: 8px;
    flex-wrap: wrap;
  }

  .progress-section {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .export-progress-bar {
    position: fixed;
    bottom: calc(16px + env(safe-area-inset-bottom, 0px));
    left: 50%;
    transform: translateX(-50%);
    z-index: 60;
    display: flex;
    flex-direction: column;
    gap: 8px;
    width: min(420px, calc(100vw - 32px));
    padding: 12px 16px;
    background: var(--bg-sidebar);
    border: 1px solid var(--border);
    border-radius: 12px;
    box-shadow: 0 8px 30px rgba(0, 0, 0, 0.18);
  }

  .export-title {
    font-size: 0.85rem;
    font-weight: 600;
    color: var(--text-primary);
  }

  .progress-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }

  .progress-head h2 {
    margin-bottom: 0;
  }

  .progress-pct {
    font-size: 0.85rem;
    font-weight: 700;
    color: var(--accent);
    font-variant-numeric: tabular-nums;
  }

  .progress-stats {
    display: flex;
    gap: 24px;
    font-size: 0.9rem;
    color: var(--text-secondary);
  }

  .current-url {
    flex: 1;
    text-align: right;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    direction: rtl;
  }

  .streamed-info {
    font-size: 0.85rem;
    color: var(--success);
    font-style: italic;
  }

  .results-toolbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 12px;
  }

  .page-size-selector {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 0.9rem;
    color: var(--text-secondary);
  }

  /* Pagination */
  .results-skeleton {
    display: flex;
    flex-direction: column;
    gap: 8px;
    border-radius: var(--radius-lg);
    border: 1px solid var(--border);
    padding: 12px;
  }

  .pagination {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-top: 20px;
    padding-top: 16px;
    border-top: 1px solid var(--border);
  }

  .pagination-info {
    font-size: 0.85rem;
    color: var(--text-secondary);
  }

  .pagination-controls {
    display: flex;
    gap: 4px;
    align-items: center;
  }

  .page-ellipsis {
    padding: 0 4px;
    color: var(--text-muted);
  }

  .pagination-page {
    font-size: 0.85rem;
    color: var(--text-secondary);
  }

  /* ==========================================
     RESPONSIVE — Mobile First
     Base = mobile (≤ 767px)
     ========================================== */

  /* Header mobile: stacked layout */
  .app-header {
    flex-wrap: wrap;
    gap: 10px;
    padding: calc(10px + env(safe-area-inset-top)) 12px 10px;
  }

  .header-left {
    width: 100%;
    justify-content: space-between;
  }

  .header-center {
    width: 100%;
    flex: 1 1 100%;
  }

  .project-list-header {
    width: 100%;
    overflow-x: auto;
    padding: 4px 2px;
  }

  .project-chip {
    padding: 8px 14px;
    min-height: 42px;
    font-size: 0.85rem;
  }

  .project-name {
    max-width: 140px;
    font-size: 0.85rem;
  }

  .main-content {
    padding: var(--space-md);
    padding-bottom: calc(var(--space-md) + env(safe-area-inset-bottom));
  }

  .form-row {
    grid-template-columns: 1fr;
  }

  .progress-stats {
    flex-wrap: wrap;
    gap: var(--space-sm);
  }

  .results-toolbar {
    flex-direction: column;
    align-items: flex-start;
    gap: var(--space-sm);
  }

  .pagination {
    flex-direction: column;
    gap: var(--space-md);
    align-items: center;
  }

  .pagination-controls {
    flex-wrap: wrap;
    justify-content: center;
  }

  :global(.btn-edge) {
    display: none;
  }

  section {
    padding: var(--section-padding);
    border-radius: var(--radius-lg);
  }

  .checkboxes {
    gap: var(--space-md);
  }

  /* --- Tablet (768px+) --- */
  @media (min-width: 768px) {
    .app-header {
      flex-wrap: nowrap;
      gap: 16px;
      padding: calc(12px + env(safe-area-inset-top)) 20px 12px;
    }

    .header-left {
      width: auto;
    }

    .header-center {
      flex: 0 1 420px;
    }

    .project-list-header {
      width: auto;
      overflow-x: auto;
      padding: 4px 2px;
    }

    .project-chip {
      padding: 10px 16px;
      min-height: 44px;
      font-size: 0.9rem;
    }

    .project-name {
      max-width: 180px;
      font-size: 0.9rem;
    }

    .main-content {
      padding: var(--space-lg);
    }

    .form-row {
      grid-template-columns: 1fr 1fr;
    }

    .results-toolbar {
      flex-direction: row;
      justify-content: space-between;
    }

    .pagination {
      flex-direction: row;
      justify-content: space-between;
    }

    :global(.btn-edge) {
      display: inline-flex;
    }
  }

  /* --- Desktop (1024px+) --- */
  @media (min-width: 1024px) {
    .app-header {
      padding: calc(14px + env(safe-area-inset-top)) 24px 14px;
      gap: 20px;
    }

    .header-center {
      flex: 0 1 520px;
    }

    .project-create {
      max-width: 520px;
    }

    .main-content {
      padding: var(--space-lg);
      padding-bottom: calc(var(--space-lg) + env(safe-area-inset-bottom));
    }
  }

  /* --- Wide (1440px+) --- */
  @media (min-width: 1440px) {
    .app-header {
      padding: calc(16px + env(safe-area-inset-top)) 32px 16px;
    }

    .main-content {
      padding: var(--space-lg) var(--space-xl);
      padding-bottom: calc(var(--space-lg) + env(safe-area-inset-bottom));
    }
  }
</style>
