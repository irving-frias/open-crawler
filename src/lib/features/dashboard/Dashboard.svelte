<script lang="ts">
  import { getDashboardStats } from '$lib/api/analytics';
  import type { DashboardStats as DashboardStatsT } from '$lib/api/types';
  import { m } from '$lib/paraglide/messages.js';
  import { translateIssueName, translateSeverity } from '$lib/i18n-issues';
  import DonutChart from '$lib/components/charts/DonutChart.svelte';
  import BarChart from '$lib/components/charts/BarChart.svelte';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card/index.js';
  import { Skeleton } from '$lib/components/ui/skeleton/index.js';

  let {
    projectId,
  }: {
    projectId: string;
  } = $props();

  let stats = $state<DashboardStatsT | null>(null);
  let loading = $state(false);
  let error = $state('');
  let statsSeq = 0;

  const STATUS_COLORS: Record<string, string> = {
    '2xx': 'var(--success)',
    '3xx': 'var(--info, #3b82f6)',
    '4xx': 'var(--warning)',
    '5xx': 'var(--danger)',
  };

  $effect(() => {
    if (projectId) loadStats();
    else stats = null;
  });

  async function loadStats() {
    const seq = ++statsSeq;
    loading = true;
    error = '';
    try {
      const data = await getDashboardStats(projectId);
      if (seq !== statsSeq) return;
      stats = data;
    } catch (e) {
      if (seq === statsSeq) error = String(e);
    } finally {
      if (seq === statsSeq) loading = false;
    }
  }

  let statusSegments = $derived(
    (stats?.status_distribution ?? []).map((s: { status: number; count: number }) => {
      const code = s.status;
      const bucket = code >= 500 ? '5xx' : code >= 400 ? '4xx' : code >= 300 ? '3xx' : '2xx';
      return {
        label: String(code),
        value: s.count,
        color: STATUS_COLORS[bucket] ?? 'var(--text-muted)',
      };
    })
  );

  let issueItems = $derived(
    (stats?.top_issues ?? [])
      .filter((i: { issue_type: string; severity: string; count: number }) => i.severity === 'error')
      .slice(0, 10)
      .map((i: { issue_type: string; severity: string; count: number }) => ({
        label: translateIssueName(i.issue_type),
        value: i.count,
        color:
          i.severity === 'error'
            ? 'var(--danger)'
            : i.severity === 'warning'
              ? 'var(--warning)'
              : 'var(--text-muted)',
      }))
  );

  function formatBytes(bytes: number): string {
    if (bytes >= 1048576) return `${(bytes / 1048576).toFixed(1)} MB`;
    if (bytes >= 1024) return `${(bytes / 1024).toFixed(0)} KB`;
    return `${Math.round(bytes)} B`;
  }
</script>

{#if loading && !stats}
  <div class="dash-loading">
    <Skeleton class="h-24 w-full" />
    <Skeleton class="h-24 w-full" />
    <Skeleton class="h-24 w-full" />
  </div>
{:else if error && !stats}
  <div class="dash-error">{error}</div>
{:else if stats && stats.total_pages === 0}
  <div class="dash-empty">{m['dashboard.no_data']()}</div>
{:else if stats}
  <div class="dashboard">
    <div class="stat-grid">
      <div class="stat-card">
        <span class="stat-label">{m['dashboard.total_pages']()}</span>
        <span class="stat-value">{stats.total_pages.toLocaleString()}</span>
      </div>
      <div class="stat-card">
        <span class="stat-label">{m['dashboard.indexed_pages']()}</span>
        <span class="stat-value">{stats.indexed_pages.toLocaleString()}</span>
      </div>
      <div class="stat-card stat-danger">
        <span class="stat-label">{m['dashboard.broken_pages']()}</span>
        <span class="stat-value">{stats.broken_pages.toLocaleString()}</span>
      </div>
      <div class="stat-card stat-warning">
        <span class="stat-label">{m['dashboard.blocked_pages']()}</span>
        <span class="stat-value">{stats.blocked_pages.toLocaleString()}</span>
      </div>
      <div class="stat-card stat-warning">
        <span class="stat-label">{m['dashboard.duplicates']()}</span>
        <span class="stat-value">{stats.duplicate_count.toLocaleString()}</span>
      </div>
      <div class="stat-card">
        <span class="stat-label">{m['dashboard.avg_load']()}</span>
        <span class="stat-value">{Math.round(stats.avg_load_ms).toLocaleString()}{m['dashboard.unit_ms']()}</span>
      </div>
      <div class="stat-card">
        <span class="stat-label">{m['dashboard.avg_size']()}</span>
        <span class="stat-value">{formatBytes(stats.avg_size_bytes)}</span>
      </div>
      <div class="stat-card">
        <span class="stat-label">{m['dashboard.avg_readability']()}</span>
        <span class="stat-value">
          {stats.avg_readability != null ? Math.round(stats.avg_readability) : '—'}
        </span>
      </div>
    </div>

    <div class="dash-row">
      <Card>
        <CardHeader>
          <CardTitle>{m['dashboard.status_distribution']()}</CardTitle>
        </CardHeader>
        <CardContent>
          <div class="chart-center">
            <DonutChart
              segments={statusSegments}
              centerValue={stats.total_pages.toLocaleString()}
              centerLabel={m['dashboard.total_pages']()}
            />
            {#if statusSegments.length > 0}
              <ul class="legend">
                {#each statusSegments as seg}
                  <li>
                    <span class="legend-dot" style={`background: ${seg.color}`}></span>
                    <span>{seg.label}</span>
                    <span class="legend-count">{seg.value.toLocaleString()}</span>
                  </li>
                {/each}
              </ul>
            {/if}
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>{m['dashboard.top_issues']()}</CardTitle>
        </CardHeader>
        <CardContent>
          {#if issueItems.length > 0}
            <BarChart items={issueItems} />
          {:else}
            <p class="empty-hint">{m['dashboard.no_issues']()}</p>
          {/if}
        </CardContent>
      </Card>
    </div>

    <div class="dash-row">
      <Card>
        <CardHeader>
          <CardTitle>{m['dashboard.content_health']()}</CardTitle>
        </CardHeader>
        <CardContent>
          <div class="health-row">
            <span class="health-item">
              <span class="sev-warning health-badge">{stats.missing_title_count}</span>
              {m['dashboard.missing_title']()}
            </span>
            <span class="health-item">
              <span class="sev-warning health-badge">{stats.missing_description_count}</span>
              {m['dashboard.missing_description']()}
            </span>
            <span class="health-item">
              <span class="sev-warning health-badge">{stats.missing_h1_count}</span>
              {m['dashboard.missing_h1']()}
            </span>
          </div>
        </CardContent>
      </Card>
    </div>
  </div>
{/if}

<style>
  .dashboard {
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  .stat-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
    gap: 12px;
  }

  .stat-card {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 16px;
    background: var(--bg-card);
    border: none;
    border-radius: var(--radius-lg);
    box-shadow: var(--neu-raised-md);
  }

  .stat-label {
    font-size: 0.8rem;
    color: var(--text-muted);
  }

  .stat-value {
    font-size: 1.5rem;
    font-weight: 700;
    color: var(--text);
    font-variant-numeric: tabular-nums;
  }

  .stat-danger .stat-value {
    color: var(--danger);
  }

  .stat-warning .stat-value {
    color: var(--warning);
  }

  .dash-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;
  }

  .chart-center {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 16px;
  }

  .legend {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 6px;
    width: 100%;
    max-width: 220px;
  }

  .legend li {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 0.85rem;
    color: var(--text-secondary);
  }

  .legend-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .legend-count {
    margin-left: auto;
    color: var(--text);
    font-weight: 600;
  }

  .health-row {
    display: flex;
    flex-wrap: wrap;
    gap: 24px;
  }

  .health-item {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 0.9rem;
    color: var(--text-secondary);
  }

  .health-badge {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 28px;
    height: 28px;
    padding: 0 8px;
    border-radius: 999px;
    font-weight: 700;
    font-size: 0.85rem;
  }

  .sev-warning {
    background: var(--warning-subtle);
    color: var(--warning);
  }

  .empty-hint {
    color: var(--text-muted);
    font-size: 0.9rem;
  }

  .dash-empty,
  .dash-error {
    padding: 40px;
    text-align: center;
    color: var(--text-muted);
    background: var(--bg-card);
    border: none;
    border-radius: var(--radius-lg);
    box-shadow: var(--neu-pressed-sm);
  }

  .dash-error {
    color: var(--danger);
  }

  .dash-loading {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  @media (max-width: 900px) {
    .dash-row {
      grid-template-columns: 1fr;
    }
  }
</style>
