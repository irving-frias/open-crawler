<script lang="ts">
  import { m } from '$lib/paraglide/messages.js';
  import { SearchX, Pencil, Trash2, Loader2 } from 'lucide-svelte';
  import { setAppShell } from '$lib/app.svelte';
  import AppHeader from '$lib/features/project/AppHeader.svelte';
  import DeleteProjectDialog from '$lib/features/project/DeleteProjectDialog.svelte';
  import CrawlControls from '$lib/features/crawl-controls/CrawlControls.svelte';
  import CrawlProgress from '$lib/features/crawl-controls/CrawlProgress.svelte';
  import ResumeDialog from '$lib/features/crawl-controls/ResumeDialog.svelte';
  import ResultsView from '$lib/features/results/ResultsView.svelte';
  import ExportProgressBar from '$lib/features/export/ExportProgressBar.svelte';
  import Splash from '$lib/features/splash/Splash.svelte';
  import { Input } from '$lib/components/ui/input/index.js';
  import { Button } from '$lib/components/ui/button/index.js';

  const app = setAppShell();

  let PageDetailPanel = $state<typeof import('$lib/features/page-detail/PageDetailPanel.svelte').default | null>(null);
  let SettingsModal = $state<typeof import('$lib/features/settings/SettingsModal.svelte').default | null>(null);

  const splashMountTime = Date.now();
  let splashVisible = $state(true);
  let splashGone = $state(false);

  $effect(() => {
    if (app.initialized && !splashGone) {
      const remaining = Math.max(0, 800 - (Date.now() - splashMountTime));
      const timer = setTimeout(() => {
        splashVisible = false;
        setTimeout(() => {
          splashGone = true;
        }, 450);
      }, remaining);
      return () => clearTimeout(timer);
    }
  });

  $effect(() => {
    if (app.detailPageId && !PageDetailPanel) {
      import('$lib/features/page-detail/PageDetailPanel.svelte').then((m) => (PageDetailPanel = m.default));
    }
  });

  $effect(() => {
    if (app.settingsModalOpen && !SettingsModal) {
      import('$lib/features/settings/SettingsModal.svelte').then((m) => (SettingsModal = m.default));
    }
  });
</script>

<div class="app-layout">
  {#if !splashGone}
    <Splash visible={splashVisible} />
  {/if}
  <AppHeader
    projects={app.projects}
    selectedProjectId={app.selectedProjectId}
    bind:newProjectName={app.newProjectName}
    onSelect={app.selectProject}
    onCreate={app.createProject}
    onOpenSettings={() => (app.settingsModalOpen = true)}
  />

  <!-- Main content -->
  <main class="main-content">
    {#if !app.initialized}
      <div class="boot-state">
        <Loader2 class="boot-spinner" />
        <span>{m['app.loading']()}</span>
      </div>
    {:else if !app.selectedProjectId}
      <div class="no-project">
        <SearchX class="no-project-icon" />
        <h2>{m['app.select_project']()}</h2>
        <p>{m['app.select_project_hint']()}</p>
      </div>
    {:else}
      <div class="project-header">
        {#if app.renamingProjectId === app.selectedProjectId}
          <Input
            type="text"
            bind:value={app.renamingName}
            onkeydown={(e) => {
              if (e.key === 'Enter') app.confirmRename();
              if (e.key === 'Escape') app.cancelRename();
            }}
            onblur={app.confirmRename}
            class="h-9 max-w-xs text-base"
            placeholder={m['sidebar.new_project_placeholder']()}
          />
        {:else}
          <h2 class="project-title">
            {#if app.siteFavicon}
              <img src={app.siteFavicon} alt="" class="site-favicon" />
            {/if}
            <span>{app.getSelectedProject()?.name}</span>
          </h2>
          <div class="project-actions">
            <Button
              variant="ghost"
              size="sm"
              class="btn-project-action"
              title={m['sidebar.rename']()}
              aria-label={m['sidebar.rename']()}
              onclick={() => {
                const p = app.getSelectedProject();
                if (p) app.startRename(p.id, p.name);
              }}
            >
              <Pencil class="size-4" />
              <span>{m['sidebar.rename']()}</span>
            </Button>
            <Button
              variant="ghost"
              size="sm"
              class="btn-project-action hover:bg-destructive hover:text-white"
              title={m['sidebar.delete']()}
              aria-label={m['sidebar.delete']()}
              onclick={() => app.requestDelete(app.selectedProjectId)}
            >
              <Trash2 class="size-4" />
              <span>{m['sidebar.delete']()}</span>
            </Button>
          </div>
        {/if}
      </div>

      {#if app.error}
        <div class="error">{app.error}</div>
      {/if}

      <ResumeDialog
        bind:open={app.showResumeDialog}
        resumableInfo={app.resumableInfo}
        elapsedLabel={app.formatDuration(app.resumableInfo?.elapsed_secs ?? 0)}
        onFreshStart={() => {
          app.showResumeDialog = false;
          app.startCrawl(false);
        }}
        onResume={() => app.startCrawl(true)}
      />

      <CrawlControls
        status={app.status}
        resumableInfo={app.resumableInfo}
        hasResults={app.results.items.length > 0}
        exporting={app.exportProgress.running}
        bind:seedUrl={app.seedUrl}
        bind:maxDepth={app.maxDepth}
        bind:maxCrawlTime={app.maxCrawlTime}
        bind:proxyUrl={app.proxyUrl}
        bind:proxyUser={app.proxyUser}
        bind:proxyPass={app.proxyPass}
        bind:respectRobots={app.respectRobots}
        bind:renderJs={app.renderJs}
        bind:checkSitemap={app.checkSitemap}
        bind:checkSemantics={app.checkSemantics}
        onStart={app.handleStartCrawl}
        onStop={app.stopCrawl}
        onRefresh={() => app.loadResults(app.currentPage)}
        onExport={app.exportFull}
      />

      {#if app.status === 'running'}
        <CrawlProgress
          progress={app.progress}
          streamedCount={app.streamedCount}
        />
      {/if}

      {#if app.sitemapInfo}
        <div class="sitemap-info">{app.sitemapInfo}</div>
      {/if}

      {#if app.results.items.length > 0 || app.streamedCount > 0 || app.resultsLoading}
        <ResultsView
          bind:activeTab={app.activeTab}
          results={app.results}
          resultsLoading={app.resultsLoading}
          currentPage={app.currentPage}
          pageSize={app.pageSize}
          bind:pageSizeSelect={app.pageSizeSelect}
          bind:semanticFilter={app.semanticFilter}
          debouncedSearch={app.debouncedSearch}
          bind:expandedIssueUrl={app.expandedIssueUrl}
          selectedProjectId={app.selectedProjectId}
          onPageSizeChange={app.changePageSize}
          onGoToPage={app.goToPage}
          onOpenDetail={app.openDetail}
          onSearch={app.onSearchInput}
          onFilterChange={app.handleFilterChange}
          onFilterIssueType={app.handleFilterIssueType}
          onClearFilter={() => app.handleFilterIssueType(null)}
        />
      {/if}
    {/if}
  </main>
</div>

{#if PageDetailPanel}
  <PageDetailPanel bind:pageId={app.detailPageId} onClose={() => (app.detailPageId = '')} />
{/if}

{#if SettingsModal}
  <SettingsModal bind:open={app.settingsModalOpen} />
{/if}

<ExportProgressBar exportProgress={app.exportProgress} />

<DeleteProjectDialog
  bind:open={app.deleteDialogOpen}
  pendingId={app.deletePendingId}
  onDelete={app.deleteProject}
  onClose={app.closeDelete}
/>

<style>
  .app-layout {
    display: flex;
    flex-direction: column;
    height: 100vh;
    height: 100dvh;
    overflow: hidden;
  }

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

  .project-title {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .project-actions {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .boot-state {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    color: var(--text-muted);
    font-size: 0.9rem;
  }

  :global(.boot-spinner) {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .site-favicon {
    width: 20px;
    height: 20px;
    border-radius: 6px;
    object-fit: contain;
    background: var(--bg-card);
  }

  .error {
    background: var(--danger-subtle);
    border: none;
    box-shadow: var(--neu-raised-sm);
    color: var(--danger);
    padding: 12px 16px;
    border-radius: var(--radius-lg);
  }

  .sitemap-info {
    background: var(--success-subtle);
    border: none;
    box-shadow: var(--neu-raised-sm);
    color: var(--success);
    padding: 10px 16px;
    border-radius: var(--radius-lg);
    font-size: 0.9rem;
  }

  @media (max-width: 767px) {
    .main-content {
      padding: var(--space-md);
      padding-bottom: calc(var(--space-md) + env(safe-area-inset-bottom));
    }
  }

  @media (min-width: 768px) {
    .main-content {
      padding: var(--space-lg);
    }
  }

  @media (min-width: 1024px) {
    .main-content {
      padding: var(--space-lg);
      padding-bottom: calc(var(--space-lg) + env(safe-area-inset-bottom));
    }
  }

  @media (min-width: 1440px) {
    .main-content {
      padding: var(--space-lg) var(--space-xl);
      padding-bottom: calc(var(--space-lg) + env(safe-area-inset-bottom));
    }
  }
</style>
