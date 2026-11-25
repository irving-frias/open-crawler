<script lang="ts">
  import {
    listCrawlSnapshots,
    compareCrawlsPage,
    getSnapshotRetention,
    setSnapshotRetention,
  } from '#lib/api/snapshots.ts';
  import type { ChangedUrl, CrawlSnapshot, SnapshotStats } from '#lib/api/types.ts';
  import { m } from '#lib/paraglide/messages.js';
  import { toast } from 'svelte-sonner';
  import { Button } from '#lib/components/ui/button/index.js';
  import { Card, CardContent, CardHeader, CardTitle } from '#lib/components/ui/card/index.js';
  import * as Select from '#lib/components/ui/select/index.js';
  import { Label } from '#lib/components/ui/label/index.js';
  import { Skeleton } from '#lib/components/ui/skeleton/index.js';
  import { navigation } from '#lib/navigation.svelte.ts';

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

  // Per-project snapshot retention (T15).
  let retention = $state(12);
  let retentionSaving = $state(false);

  async function loadRetention() {
    try {
      retention = await getSnapshotRetention(projectId);
    } catch {
      // Keep the default; the control still works on save.
    }
  }

  async function saveRetention() {
    const value = Math.min(100, Math.max(1, Math.round(retention) || 12));
    retentionSaving = true;
    try {
      retention = await setSnapshotRetention(projectId, value);
      toast.success(m['snapshot.retention_saved']({ count: retention.toString() }));
      await loadSnapshots();
    } catch (e) {
      toast.error(String(e));
    } finally {
      retentionSaving = false;
    }
  }

  const PAGE_SIZE = 100;
  type SectionState = { total: number; items: string[]; page: number; loading: boolean };
  let newState = $state<SectionState>({ total: 0, items: [], page: 0, loading: false });
  let removedState = $state<SectionState>({ total: 0, items: [], page: 0, loading: false });
  let changedState = $state<{ total: number; items: ChangedUrl[]; page: number; loading: boolean }>(
    {
      total: 0,
      items: [],
      page: 0,
      loading: false,
    }
  );
  let unchangedCount = $state(0);
  let before = $state<SnapshotStats | null>(null);
  let after = $state<SnapshotStats | null>(null);
  let snapshotsSeq = 0;

  // Visual diff filters (T19, 1.6): which section cards show, and which
  // changed URLs show inside the changed card.
  let sectionFilter = $state<'all' | 'new' | 'removed' | 'changed'>('all');
  let changedFilter = $state<'all' | 'new-errors' | 'resolved' | 'other'>('all');

  /** Kind of a changed URL from its status_code diff (T19, 1.6). */
  function changeKind(change: ChangedUrl): 'new-errors' | 'resolved' | 'other' {
    const status = change.diffs.find((d) => d.field === 'status_code');
    if (!status) return 'other';
    const beforeCode = Number(status.before);
    const afterCode = Number(status.after);
    if (Number.isNaN(beforeCode) || Number.isNaN(afterCode)) return 'other';
    const wasError = beforeCode >= 400;
    const isError = afterCode >= 400;
    if (!wasError && isError) return 'new-errors';
    if (wasError && !isError) return 'resolved';
    return 'other';
  }

  const visibleChanged = $derived(
    changedFilter === 'all'
      ? changedState.items
      : changedState.items.filter((c) => changeKind(c) === changedFilter)
  );
  const newErrorsCount = $derived(
    changedState.items.filter((c) => changeKind(c) === 'new-errors').length
  );
  const resolvedCount = $derived(
    changedState.items.filter((c) => changeKind(c) === 'resolved').length
  );

  $effect(() => {
    if (projectId) {
      loadSnapshots();
      loadRetention();
    } else {
      snapshots = [];
      resetComparison();
    }
  });

  function resetComparison() {
    newState = { total: 0, items: [], page: 0, loading: false };
    removedState = { total: 0, items: [], page: 0, loading: false };
    changedState = { total: 0, items: [], page: 0, loading: false };
    unchangedCount = 0;
    before = null;
    after = null;
  }

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
      resetComparison();
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
    resetComparison();
    try {
      const [n, r, c] = await Promise.all([
        compareCrawlsPage(snapshotA, snapshotB, 'new', 1, PAGE_SIZE),
        compareCrawlsPage(snapshotA, snapshotB, 'removed', 1, PAGE_SIZE),
        compareCrawlsPage(snapshotA, snapshotB, 'changed', 1, PAGE_SIZE),
      ]);
      newState = { total: n.total, items: n.new_urls, page: 1, loading: false };
      removedState = { total: r.total, items: r.removed_urls, page: 1, loading: false };
      changedState = { total: c.total, items: c.changed_urls, page: 1, loading: false };
      unchangedCount = n.unchanged_count;
      before = n.before;
      after = n.after;
    } catch (e) {
      error = String(e);
    } finally {
      comparing = false;
    }
  }

  async function showMore(section: 'new' | 'removed' | 'changed') {
    if (!snapshotA || !snapshotB) return;
    const s = section === 'new' ? newState : section === 'removed' ? removedState : changedState;
    if (s.loading || s.page * PAGE_SIZE >= s.total) return;
    s.loading = true;
    try {
      const r = await compareCrawlsPage(snapshotA, snapshotB, section, s.page + 1, PAGE_SIZE);
      if (section === 'new') {
        newState = {
          ...newState,
          items: [...newState.items, ...r.new_urls],
          page: s.page + 1,
          loading: false,
        };
      } else if (section === 'removed') {
        removedState = {
          ...removedState,
          items: [...removedState.items, ...r.removed_urls],
          page: s.page + 1,
          loading: false,
        };
      } else {
        changedState = {
          ...changedState,
          items: [...changedState.items, ...r.changed_urls],
          page: s.page + 1,
          loading: false,
        };
      }
      unchangedCount = r.unchanged_count;
    } catch (e) {
      error = String(e);
      s.loading = false;
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

  function formatStat(stat: any): {
    pages: string;
    indexable: string;
    broken: string;
    load: string;
    size: string;
    readability: string;
  } {
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

  function hasMore(s: { total: number; page: number; loading: boolean }): boolean {
    return !s.loading && s.page > 0 && s.page * PAGE_SIZE < s.total;
  }
</script>

<div class="comparator">
  <div class="comp-field comp-retention">
    <Label for="snapshot-retention">{m['snapshot.retention']()}</Label>
    <input
      id="snapshot-retention"
      type="number"
      min="1"
      max="100"
      bind:value={retention}
      disabled={retentionSaving}
      onchange={saveRetention}
    />
    <span class="comp-retention-hint">{m['snapshot.retention_hint']()}</span>
  </div>
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
      <Button
        onclick={runCompare}
        disabled={!snapshotA || !snapshotB || snapshotA === snapshotB || comparing}
      >
        {comparing ? '…' : m['comparator.compare_btn']()}
      </Button>
    </div>

    {#if before && after}
      <div class="comp-summary">
        <Card>
          <CardContent>
            <div class="comp-summary-grid">
              <div class="comp-summary-item comp-new">
                <span class="comp-summary-value">{newState.total.toLocaleString()}</span>
                <span class="comp-summary-label">{m['comparator.new']()}</span>
              </div>
              <div class="comp-summary-item comp-removed">
                <span class="comp-summary-value">{removedState.total.toLocaleString()}</span>
                <span class="comp-summary-label">{m['comparator.removed']()}</span>
              </div>
              <div class="comp-summary-item comp-changed">
                <span class="comp-summary-value">{changedState.total.toLocaleString()}</span>
                <span class="comp-summary-label">{m['comparator.changed']()}</span>
              </div>
              <div class="comp-summary-item">
                <span class="comp-summary-value">{unchangedCount.toLocaleString()}</span>
                <span class="comp-summary-label">{m['comparator.unchanged']()}</span>
              </div>
              <div class="comp-summary-item comp-new-errors">
                <span class="comp-summary-value">{newErrorsCount.toLocaleString()}</span>
                <span class="comp-summary-label">{m['comparator.new_errors']()}</span>
              </div>
              <div class="comp-summary-item comp-resolved">
                <span class="comp-summary-value">{resolvedCount.toLocaleString()}</span>
                <span class="comp-summary-label">{m['comparator.resolved_errors']()}</span>
              </div>
            </div>
            {#if hasMore(changedState)}
              <p class="comp-summary-note">{m['comparator.errors_page_note']()}</p>
            {/if}
          </CardContent>
        </Card>
      </div>

      <div class="comp-filters" role="group" aria-label={m['comparator.filter_label']()}>
        <Button
          variant={sectionFilter === 'all' ? 'default' : 'outline'}
          size="sm"
          onclick={() => (sectionFilter = 'all')}
        >
          {m['comparator.filter_all']()}
        </Button>
        <Button
          variant={sectionFilter === 'new' ? 'default' : 'outline'}
          size="sm"
          onclick={() => (sectionFilter = 'new')}
        >
          {m['comparator.new']()} ({newState.total})
        </Button>
        <Button
          variant={sectionFilter === 'removed' ? 'default' : 'outline'}
          size="sm"
          onclick={() => (sectionFilter = 'removed')}
        >
          {m['comparator.removed']()} ({removedState.total})
        </Button>
        <Button
          variant={sectionFilter === 'changed' ? 'default' : 'outline'}
          size="sm"
          onclick={() => (sectionFilter = 'changed')}
        >
          {m['comparator.changed']()} ({changedState.total})
        </Button>
        {#if sectionFilter === 'changed'}
          <span class="comp-filter-sep"></span>
          <Button
            variant={changedFilter === 'all' ? 'secondary' : 'ghost'}
            size="sm"
            onclick={() => (changedFilter = 'all')}
          >
            {m['comparator.changed_all']()}
          </Button>
          <Button
            variant={changedFilter === 'new-errors' ? 'secondary' : 'ghost'}
            size="sm"
            onclick={() => (changedFilter = 'new-errors')}
          >
            {m['comparator.changed_new_errors']()} ({newErrorsCount})
          </Button>
          <Button
            variant={changedFilter === 'resolved' ? 'secondary' : 'ghost'}
            size="sm"
            onclick={() => (changedFilter = 'resolved')}
          >
            {m['comparator.changed_resolved']()} ({resolvedCount})
          </Button>
          <Button
            variant={changedFilter === 'other' ? 'secondary' : 'ghost'}
            size="sm"
            onclick={() => (changedFilter = 'other')}
          >
            {m['comparator.changed_other']()}
          </Button>
        {/if}
      </div>

      <div class="comp-stats">
        <Card>
          <CardHeader>
            <CardTitle>{m['comparator.stats_before']()}</CardTitle>
          </CardHeader>
          <CardContent>
            {@const s = formatStat(before)}
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
            {@const s = formatStat(after)}
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
        {#if newState.total > 0 && (sectionFilter === 'all' || sectionFilter === 'new')}
          <Card class="comp-card comp-card-new">
            <CardHeader>
              <CardTitle>{m['comparator.new']()} ({newState.total})</CardTitle>
            </CardHeader>
            <CardContent>
              <ul class="comp-url-list">
                {#each newState.items as url (url)}
                  <li class="comp-url-new">
                    <button
                      type="button"
                      class="comp-url-link"
                      title={m['nav.view_detail']()}
                      onclick={() => navigation.goToDetailByUrl(url)}
                    >
                      {url}
                    </button>
                  </li>
                {/each}
              </ul>
              {#if hasMore(newState)}
                <Button
                  variant="ghost"
                  size="sm"
                  class="mt-2"
                  onclick={() => showMore('new')}
                  disabled={newState.loading}
                >
                  {newState.loading ? '…' : m['comparator.show_more']()}
                </Button>
              {/if}
            </CardContent>
          </Card>
        {/if}

        {#if removedState.total > 0 && (sectionFilter === 'all' || sectionFilter === 'removed')}
          <Card class="comp-card comp-card-removed">
            <CardHeader>
              <CardTitle>{m['comparator.removed']()} ({removedState.total})</CardTitle>
            </CardHeader>
            <CardContent>
              <ul class="comp-url-list">
                {#each removedState.items as url (url)}
                  <li class="comp-url-removed">
                    <button
                      type="button"
                      class="comp-url-link"
                      title={m['nav.view_detail']()}
                      onclick={() => navigation.goToDetailByUrl(url)}
                    >
                      {url}
                    </button>
                  </li>
                {/each}
              </ul>
              {#if hasMore(removedState)}
                <Button
                  variant="ghost"
                  size="sm"
                  class="mt-2"
                  onclick={() => showMore('removed')}
                  disabled={removedState.loading}
                >
                  {removedState.loading ? '…' : m['comparator.show_more']()}
                </Button>
              {/if}
            </CardContent>
          </Card>
        {/if}

        {#if changedState.total > 0 && (sectionFilter === 'all' || sectionFilter === 'changed')}
          <Card class="comp-card comp-card-changed">
            <CardHeader>
              <CardTitle>{m['comparator.changed']()} ({changedState.total})</CardTitle>
            </CardHeader>
            <CardContent>
              {#if visibleChanged.length === 0}
                <p class="comp-empty-filter">{m['comparator.changed_empty_filter']()}</p>
              {:else}
                <ul class="comp-change-list">
                  {#each visibleChanged as change (change.url)}
                    {@const kind = changeKind(change)}
                    <li class="comp-change-item comp-change-{kind}">
                      <button
                        type="button"
                        class="comp-change-url comp-url-link"
                        title={m['nav.view_detail']()}
                        onclick={() => navigation.goToDetailByUrl(change.url)}
                      >
                        {change.url}
                      </button>
                      {#if kind === 'new-errors'}
                        <span class="comp-kind-badge comp-kind-new-errors">
                          {m['comparator.changed_new_errors']()}
                        </span>
                      {:else if kind === 'resolved'}
                        <span class="comp-kind-badge comp-kind-resolved">
                          {m['comparator.changed_resolved']()}
                        </span>
                      {/if}
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
                {#if hasMore(changedState)}
                  <Button
                    variant="ghost"
                    size="sm"
                    class="mt-2"
                    onclick={() => showMore('changed')}
                    disabled={changedState.loading}
                  >
                    {changedState.loading ? '…' : m['comparator.show_more']()}
                  </Button>
                {/if}
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

  .comp-retention {
    flex: none;
    max-width: 320px;
  }

  .comp-retention input {
    width: 90px;
    padding: 6px 8px;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: transparent;
    color: var(--text);
    font: inherit;
  }

  .comp-retention-hint {
    font-size: 0.8rem;
    color: var(--text-muted);
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

  .comp-new .comp-summary-value {
    color: var(--success);
  }
  .comp-removed .comp-summary-value {
    color: var(--danger);
  }
  .comp-changed .comp-summary-value {
    color: var(--warning);
  }
  .comp-new-errors .comp-summary-value {
    color: var(--danger);
  }
  .comp-resolved .comp-summary-value {
    color: var(--success);
  }

  .comp-summary-note {
    margin-top: 8px;
    font-size: 0.8rem;
    color: var(--text-muted);
  }

  .comp-filters {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px;
  }

  .comp-filter-sep {
    width: 1px;
    height: 20px;
    background: var(--border);
    margin: 0 4px;
  }

  .comp-card {
    border-left: 3px solid transparent;
  }
  .comp-card-new {
    border-left-color: var(--success);
  }
  .comp-card-removed {
    border-left-color: var(--danger);
  }
  .comp-card-changed {
    border-left-color: var(--warning);
  }

  .comp-change-item {
    border-left: 3px solid transparent;
    padding-left: 8px;
  }
  .comp-change-new-errors {
    border-left-color: var(--danger);
  }
  .comp-change-resolved {
    border-left-color: var(--success);
  }

  .comp-kind-badge {
    display: inline-block;
    font-size: 0.7rem;
    font-weight: 600;
    padding: 1px 6px;
    border-radius: 999px;
    margin-left: 8px;
    vertical-align: middle;
  }
  .comp-kind-new-errors {
    color: var(--danger);
    background: color-mix(in srgb, var(--danger) 12%, transparent);
  }
  .comp-kind-resolved {
    color: var(--success);
    background: color-mix(in srgb, var(--success) 12%, transparent);
  }

  .comp-empty-filter {
    font-size: 0.85rem;
    color: var(--text-muted);
  }

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

  .comp-url-new {
    color: var(--success);
  }
  .comp-url-removed {
    color: var(--danger);
    text-decoration: line-through;
  }

  .comp-url-link {
    background: none;
    border: none;
    padding: 0;
    cursor: pointer;
    font: inherit;
    color: inherit;
    text-align: left;
    word-break: break-all;
  }

  .comp-url-link:hover {
    color: var(--accent);
    text-decoration: underline;
  }

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
