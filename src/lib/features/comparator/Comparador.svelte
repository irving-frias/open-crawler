<script lang="ts">
  import { listCrawlSnapshots, compareCrawls } from '$lib/api/snapshots';
  import type { CompareResult, CrawlSnapshot } from '$lib/api/types';
  import { m } from '$lib/paraglide/messages.js';
  import { Button } from '$lib/components/ui/button/index.js';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card/index.js';
  import * as Select from '$lib/components/ui/select/index.js';
  import { Label } from '$lib/components/ui/label/index.js';
  import { Skeleton } from '$lib/components/ui/skeleton/index.js';
  import { Badge } from '$lib/components/ui/badge/index.js';

  let {
    projectId,
  }: {
    projectId: string;
  } = $props();

  let snapshots = $state<CrawlSnapshot[]>([]);
  let loading = $state(false);
  let comparing = $state(false);
  let error = $state('');

  let snapshotA = $state('');
  let snapshotB = $state('');

  let comparison = $state<CompareResult | null>(null);
  let snapshotsSeq = 0;
  const LIST_PAGE_SIZE = 100;
  let newUrlsLimit = $state(LIST_PAGE_SIZE);
  let removedUrlsLimit = $state(LIST_PAGE_SIZE);
  let changedUrlsLimit = $state(LIST_PAGE_SIZE);

  $effect(() => {
    if (projectId) {
      loadSnapshots();
    } else {
      snapshots = [];
      comparison = null;
    }
  });

  async function loadSnapshots() {
    const seq = ++snapshotsSeq;
    loading = true;
    error = '';
    try {
      const data = await listCrawlSnapshots(projectId);
      if (seq !== snapshotsSeq) return;
      snapshots = data;
      if (data.length >= 2) {
        snapshotA = data[data.length - 1].id;
        snapshotB = data[0].id;
      } else if (data.length === 1) {
        snapshotA = '';
        snapshotB = data[0].id;
      } else {
        snapshotA = '';
        snapshotB = '';
      }
      comparison = null;
    } catch (e) {
      if (seq === snapshotsSeq) error = String(e);
    } finally {
      if (seq === snapshotsSeq) loading = false;
    }
  }

  async function runCompare() {
    if (!snapshotA || !snapshotB || snapshotA === snapshotB) return;
    comparing = true;
    error = '';
    try {
      comparison = await compareCrawls(snapshotA, snapshotB);
    } catch (e) {
      error = String(e);
    } finally {
      comparing = false;
    }
  }

  function formatTime(iso: string): string {
    const d = new Date(iso);
    if (isNaN(d.getTime())) return iso;
    return d.toLocaleString();
  }

  function snapshotLabel(id: string): string {
    const s = snapshots.find((x) => x.id === id);
    if (!s) return '—';
    return `${formatTime(s.snapshot_time)} (${s.total_pages} pages)`;
  }

  function fieldName(field: string): string {
    const key = `comparator.field.${field}`;
    if (m[key as keyof typeof m]) return (m[key as keyof typeof m] as () => string)();
    return field.replace(/_/g, ' ');
  }

  function formatStat(stat: any): { pages: string; indexable: string; broken: string; load: string; size: string; readability: string } {
    return {
      pages: stat.total_pages.toLocaleString(),
      indexable: stat.indexed_pages.toLocaleString(),
      broken: stat.broken_pages.toLocaleString(),
      load: `${Math.round(stat.avg_load_ms).toLocaleString()} ms`,
      size: formatBytes(stat.avg_size_bytes),
      readability: stat.avg_readability != null ? String(Math.round(stat.avg_readability)) : '—',
    };
  }

  function formatBytes(bytes: number): string {
    if (bytes >= 1048576) return `${(bytes / 1048576).toFixed(1)} MB`;
    if (bytes >= 1024) return `${(bytes / 1024).toFixed(0)} KB`;
    return `${Math.round(bytes)} B`;
  }

  function diffValue(v: string | null | undefined): string {
    return v == null ? '—' : v;
  }

  $effect(() => {
    if (comparison) {
      newUrlsLimit = LIST_PAGE_SIZE;
      removedUrlsLimit = LIST_PAGE_SIZE;
      changedUrlsLimit = LIST_PAGE_SIZE;
    }
  });

  const visibleNewUrls = $derived(comparison?.new_urls.slice(0, newUrlsLimit) ?? []);
  const visibleRemovedUrls = $derived(comparison?.removed_urls.slice(0, removedUrlsLimit) ?? []);
  const visibleChangedUrls = $derived(comparison?.changed_urls.slice(0, changedUrlsLimit) ?? []);
</script>

<div class="comparator">
  {#if loading}
    <div class="comp-loading">
      <Skeleton class="h-24 w-full" />
    </div>
  {:else if error}
    <div class="comp-error">{error}</div>
  {:else if snapshots.length < 2}
    <div class="comp-empty">
      {m['comparator.no_snapshots']()}
    </div>
  {:else}
    <div class="comp-controls">
      <div class="comp-field">
        <Label>{m['comparator.snapshot_a']()}</Label>
        <Select.Root type="single" bind:value={snapshotA}>
          <Select.Trigger class="w-full justify-between">
            {snapshotA ? snapshotLabel(snapshotA) : '—'}
          </Select.Trigger>
          <Select.Content>
            {#each snapshots as s (s.id)}
              {#if s.id !== snapshotB}
                <Select.Item value={s.id}>
                  {formatTime(s.snapshot_time)} ({s.total_pages} pages)
                </Select.Item>
              {/if}
            {/each}
          </Select.Content>
        </Select.Root>
      </div>
      <div class="comp-field">
        <Label>{m['comparator.snapshot_b']()}</Label>
        <Select.Root type="single" bind:value={snapshotB}>
          <Select.Trigger class="w-full justify-between">
            {snapshotB ? snapshotLabel(snapshotB) : '—'}
          </Select.Trigger>
          <Select.Content>
            {#each snapshots as s (s.id)}
              {#if s.id !== snapshotA}
                <Select.Item value={s.id}>
                  {formatTime(s.snapshot_time)} ({s.total_pages} pages)
                </Select.Item>
              {/if}
            {/each}
          </Select.Content>
        </Select.Root>
      </div>
      <Button onclick={runCompare} disabled={!snapshotA || !snapshotB || snapshotA === snapshotB || comparing}>
        {comparing ? '…' : m['comparator.compare_btn']()}
      </Button>
    </div>

    {#if comparison}
      <div class="comp-summary">
        <Card>
          <CardContent>
            <div class="comp-summary-grid">
              <div class="comp-summary-item comp-new">
                <span class="comp-summary-value">{comparison.new_urls.length.toLocaleString()}</span>
                <span class="comp-summary-label">{m['comparator.new']()}</span>
              </div>
              <div class="comp-summary-item comp-removed">
                <span class="comp-summary-value">{comparison.removed_urls.length.toLocaleString()}</span>
                <span class="comp-summary-label">{m['comparator.removed']()}</span>
              </div>
              <div class="comp-summary-item comp-changed">
                <span class="comp-summary-value">{comparison.changed_urls.length.toLocaleString()}</span>
                <span class="comp-summary-label">{m['comparator.changed']()}</span>
              </div>
              <div class="comp-summary-item">
                <span class="comp-summary-value">{comparison.unchanged_count.toLocaleString()}</span>
                <span class="comp-summary-label">{m['comparator.unchanged']()}</span>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>

      <div class="comp-stats">
        <Card>
          <CardHeader>
            <CardTitle>{m['comparator.stats_before']()}</CardTitle>
          </CardHeader>
          <CardContent>
            {@const s = formatStat(comparison.before)}
            <ul class="comp-stat-list">
              <li>{m['dashboard.total_pages']()}: <strong>{s.pages}</strong></li>
              <li>{m['dashboard.indexed_pages']()}: <strong>{s.indexable}</strong></li>
              <li>{m['dashboard.broken_pages']()}: <strong>{s.broken}</strong></li>
              <li>{m['dashboard.avg_load']()}: <strong>{s.load}</strong></li>
              <li>{m['dashboard.avg_size']()}: <strong>{s.size}</strong></li>
              <li>{m['dashboard.avg_readability']()}: <strong>{s.readability}</strong></li>
            </ul>
          </CardContent>
        </Card>
        <Card>
          <CardHeader>
            <CardTitle>{m['comparator.stats_after']()}</CardTitle>
          </CardHeader>
          <CardContent>
            {@const s = formatStat(comparison.after)}
            <ul class="comp-stat-list">
              <li>{m['dashboard.total_pages']()}: <strong>{s.pages}</strong></li>
              <li>{m['dashboard.indexed_pages']()}: <strong>{s.indexable}</strong></li>
              <li>{m['dashboard.broken_pages']()}: <strong>{s.broken}</strong></li>
              <li>{m['dashboard.avg_load']()}: <strong>{s.load}</strong></li>
              <li>{m['dashboard.avg_size']()}: <strong>{s.size}</strong></li>
              <li>{m['dashboard.avg_readability']()}: <strong>{s.readability}</strong></li>
            </ul>
          </CardContent>
        </Card>
      </div>

      <div class="comp-lists">
        {#if comparison.new_urls.length > 0}
          <Card>
            <CardHeader>
              <CardTitle>{m['comparator.new']()} ({comparison.new_urls.length})</CardTitle>
            </CardHeader>
            <CardContent>
              <ul class="comp-url-list">
                {#each visibleNewUrls as url}
                  <li class="comp-url-new">{url}</li>
                {/each}
              </ul>
              {#if comparison.new_urls.length > newUrlsLimit}
                <Button variant="ghost" size="sm" class="mt-2" onclick={() => (newUrlsLimit += LIST_PAGE_SIZE)}>
                  {m['comparator.show_more']()}
                </Button>
              {/if}
            </CardContent>
          </Card>
        {/if}

        {#if comparison.removed_urls.length > 0}
          <Card>
            <CardHeader>
              <CardTitle>{m['comparator.removed']()} ({comparison.removed_urls.length})</CardTitle>
            </CardHeader>
            <CardContent>
              <ul class="comp-url-list">
                {#each visibleRemovedUrls as url}
                  <li class="comp-url-removed">{url}</li>
                {/each}
              </ul>
              {#if comparison.removed_urls.length > removedUrlsLimit}
                <Button variant="ghost" size="sm" class="mt-2" onclick={() => (removedUrlsLimit += LIST_PAGE_SIZE)}>
                  {m['comparator.show_more']()}
                </Button>
              {/if}
            </CardContent>
          </Card>
        {/if}

        {#if comparison.changed_urls.length > 0}
          <Card>
            <CardHeader>
              <CardTitle>{m['comparator.changed']()} ({comparison.changed_urls.length})</CardTitle>
            </CardHeader>
            <CardContent>
              <ul class="comp-change-list">
                {#each visibleChangedUrls as change (change.url)}
                  <li>
                    <div class="comp-change-url">{change.url}</div>
                    <ul class="comp-diff-list">
                      {#each change.diffs as diff (diff.field)}
                        <li class="comp-diff-item">
                          <span class="comp-diff-field">{fieldName(diff.field)}</span>
                          <span class="comp-diff-before">{diffValue(diff.before)}</span>
                          <span class="comp-diff-arrow">→</span>
                          <span class="comp-diff-after">{diffValue(diff.after)}</span>
                        </li>
                      {/each}
                    </ul>
                  </li>
                {/each}
              </ul>
              {#if comparison.changed_urls.length > changedUrlsLimit}
                <Button variant="ghost" size="sm" class="mt-2" onclick={() => (changedUrlsLimit += LIST_PAGE_SIZE)}>
                  {m['comparator.show_more']()}
                </Button>
              {/if}
            </CardContent>
          </Card>
        {/if}
      </div>
    {/if}
  {/if}
</div>

<style>
  .comparator {
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  .comp-controls {
    display: flex;
    gap: 16px;
    align-items: flex-end;
    flex-wrap: wrap;
  }

  .comp-field {
    display: flex;
    flex-direction: column;
    gap: 6px;
    flex: 1;
    min-width: 220px;
  }

  .comp-summary-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
    gap: 12px;
  }

  .comp-summary-item {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .comp-summary-value {
    font-size: 1.6rem;
    font-weight: 700;
    color: var(--text);
    font-variant-numeric: tabular-nums;
  }

  .comp-summary-label {
    font-size: 0.8rem;
    color: var(--text-muted);
  }

  .comp-new .comp-summary-value { color: var(--success); }
  .comp-removed .comp-summary-value { color: var(--danger); }
  .comp-changed .comp-summary-value { color: var(--warning); }

  .comp-stats {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;
  }

  .comp-stat-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
    font-size: 0.9rem;
    color: var(--text-secondary);
  }

  .comp-stat-list strong {
    color: var(--text);
  }

  .comp-lists {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .comp-url-list,
  .comp-change-list,
  .comp-diff-list {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .comp-url-list {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .comp-url-list li {
    font-size: 0.9rem;
    word-break: break-all;
    padding: 4px 0;
  }

  .comp-url-new { color: var(--success); }
  .comp-url-removed { color: var(--danger); text-decoration: line-through; }

  .comp-change-list {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .comp-change-url {
    font-weight: 600;
    color: var(--text);
    font-size: 0.9rem;
    word-break: break-all;
  }

  .comp-diff-list {
    margin-top: 6px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .comp-diff-item {
    display: flex;
    align-items: baseline;
    gap: 8px;
    font-size: 0.85rem;
  }

  .comp-diff-field {
    color: var(--text-muted);
    min-width: 130px;
  }

  .comp-diff-before {
    color: var(--danger);
    word-break: break-all;
    max-width: 40%;
  }

  .comp-diff-after {
    color: var(--success);
    word-break: break-all;
    max-width: 40%;
  }

  .comp-diff-arrow {
    color: var(--text-muted);
  }

  .comp-empty,
  .comp-error {
    padding: 40px;
    text-align: center;
    color: var(--text-muted);
    background: var(--bg-card);
    border: none;
    border-radius: var(--radius-lg);
    box-shadow: var(--neu-pressed-sm);
  }

  .comp-error {
    color: var(--danger);
  }

  .comp-loading {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  @media (max-width: 800px) {
    .comp-stats {
      grid-template-columns: 1fr;
    }
  }
</style>
