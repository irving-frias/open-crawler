<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import { confirm } from '@tauri-apps/plugin-dialog';
  import ResultsTable from '$lib/components/ResultsTable.svelte';
  import PageDetailPanel from '$lib/components/PageDetailPanel.svelte';
  import SemanticDashboard from '$lib/components/SemanticDashboard.svelte';
  import SettingsModal from '$lib/components/SettingsModal.svelte';
  import Toast from '$lib/components/Toast.svelte';
  import { m } from '$lib/paraglide/messages.js';

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

  // Crawl state
  let status = $state<'idle' | 'running' | 'paused' | 'completed' | 'error'>('idle');
  let progress = $state({ crawled: 0, queued: 0, current: '', errors: 0 });
  let results = $state<any>({ items: [], total: 0, page: 1, page_size: 50 });
  let error = $state('');
  let sitemapInfo = $state('');

  // Pagination
  let currentPage = $state(1);
  let pageSize = $state(50);
  let totalPages = $derived(Math.ceil(results.total / pageSize));

  // Resume state
  let resumableInfo = $state<any>(null);
  let showResumeDialog = $state(false);

  // Streamed count during live crawl
  let streamedCount = $state(0);

  // Expanded issue row
  let expandedIssueUrl = $state('');

  // Tab + filter state
  let activeTab = $state<'results' | 'dashboard'>('results');
  let semanticFilter = $state('');

  // Detail panel
  let detailPageId = $state('');

  // Mobile sidebar
  let sidebarOpen = $state(false);

  // Search
  let searchQuery = $state('');
  let debouncedSearch = $state('');
  let searchDebounceTimer: ReturnType<typeof setTimeout> | null = null;

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

  // Toast notifications
  let toasts = $state<Array<{ id: number; message: string; type: 'success' | 'error' | 'warning' | 'info' }>>([]);
  let toastCounter = $state(0);
  function showToast(message: string, type: 'success' | 'error' | 'warning' | 'info' = 'info') {
    const id = ++toastCounter;
    toasts = [...toasts, { id, message, type }];
    setTimeout(() => { toasts = toasts.filter((t) => t.id !== id); }, 5000);
  }
  function dismissToast(id: number) {
    toasts = toasts.filter((t) => t.id !== id);
  }

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
    } catch (e) {
      // Settings may not exist yet, use defaults
    }
  }

  function selectProject(id: string) {
    selectedProjectId = id;
    sidebarOpen = false;
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

  async function deleteProject(id: string) {
    const ok = await confirm(m["dialog.delete_confirm"](), { title: m["dialog.delete_title"](), kind: 'warning' });
    if (!ok) return;
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
    });
    unlistenFns.push(un2);

    const un3 = await listen<any>('crawl-error', (event) => {
      const p = event.payload;
      if (p.project_id !== selectedProjectId) return;
      status = 'error';
      error = p.error || String(p);
    });
    unlistenFns.push(un3);

    const un4 = await listen<any>('crawl-stopped', (event) => {
      const p = event.payload;
      if (p.project_id !== selectedProjectId) return;
      status = 'idle';
      streamedCount = 0;
      checkResumable();
    });
    unlistenFns.push(un4);

    const un5 = await listen<any>('sitemap-discovered', (event) => {
      const p = event.payload;
      sitemapInfo = `Sitemap: ${p.urls_found} URLs from ${p.sitemaps_checked} sitemaps`;
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
        },
      });
    } catch (e) {
      status = 'error';
      error = String(e);
      showToast(String(e), 'error');
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
      showToast(m["progress.title"]() + ': stopped', 'info');
    } catch (e) {
      error = String(e);
      showToast(String(e), 'error');
    }
  }

  async function loadResults(page: number = 1) {
    if (!selectedProjectId) return;
    try {
      currentPage = page;
      const data = await invoke('get_results', {
        projectId: selectedProjectId,
        page: currentPage,
        pageSize: pageSize,
        semanticIssueType: semanticFilter || null,
        search: debouncedSearch || null,
      });
      results = data;
    } catch (e) {
      console.error('[Crawler] Failed to load results:', e);
    }
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

  async function exportFull() {
    if (!selectedProjectId) return;
    try {
      const { save, confirm } = await import('@tauri-apps/plugin-dialog');

      const path = await save({
        defaultPath: `crawl-results-${selectedProjectId}.xlsx`,
        filters: [{ name: 'Excel', extensions: ['xlsx'] }, { name: 'CSV', extensions: ['csv'] }],
      });
      if (!path) return;

      const format = path.endsWith('.csv') ? 'csv' : 'xlsx';
      await invoke('export_full', {
        projectId: selectedProjectId,
        filePath: path,
        format,
      });
      showToast(`Exported to ${path.split(/[/\\]/).pop()}`, 'success');
    } catch (e) {
      error = String(e);
      showToast(String(e), 'error');
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
  <Toast {toasts} onDismiss={dismissToast} />

  <!-- Mobile hamburger -->
  <button class="hamburger" onclick={() => sidebarOpen = !sidebarOpen} aria-label="Toggle menu">
    {sidebarOpen ? '✕' : '☰'}
  </button>

  <!-- Sidebar overlay (mobile) -->
  {#if sidebarOpen}
    <div class="sidebar-overlay" onclick={() => sidebarOpen = false} role="presentation"></div>
  {/if}

  <!-- Sidebar -->
  <aside class="sidebar" class:open={sidebarOpen}>
    <div class="sidebar-header">
      <h1 class="logo">{m["app.title"]()}</h1>
      <button class="btn-settings" onclick={() => settingsModalOpen = true} aria-label="Settings">⚙️</button>
    </div>

    <div class="project-create">
      <input
        type="text"
        bind:value={newProjectName}
        placeholder={m["sidebar.new_project_placeholder"]()}
        onkeydown={(e) => e.key === 'Enter' && createProject()}
      />
      <button class="btn-icon" onclick={createProject} disabled={!newProjectName.trim()}>+</button>
    </div>

    <nav class="project-list">
      {#each projects as project (project.id)}
        <div
          class="project-item"
          class:selected={project.id === selectedProjectId}
          role="button"
          tabindex="0"
          onclick={() => selectProject(project.id)}
          onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') selectProject(project.id); }}
        >
          {#if renamingProjectId === project.id}
            <input
              type="text"
              bind:value={renamingName}
              onkeydown={(e) => {
                if (e.key === 'Enter') confirmRename();
                if (e.key === 'Escape') cancelRename();
              }}
              onblur={confirmRename}
              class="rename-input"
              onclick={(e) => e.stopPropagation()}
            />
          {:else}
            <span class="project-name">{project.name}</span>
            <span class="project-date">
              {new Date(project.created_at).toLocaleDateString()}
            </span>
            <div class="project-actions">
              <button
                class="btn-mini"
                title={m["sidebar.rename"]()}
                onclick={(e) => {
                  e.stopPropagation();
                  startRename(project.id, project.name);
                }}
              >&#9998;</button>
              <button
                class="btn-mini btn-mini-danger"
                title={m["sidebar.delete"]()}
                onclick={(e) => {
                  e.stopPropagation();
                  deleteProject(project.id);
                }}
              >&times;</button>
            </div>
          {/if}
        </div>
      {/each}
      {#if projects.length === 0}
        <div class="empty-projects">{m["sidebar.no_projects"]()}</div>
      {/if}
    </nav>
  </aside>

  <!-- Main content -->
  <main class="main-content">
    {#if !selectedProjectId}
      <div class="no-project">
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
        <div class="resume-dialog">
          <h3>{m["resume.title"]()}</h3>
          <p>
            {m["resume.found"]({ pages: resumableInfo.pages_crawled, urls: resumableInfo.queue_remaining, time: formatDuration(resumableInfo.elapsed_secs) })}
          </p>
          <div class="resume-actions">
            <button class="btn btn-primary" onclick={() => startCrawl(true)}>
              {m["resume.resume_btn"]()}
            </button>
            <button class="btn btn-secondary" onclick={() => { showResumeDialog = false; startCrawl(false); }}>
              {m["resume.fresh_btn"]()}
            </button>
            <button class="btn btn-secondary" onclick={() => showResumeDialog = false}>
              {m["resume.cancel"]()}
            </button>
          </div>
        </div>
      {/if}

      <section class="config-section">
        <h2>{m["config.title"]()}</h2>

        <div class="form-group">
          <label for="seed">{m["config.seed_url"]()}</label>
          <input
            id="seed"
            type="url"
            bind:value={seedUrl}
            placeholder={m["config.seed_url_placeholder"]()}
            disabled={status === 'running'}
          />
        </div>

        <div class="form-row">
          <div class="form-group">
            <label for="maxDepth">{m["config.max_depth"]()}</label>
            <input id="maxDepth" type="number" bind:value={maxDepth} min="1" max="100" disabled={status === 'running'} />
          </div>
          <div class="form-group">
            <label for="maxTime">{m["config.time_limit"]()}</label>
            <input id="maxTime" type="number" bind:value={maxCrawlTime} min="0" max="86400" disabled={status === 'running'} />
          </div>
        </div>

        <div class="form-group checkboxes">
          <label class="checkbox-label">
            <input type="checkbox" bind:checked={respectRobots} disabled={status === 'running'} />
            {m["config.respect_robots"]()}
          </label>
          <label class="checkbox-label">
            <input type="checkbox" bind:checked={renderJs} disabled={status === 'running'} />
            {m["config.render_js"]()}
          </label>
          <label class="checkbox-label">
            <input type="checkbox" bind:checked={checkSitemap} disabled={status === 'running'} />
            {m["config.check_sitemap"]()}
          </label>
          <label class="checkbox-label">
            <input type="checkbox" bind:checked={checkSemantics} disabled={status === 'running'} />
            {m["config.check_semantics"]()}
          </label>
        </div>

        <div class="actions">
          {#if status === 'idle' || status === 'completed' || status === 'error'}
            <button class="btn btn-primary" onclick={handleStartCrawl} disabled={!seedUrl}>
              {resumableInfo ? m["config.resume"]() : m["config.start"]()}
            </button>
          {:else if status === 'running'}
            <button class="btn btn-danger" onclick={stopCrawl}>{m["config.stop"]()}</button>
          {/if}
          <button class="btn btn-secondary" onclick={() => loadResults(currentPage)}>{m["config.refresh"]()}</button>
          {#if results.items.length > 0}
            <button class="btn btn-secondary" onclick={exportFull}>{m["settings.export"]()}</button>
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
          <h2>{m["progress.title"]()}</h2>
          <div class="progress-bar">
            <div
              class="progress-fill"
              style="width: {progress.crawled > 0
                ? Math.min((progress.crawled / (progress.crawled + progress.queued)) * 100, 100)
                : 0}%"
            ></div>
          </div>
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

      {#if results.items.length > 0 || streamedCount > 0}
        <section class="results-section">
          <div class="results-tabs">
            <button class="tab" class:active={activeTab === 'results'} onclick={() => activeTab = 'results'}>
              {m["tabs.results"]({ count: results.total.toLocaleString() })}
            </button>
            <button class="tab" class:active={activeTab === 'dashboard'} onclick={() => activeTab = 'dashboard'}>
              {m["tabs.issues_dashboard"]()}
            </button>
          </div>

          {#if activeTab === 'results'}
            <div class="results-toolbar">
              <div class="page-size-selector">
                <label for="pageSizeSelect">{m["results.page_size_show"]()}</label>
                <select id="pageSizeSelect" bind:value={pageSize} onchange={() => changePageSize(pageSize)}>
                  <option value={25}>25</option>
                  <option value={50}>50</option>
                  <option value={100}>100</option>
                  <option value={200}>200</option>
                </select>
              </div>
              {#if semanticFilter}
                <span class="active-filter">
                  {m["results.filtered_by"]({ type: semanticFilter.replace(/_/g, ' ') })}
                  <button class="btn-clear-filter" onclick={() => handleFilterIssueType(null)}>&times;</button>
                </span>
              {/if}
            </div>

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
                  <button class="btn-page" onclick={() => goToPage(1)} disabled={currentPage === 1}>&laquo;</button>
                  <button class="btn-page" onclick={() => goToPage(currentPage - 1)} disabled={currentPage === 1}>&lsaquo;</button>
                  {#each getPageNumbers() as pageNum}
                    {#if pageNum === '...'}
                      <span class="page-ellipsis">&hellip;</span>
                    {:else}
                      <button
                        class="btn-page"
                        class:active={pageNum === currentPage}
                        onclick={() => goToPage(pageNum)}
                      >
                        {pageNum}
                      </button>
                    {/if}
                  {/each}
                  <button class="btn-page" onclick={() => goToPage(currentPage + 1)} disabled={currentPage === totalPages}>&rsaquo;</button>
                  <button class="btn-page" onclick={() => goToPage(totalPages)} disabled={currentPage === totalPages}>&raquo;</button>
                </div>
                <span class="pagination-page">
                  {m["results.page_of"]({ current: currentPage.toString(), total: totalPages.toString() })}
                </span>
              </div>
            {/if}
          {:else}
            <SemanticDashboard
              projectId={selectedProjectId}
              onFilterIssueType={handleFilterIssueType}
              bind:activeFilter={semanticFilter}
            />
          {/if}
        </section>
      {/if}
    {/if}
  </main>
</div>

<PageDetailPanel bind:pageId={detailPageId} onClose={() => detailPageId = ''} />

<SettingsModal bind:open={settingsModalOpen} />

<style>
  .app-layout {
    display: flex;
    height: 100vh;
    overflow: hidden;
  }

  /* Sidebar */
  .sidebar {
    width: 350px;
    min-width: 350px;
    background: var(--bg-sidebar);
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .sidebar-header {
    padding: 20px;
    border-bottom: 1px solid var(--bg-hover);
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .btn-settings {
    background: none;
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    font-size: 1.1rem;
    cursor: pointer;
    padding: 4px 8px;
    transition: all var(--transition-base);
    line-height: 1;
  }
  .btn-settings:hover {
    background: var(--bg-hover);
    border-color: var(--text-muted);
  }

  .logo {
    font-size: 1.3rem;
    font-weight: 700;
    background: var(--accent-gradient);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    margin: 0;
  }

  .project-create {
    display: flex;
    gap: 6px;
    padding: 12px 16px;
    border-bottom: 1px solid var(--bg-hover);
  }

  .project-create input {
    flex: 1;
    padding: 8px 10px;
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    color: var(--text);
    font-size: 0.85rem;
    transition: border-color var(--transition-base), box-shadow var(--transition-base);
  }

  .project-create input:focus {
    outline: none;
    border-color: var(--accent);
    box-shadow: 0 0 0 2px var(--accent-subtle);
  }

  .btn-icon {
    width: 34px;
    height: 34px;
    background: var(--accent-gradient);
    border: none;
    border-radius: var(--radius-md);
    color: white;
    font-size: 1.2rem;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: transform var(--transition-fast), box-shadow var(--transition-base);
  }
  .btn-icon:hover:not(:disabled) {
    transform: scale(1.05);
    box-shadow: 0 2px 8px rgba(102, 126, 234, 0.4);
  }
  .btn-icon:active:not(:disabled) {
    transform: scale(0.95);
  }

  .btn-icon:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .project-list {
    flex: 1;
    overflow-y: auto;
    padding: 8px;
  }

  .project-item {
    padding: 10px 12px;
    border-radius: var(--radius-lg);
    cursor: pointer;
    margin-bottom: 4px;
    transition: background var(--transition-base), border-color var(--transition-base);
    border-left: 3px solid transparent;
  }

  .project-item:hover {
    background: var(--bg-card);
  }

  .project-item.selected {
    background: var(--bg-hover);
    border-left: 3px solid var(--accent);
  }

  .project-name {
    display: block;
    font-size: 0.9rem;
    color: var(--text);
    font-weight: 500;
  }

  .project-date {
    display: block;
    font-size: 0.75rem;
    color: var(--text-muted);
    margin-top: 2px;
  }

  .project-actions {
    display: none;
    gap: 4px;
    margin-top: 6px;
  }

  .project-item:hover .project-actions {
    display: flex;
  }

  .btn-mini {
    padding: 2px 8px;
    background: var(--border);
    border: none;
    border-radius: 4px;
    color: var(--text-secondary);
    cursor: pointer;
    font-size: 0.8rem;
  }

  .btn-mini:hover {
    background: #4a4d54;
    color: var(--text);
  }

  .btn-mini-danger:hover {
    background: var(--danger);
    color: white;
  }

  .rename-input {
    width: 100%;
    padding: 4px 8px;
    background: var(--bg-deep);
    border: 1px solid var(--accent);
    border-radius: 4px;
    color: var(--text);
    font-size: 0.9rem;
  }

  .empty-projects {
    padding: 20px;
    text-align: center;
    color: var(--text-muted);
    font-size: 0.85rem;
  }

  /* Main content */
  .main-content {
    flex: 1;
    overflow-y: auto;
    padding: 24px 32px;
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  .no-project {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    color: var(--text-muted);
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

  /* Resume dialog */
  .resume-dialog {
    background: var(--bg-card);
    border: 1px solid var(--accent);
    border-radius: 12px;
    padding: 24px;
  }

  .resume-dialog h3 {
    margin: 0 0 12px 0;
    color: var(--text);
  }

  .resume-dialog p {
    margin: 0 0 16px 0;
    color: var(--text-secondary);
    line-height: 1.5;
  }

  .resume-actions {
    display: flex;
    gap: 12px;
  }

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

  input[type='url'],
  input[type='text'],
  input[type='number'],
  select {
    width: 100%;
    padding: 10px 14px;
    background: var(--bg-deep);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    color: var(--text);
    font-size: 0.95rem;
    transition: border-color var(--transition-base), box-shadow var(--transition-base);
  }

  input:focus,
  select:focus {
    outline: none;
    border-color: var(--accent);
    box-shadow: 0 0 0 2px var(--accent-subtle);
  }

  input:disabled {
    opacity: 0.5;
    cursor: not-allowed;
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

  .checkbox-label input[type='checkbox'] {
    width: 18px;
    height: 18px;
    accent-color: var(--accent);
  }

  .actions {
    display: flex;
    gap: 12px;
    margin-top: 8px;
  }

  .btn {
    padding: 10px 24px;
    border: none;
    border-radius: var(--radius-lg);
    font-size: 0.95rem;
    font-weight: var(--weight-semibold);
    cursor: pointer;
    transition: all var(--transition-base);
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-primary {
    background: var(--accent-gradient);
    color: white;
  }

  .btn-primary:hover:not(:disabled) {
    transform: translateY(-1px);
    box-shadow: var(--shadow-md);
  }
  .btn-primary:active:not(:disabled) {
    transform: translateY(0);
  }

  .btn-secondary {
    background: var(--border);
    color: var(--text);
  }

  .btn-secondary:hover:not(:disabled) {
    background: #4a4d54;
  }

  .btn-danger {
    background: var(--danger);
    color: white;
  }

  .btn-danger:hover:not(:disabled) {
    background: #ff5252;
  }

  .progress-section {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .progress-bar {
    height: 8px;
    background: var(--bg-deep);
    border-radius: 4px;
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    background: var(--accent-gradient);
    transition: width 0.3s;
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

  .results-tabs {
    display: flex;
    gap: 4px;
    margin-bottom: 16px;
    border-bottom: 1px solid var(--border);
    padding-bottom: 0;
  }

  .tab {
    padding: 10px 20px;
    background: none;
    border: none;
    border-bottom: 2px solid transparent;
    color: var(--text-secondary);
    font-size: 0.9rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s;
  }
  .tab:hover { color: var(--text); }
  .tab.active {
    color: var(--text);
    border-bottom-color: var(--accent);
  }

  .results-toolbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 12px;
  }

  .active-filter {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 10px;
    background: var(--border);
    border-radius: 6px;
    font-size: 0.8rem;
    color: var(--text);
  }

  .btn-clear-filter {
    background: none;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    font-size: 1rem;
    padding: 0;
  }
  .btn-clear-filter:hover { color: var(--danger); }

  .page-size-selector {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 0.9rem;
    color: var(--text-secondary);
  }

  .page-size-selector select {
    width: auto;
    padding: 4px 8px;
    font-size: 0.85rem;
  }

  /* Pagination */
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

  .btn-page {
    min-width: 36px;
    height: 36px;
    padding: 0 8px;
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    background: var(--bg-deep);
    color: var(--text-secondary);
    font-size: 0.85rem;
    cursor: pointer;
    transition: all var(--transition-base);
  }

  .btn-page:hover:not(:disabled):not(.active) {
    background: var(--bg-hover);
    color: var(--text);
    border-color: var(--accent);
  }

  .btn-page.active {
    background: var(--accent-gradient);
    color: white;
    border-color: transparent;
  }

  .btn-page:disabled {
    opacity: 0.3;
    cursor: not-allowed;
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

  /* --- Hamburger (mobile only) --- */
  .hamburger {
    display: none;
    position: fixed;
    top: 12px;
    left: 12px;
    z-index: calc(var(--z-sidebar) + 2);
    width: 40px;
    height: 40px;
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    color: var(--text);
    font-size: 1.2rem;
    cursor: pointer;
    align-items: center;
    justify-content: center;
    transition: background var(--transition-base);
  }
  .hamburger:hover { background: var(--bg-hover); }

  .sidebar-overlay {
    display: none;
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    z-index: calc(var(--z-sidebar) - 1);
  }

  /* --- Mobile base (≤ 767px) --- */
  .sidebar {
    position: fixed;
    top: 0;
    left: 0;
    bottom: 0;
    width: 280px;
    z-index: var(--z-sidebar);
    transform: translateX(-100%);
    transition: transform var(--transition-base);
    background: var(--bg-sidebar);
  }
  .sidebar.open {
    transform: translateX(0);
  }

  .main-content {
    padding: var(--space-md);
    padding-top: 60px;
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

  .btn-page {
    min-width: 32px;
    height: 32px;
    font-size: var(--text-sm);
  }

  section {
    padding: var(--section-padding);
    border-radius: var(--radius-lg);
  }

  .resume-actions {
    flex-direction: column;
  }

  .resume-actions .btn {
    width: 100%;
    justify-content: center;
  }

  .checkboxes {
    gap: var(--space-md);
  }

  /* --- Tablet (768px+) --- */
  @media (min-width: 768px) {
    .hamburger {
      display: none;
    }

    .sidebar {
      position: fixed;
      transform: translateX(-100%);
      transition: transform var(--transition-base);
    }
    .sidebar.open {
      transform: translateX(0);
    }

    .main-content {
      padding: var(--space-lg);
      padding-top: var(--space-lg);
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

    .resume-actions {
      flex-direction: row;
    }
    .resume-actions .btn {
      width: auto;
    }
  }

  /* --- Desktop (1024px+) --- */
  @media (min-width: 1024px) {
    .hamburger {
      display: none;
    }

    .sidebar-overlay {
      display: none;
    }

    .sidebar {
      position: relative;
      transform: none;
      width: 300px;
      min-width: 300px;
    }

    .main-content {
      padding: var(--space-lg);
      padding-top: var(--space-lg);
    }
  }

  /* --- Wide (1440px+) --- */
  @media (min-width: 1440px) {
    .sidebar {
      width: 350px;
      min-width: 350px;
    }

    .main-content {
      padding: var(--space-lg) var(--space-xl);
    }
  }

  /* --- Mobile only: show hamburger --- */
  @media (max-width: 767px) {
    .hamburger {
      display: flex;
    }

    .sidebar-overlay {
      display: block;
    }
  }
</style>
