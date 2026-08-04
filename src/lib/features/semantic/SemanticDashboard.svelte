<script lang="ts">
  import { getSemanticIssueCounts } from '$lib/api/results';
  import type { IssueCount } from '$lib/api/types';
  import { m } from '$lib/paraglide/messages.js';
  import { translateIssueName } from '$lib/i18n-issues';
  import {
    ImageOff, FileText, Text, Link2, Unlink, Heading1, List, Globe, Bot,
    Blocks, Monitor, Copy, Accessibility, LayoutGrid, TextCursorInput,
    TriangleAlert, ChevronRight,
  } from 'lucide-svelte';
  import { Button } from '$lib/components/ui/button/index.js';
  import { Skeleton } from '$lib/components/ui/skeleton/index.js';
  import { cn } from '$lib/utils.js';

  let {
    projectId,
    onFilterIssueType,
    activeFilter = $bindable(''),
  }: {
    projectId: string;
    onFilterIssueType: (issueType: string | null) => void;
    activeFilter: string;
  } = $props();

  let issueCounts = $state<IssueCount[]>([]);
  let loading = $state(false);
  let error = $state('');

  type Icon = typeof ImageOff;

  const ISSUE_ICONS: Record<string, Icon> = {
    missing_alt: ImageOff,
    image_without_alt: ImageOff,
    missing_title: FileText,
    missing_meta_desc: Text,
    missing_canonical: Link2,
    empty_link: Unlink,
    link_empty: Unlink,
    missing_label: TextCursorInput,
    form_without_label: TextCursorInput,
    missing_h1: Heading1,
    multiple_h1: Heading1,
    heading_skip: List,
    skip_from_h1: List,
    missing_lang: Globe,
    missing_robots: Bot,
    invalid_nesting: Blocks,
    missing_viewport: Monitor,
    duplicate_id: Copy,
    missing_aria: Accessibility,
    missing_main: LayoutGrid,
    missing_header: LayoutGrid,
    missing_footer: LayoutGrid,
    missing_nav: LayoutGrid,
  };

  const DEFAULT_ICON: Icon = TriangleAlert;

  $effect(() => {
    if (projectId) loadCounts();
    else { issueCounts = []; }
  });

  async function loadCounts() {
    loading = true;
    error = '';
    try {
      const data = await getSemanticIssueCounts(projectId);
      issueCounts = data.filter((i) => i.severity === 'error');
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  function toggleFilter(issueType: string) {
    if (activeFilter === issueType) {
      activeFilter = '';
      onFilterIssueType(null);
    } else {
      activeFilter = issueType;
      onFilterIssueType(issueType);
    }
  }

  function clearFilter() {
    activeFilter = '';
    onFilterIssueType(null);
  }

  function formatIssueType(type: string): string {
    return translateIssueName(type);
  }

  function getIcon(issueType: string): Icon {
    return ISSUE_ICONS[issueType] || DEFAULT_ICON;
  }

  function getDominantSeverity(item: { error: number; warning: number; info: number }): string {
    if (item.error > 0) return 'error';
    if (item.warning > 0) return 'warning';
    if (item.info > 0) return 'info';
    return 'neutral';
  }

  let severityTotals = $derived(() => {
    const totals: Record<string, number> = { error: 0, warning: 0, info: 0 };
    for (const item of issueCounts) {
      totals[item.severity] = (totals[item.severity] || 0) + item.count;
    }
    return totals;
  });

  let totalIssues = $derived(() => {
    const t = severityTotals();
    return t.error + t.warning + t.info;
  });

  let groupedIssues = $derived(() => {
    const map = new Map<string, { issue_type: string; error: number; warning: number; info: number; total: number }>();
    for (const item of issueCounts) {
      const existing = map.get(item.issue_type) || { issue_type: item.issue_type, error: 0, warning: 0, info: 0, total: 0 };
      existing[item.severity as 'error' | 'warning' | 'info'] = item.count;
      existing.total += item.count;
      map.set(item.issue_type, existing);
    }
    return Array.from(map.values()).sort((a, b) => b.total - a.total);
  });
</script>

<div class="semantic-dashboard">
  <div class="dashboard-head">
    <h2>{m["dashboard.title"]()}</h2>
    {#if activeFilter}
      <Button variant="outline" size="sm" onclick={clearFilter}>
        {m["dashboard.clear_filter"]({ type: formatIssueType(activeFilter) })} &times;
      </Button>
    {/if}
  </div>

  <div class="dashboard-body">
    {#if loading}
      <div class="flex flex-col gap-2">
        <Skeleton class="h-12 w-full" />
        <Skeleton class="h-12 w-full" />
        <Skeleton class="h-12 w-3/4" />
      </div>
    {:else if error}
      <div class="dashboard-error">{error}</div>
    {:else if issueCounts.length === 0}
      <div class="dashboard-empty">{m["dashboard.no_issues"]()}</div>
    {:else}
      <!-- Severity Summary Bar -->
      <div class="severity-bar">
        {#if severityTotals().error > 0}
          <div
            class="severity-segment error"
            style="flex: {severityTotals().error}"
            title="{severityTotals().error} errors"
          >
            {severityTotals().error}
          </div>
        {/if}
        {#if severityTotals().warning > 0}
          <div
            class="severity-segment warning"
            style="flex: {severityTotals().warning}"
            title="{severityTotals().warning} warnings"
          >
            {severityTotals().warning}
          </div>
        {/if}
        {#if severityTotals().info > 0}
          <div
            class="severity-segment info"
            style="flex: {severityTotals().info}"
            title="{severityTotals().info} info"
          >
            {severityTotals().info}
          </div>
        {/if}
      </div>
      <div class="severity-legend">
        {#if severityTotals().error > 0}
          <span class="legend-item"><span class="legend-dot error"></span> {m["dashboard.errors"]({ count: severityTotals().error.toString() })}</span>
        {/if}
        {#if severityTotals().warning > 0}
          <span class="legend-item"><span class="legend-dot warning"></span> {m["dashboard.warnings"]({ count: severityTotals().warning.toString() })}</span>
        {/if}
        {#if severityTotals().info > 0}
          <span class="legend-item"><span class="legend-dot info"></span> {m["dashboard.info"]({ count: severityTotals().info.toString() })}</span>
        {/if}
        <span class="legend-total">{m["dashboard.total_issues"]({ count: totalIssues().toString() })}</span>
      </div>

      <!-- Issue Type Cards -->
      <div class="issue-cards">
        {#each groupedIssues() as item}
          {@const sev = getDominantSeverity(item)}
          {@const Icon = getIcon(item.issue_type)}
          <Button
            variant="ghost"
            class={cn(
              'h-auto w-full justify-start gap-3 rounded-xl border px-4 py-3 text-left transition-all',
              activeFilter === item.issue_type
                ? 'border-primary bg-card ring-1 ring-primary neu-pressed-sm'
                : 'border-transparent bg-background hover:bg-muted hover:text-foreground neu-raised-sm'
            )}
            onclick={() => toggleFilter(item.issue_type)}
          >
            <Icon class="card-icon sev-{sev}" />
            <span class="card-content">
              <span class="card-type">{formatIssueType(item.issue_type)}</span>
              <span class="card-counts">
                {#if item.error > 0}
                  <span class="count-badge error">{item.error}</span>
                {/if}
                {#if item.warning > 0}
                  <span class="count-badge warning">{item.warning}</span>
                {/if}
                {#if item.info > 0}
                  <span class="count-badge info">{item.info}</span>
                {/if}
                <span class="count-total">{item.total}</span>
              </span>
            </span>
            <ChevronRight class="card-arrow size-4" />
          </Button>
        {/each}
      </div>
    {/if}
  </div>
</div>

<style>
  .semantic-dashboard {
    display: flex;
    flex-direction: column;
  }

  .dashboard-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    margin-bottom: 16px;
  }

  .dashboard-head h2 {
    font-size: 1.15rem;
    color: var(--text);
    margin: 0;
  }

  .dashboard-body {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .dashboard-error,
  .dashboard-empty {
    padding: 24px;
    text-align: center;
    color: var(--text-muted);
    font-size: 0.9rem;
  }
  .dashboard-error { color: var(--danger); }

  /* Severity Bar */
  .severity-bar {
    display: flex;
    height: 32px;
    border-radius: 8px;
    overflow: hidden;
    gap: 2px;
  }

  .severity-segment {
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 0.75rem;
    font-weight: 700;
    color: white;
    min-width: 28px;
    transition: flex 0.3s ease;
  }
  .severity-segment.error { background: var(--danger); }
  .severity-segment.warning { background: var(--warning); color: var(--bg-deep); }
  .severity-segment.info { background: var(--info); color: var(--bg-deep); }

  .severity-legend {
    display: flex;
    align-items: center;
    gap: 16px;
    font-size: 0.8rem;
    color: var(--text-secondary);
    flex-wrap: wrap;
  }

  .legend-item {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .legend-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
  }
  .legend-dot.error { background: var(--danger); }
  .legend-dot.warning { background: var(--warning); }
  .legend-dot.info { background: var(--info); }

  .legend-total {
    margin-left: auto;
    color: var(--text-muted);
    font-style: italic;
  }

  /* Issue Cards */
  .issue-cards {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  :global(.card-icon) {
    width: 36px;
    height: 36px;
    padding: 7px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--bg-card);
    box-shadow: var(--neu-raised-sm);
    border-radius: 12px;
    flex-shrink: 0;
    color: var(--text-secondary);
  }
  :global(.card-icon.sev-error) {
    background: var(--bg-issue-error);
    color: var(--danger);
  }
  :global(.card-icon.sev-warning) {
    background: var(--bg-issue-warning);
    color: var(--warning);
  }
  :global(.card-icon.sev-info) {
    background: var(--bg-issue-info);
    color: var(--info);
  }

  .card-content {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }

  .card-type {
    font-size: 0.85rem;
    color: var(--text);
    font-weight: 500;
    margin-bottom: 4px;
  }

  .card-counts {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .count-badge {
    font-size: 0.7rem;
    font-weight: 700;
    padding: 1px 6px;
    border-radius: 4px;
  }
  .count-badge.error { background: var(--bg-issue-error); color: var(--danger); }
  .count-badge.warning { background: var(--bg-issue-warning); color: var(--warning); }
  .count-badge.info { background: var(--bg-issue-info); color: var(--info); }

  .count-total {
    font-size: 0.75rem;
    color: var(--text-muted);
    margin-left: 4px;
  }

  :global(.card-arrow) {
    color: var(--text-muted);
    flex-shrink: 0;
    transition: transform var(--transition-fast);
  }

  :global(.issue-cards Button:hover .card-arrow) {
    transform: translateX(2px);
  }
</style>
