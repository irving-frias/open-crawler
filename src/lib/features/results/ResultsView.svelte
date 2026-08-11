<script lang="ts">
  import { m } from '$lib/paraglide/messages.js';
  import { X } from '@lucide/svelte';
  import { Button } from '$lib/components/ui/button/index.js';
  import { Label } from '$lib/components/ui/label/index.js';
  import { Badge } from '$lib/components/ui/badge/index.js';
  import { Skeleton } from '$lib/components/ui/skeleton/index.js';
  import * as Tabs from '$lib/components/ui/tabs/index.js';
  import * as Select from '$lib/components/ui/select/index.js';
  import FilterBar, { type FilterState } from '$lib/features/results/FilterBar.svelte';
  import ResultsTable from '$lib/features/results/ResultsTable.svelte';
  import Pagination from '$lib/features/results/Pagination.svelte';
  import { translateIssueName } from '$lib/i18n-issues';
  import { TAB_DEFS, loadTabComponent, type LazyTabId } from '$lib/tabs';
  import type { ResultsState, TabValue } from '$lib/app.svelte';

  let {
    activeTab = $bindable('results' as TabValue),
    results,
    resultsLoading,
    currentPage,
    pageSize,
    pageSizeSelect = $bindable(),
    filters,
    debouncedSearch,
    expandedIssueUrl = $bindable(),
    selectedProjectId,
    onPageSizeChange,
    onGoToPage,
    onOpenDetail,
    onSearch,
    onFilterChange,
    onFilterIssueType,
    onClearFilter,
  }: {
    activeTab: TabValue;
    results: ResultsState;
    resultsLoading: boolean;
    currentPage: number;
    pageSize: number;
    pageSizeSelect: string;
    filters: FilterState;
    debouncedSearch: string;
    expandedIssueUrl: string;
    selectedProjectId: string;
    onPageSizeChange: (size: number) => void;
    onGoToPage: (page: number) => void;
    onOpenDetail: (pageId: string) => void;
    onSearch: (query: string) => void;
    onFilterChange: (filters: FilterState) => void;
    onFilterIssueType: (issueType: string | null) => void;
    onClearFilter: () => void;
  } = $props();

  const totalPages = $derived(Math.ceil(results.total / pageSize));

  const components = $state<Partial<Record<LazyTabId, any>>>({});

  function tabLabel(id: TabValue, count: string): string {
    switch (id) {
      case 'results':
        return m['tabs.results']({ count });
      case 'overview':
        return m['tabs.overview']();
      case 'dashboard':
        return m['tabs.issues_dashboard']();
      case 'site_tree':
        return m['tabs.site_tree']();
      case 'comparator':
        return m['tabs.comparator']();
      case 'duplicates':
        return m['tabs.duplicates']();
      case 'keywords':
        return m['tabs.keywords']();
      case 'schedule':
        return m['tabs.schedule']();
      case 'seo':
        return m['tabs.seo']();
    }
  }

  $effect(() => {
    const id = activeTab;
    if (id !== 'results' && !components[id]) {
      loadTabComponent(id).then((c) => (components[id] = c));
    }
  });
</script>

<section class="results-section">
  <Tabs.Root bind:value={activeTab} class="mb-4">
    <Tabs.List>
      {#each TAB_DEFS as tab (tab.id)}
        <Tabs.Trigger value={tab.id}>
          {tabLabel(tab.id, results.total.toLocaleString())}
        </Tabs.Trigger>
      {/each}
    </Tabs.List>
  </Tabs.Root>

  {#if activeTab === 'results'}
    <div class="results-toolbar">
      <div class="page-size-selector">
        <Label for="pageSizeSelect">{m['results.page_size_show']()}</Label>
        <Select.Root
          type="single"
          bind:value={pageSizeSelect}
          onValueChange={(v) => {
            if (v) onPageSizeChange(parseInt(v, 10));
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
      {#if filters.issueType}
        <Badge variant="secondary" class="gap-1.5 px-3 py-1">
          {m['results.filtered_by']({ type: translateIssueName(filters.issueType) })}
          <Button
            variant="ghost"
            size="xs"
            class="btn-clear-filter size-5"
            onclick={onClearFilter}
            aria-label={m['results.clear_filter']()}
            title={m['results.clear_filter']()}
          >
            <X class="size-3" />
          </Button>
        </Badge>
      {/if}
    </div>

    <FilterBar items={results.items} {filters} onFilter={onFilterChange} />

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
        onDetail={onOpenDetail}
        searchQuery={debouncedSearch}
        {onSearch}
      />

      {#if totalPages > 1}
        <Pagination {currentPage} {totalPages} total={results.total} {pageSize} {onGoToPage} />
      {/if}
    {/if}
  {/if}

  <!-- Lazy tab panels are kept mounted once loaded (keep-alive) and only
       hidden, so switching tabs preserves each tab's scroll/filter state. -->
  <div class="tab-panel" hidden={activeTab !== 'overview'}>
    {#if components.overview}
      {@const Overview = components.overview}
      <Overview projectId={selectedProjectId} />
    {/if}
  </div>
  <div class="tab-panel" hidden={activeTab !== 'dashboard'}>
    {#if components.dashboard}
      {@const IssuesDashboard = components.dashboard}
      <IssuesDashboard
        projectId={selectedProjectId}
        {onFilterIssueType}
        activeFilter={filters.issueType}
      />
    {/if}
  </div>
  <div class="tab-panel" hidden={activeTab !== 'site_tree'}>
    {#if components.site_tree}
      {@const Tree = components.site_tree}
      <Tree projectId={selectedProjectId} />
    {/if}
  </div>
  <div class="tab-panel" hidden={activeTab !== 'comparator'}>
    {#if components.comparator}
      {@const Comparator = components.comparator}
      <Comparator projectId={selectedProjectId} />
    {/if}
  </div>
  <div class="tab-panel" hidden={activeTab !== 'duplicates'}>
    {#if components.duplicates}
      {@const Duplicates = components.duplicates}
      <Duplicates projectId={selectedProjectId} />
    {/if}
  </div>
  <div class="tab-panel" hidden={activeTab !== 'keywords'}>
    {#if components.keywords}
      {@const Keywords = components.keywords}
      <Keywords projectId={selectedProjectId} />
    {/if}
  </div>
  <div class="tab-panel" hidden={activeTab !== 'schedule'}>
    {#if components.schedule}
      {@const SchedulePanel = components.schedule}
      <SchedulePanel projectId={selectedProjectId} />
    {/if}
  </div>
  <div class="tab-panel" hidden={activeTab !== 'seo'}>
    {#if components.seo}
      {@const SiteSeo = components.seo}
      <SiteSeo projectId={selectedProjectId} />
    {/if}
  </div>
</section>

<style>
  section {
    background: var(--bg-card);
    border: none;
    border-radius: var(--radius-xl);
    padding: var(--space-lg);
    box-shadow: var(--neu-raised-md);
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

  .results-skeleton {
    display: flex;
    flex-direction: column;
    gap: 8px;
    border-radius: var(--radius-lg);
    border: none;
    padding: 12px;
    box-shadow: var(--neu-pressed-sm);
  }

  .tab-panel {
    border-radius: var(--radius-lg);
    border: none;
    padding: 12px;
    box-shadow: var(--neu-pressed-sm);
  }

  .tab-panel[hidden] {
    display: none !important;
  }

  @media (max-width: 767px) {
    section {
      padding: var(--section-padding);
      border-radius: var(--radius-lg);
    }

    .results-toolbar {
      flex-direction: column;
      align-items: flex-start;
      gap: var(--space-sm);
    }
  }

  @media (min-width: 768px) {
    .results-toolbar {
      flex-direction: row;
      justify-content: space-between;
    }
  }
</style>
