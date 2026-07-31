<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { m } from '$lib/paraglide/messages.js';
  import { translateIssueName } from '$lib/i18n-issues';

  let {
    projectId,
    onFilterIssueType,
    activeFilter = $bindable(''),
  }: {
    projectId: string;
    onFilterIssueType: (issueType: string | null) => void;
    activeFilter: string;
  } = $props();

  let issueCounts = $state<any[]>([]);
  let loading = $state(false);
  let error = $state('');

  const ISSUE_ICONS: Record<string, string> = {
    missing_alt: '🖼',
    missing_title: '🏷',
    missing_meta_desc: '📝',
    missing_canonical: '🔗',
    empty_link: '🔗',
    missing_label: '📋',
    missing_h1: '📰',
    heading_skip: '🔤',
    missing_lang: '🌐',
    missing_robots: '🤖',
    multiple_h1: '📰',
    image_without_alt: '🖼',
    form_without_label: '📋',
    link_empty: '🔗',
    skip_from_h1: '🔤',
    invalid_nesting: '🧩',
    missing_viewport: '📱',
    duplicate_id: '🆔',
    missing_aria: '♿',
  };

  $effect(() => {
    if (projectId) loadCounts();
    else { issueCounts = []; }
  });

  async function loadCounts() {
    loading = true;
    error = '';
    try {
      const data = await invoke<any[]>('get_semantic_issue_counts', { projectId });
      issueCounts = data;
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

  function getSeverityColor(severity: string): string {
    if (severity === 'error') return '#ff6b6b';
    if (severity === 'warning') return '#ffd43b';
    return '#74c0fc';
  }

  function getSeverityBg(severity: string): string {
    if (severity === 'error') return '#3d1f1f';
    if (severity === 'warning') return '#3d3520';
    return '#1a2a3a';
  }

  function formatIssueType(type: string): string {
    return translateIssueName(type);
  }

  function getIcon(issueType: string): string {
    return ISSUE_ICONS[issueType] || '⚠';
  }

  // Aggregate counts by severity for the bar
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

  // Group issue types by unique type (combine severities)
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

<div class="dashboard">
  <div class="dashboard-header">
    <h3>{m["dashboard.title"]()}</h3>
    {#if activeFilter}
      <button class="btn-clear" onclick={clearFilter}>
        {m["dashboard.clear_filter"]({ type: formatIssueType(activeFilter) })} &times;
      </button>
    {/if}
  </div>

  {#if loading}
    <div class="dashboard-loading">{m["dashboard.loading"]()}</div>
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
      <span class="legend-item"><span class="legend-dot error"></span> {m["dashboard.errors"]({ count: severityTotals().error.toString() })}</span>
      <span class="legend-item"><span class="legend-dot warning"></span> {m["dashboard.warnings"]({ count: severityTotals().warning.toString() })}</span>
      <span class="legend-item"><span class="legend-dot info"></span> {m["dashboard.info"]({ count: severityTotals().info.toString() })}</span>
      <span class="legend-total">{m["dashboard.total_issues"]({ count: totalIssues().toString() })}</span>
    </div>

    <!-- Issue Type Cards -->
    <div class="issue-cards">
      {#each groupedIssues() as item}
        <button
          class="issue-card"
          class:active={activeFilter === item.issue_type}
          onclick={() => toggleFilter(item.issue_type)}
        >
          <div class="card-icon">{getIcon(item.issue_type)}</div>
          <div class="card-content">
            <div class="card-type">{formatIssueType(item.issue_type)}</div>
            <div class="card-counts">
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
            </div>
          </div>
          <div class="card-arrow">→</div>
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .dashboard {
    padding: 20px;
    background: var(--bg-card);
    border-radius: 12px;
  }

  .dashboard-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;
  }

  .dashboard-header h3 {
    margin: 0;
    font-size: 1.1rem;
    color: var(--text);
  }

  .btn-clear {
    padding: 6px 12px;
    background: var(--border);
    border: 1px solid var(--text-muted);
    border-radius: 6px;
    color: var(--text);
    font-size: 0.8rem;
    cursor: pointer;
  }
  .btn-clear:hover { background: #4a4d54; }

  .dashboard-loading, .dashboard-error, .dashboard-empty {
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
    margin-bottom: 8px;
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
    margin-bottom: 20px;
    font-size: 0.8rem;
    color: var(--text-secondary);
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

  .issue-card {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 16px;
    background: var(--bg-deep);
    border: 1px solid var(--bg-hover);
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.15s;
    text-align: left;
    width: 100%;
  }

  .issue-card:hover {
    border-color: var(--text-muted);
    background: var(--bg-card);
  }

  .issue-card.active {
    border-color: var(--accent);
    box-shadow: 0 0 0 1px var(--accent);
    background: #1e2030;
  }

  .card-icon {
    font-size: 1.4rem;
    width: 36px;
    height: 36px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--bg-hover);
    border-radius: 8px;
    flex-shrink: 0;
  }

  .card-content {
    flex: 1;
    min-width: 0;
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
  .count-badge.error { background: #3d1f1f; color: var(--danger); }
  .count-badge.warning { background: #3d3520; color: var(--warning); }
  .count-badge.info { background: #1a2a3a; color: var(--info); }

  .count-total {
    font-size: 0.75rem;
    color: var(--text-muted);
    margin-left: 4px;
  }

  .card-arrow {
    color: var(--text-muted);
    font-size: 0.9rem;
    flex-shrink: 0;
    transition: color 0.15s;
  }
  .issue-card:hover .card-arrow { color: var(--text-secondary); }
</style>
