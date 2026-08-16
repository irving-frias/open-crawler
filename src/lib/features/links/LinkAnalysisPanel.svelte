<script lang="ts">
  import { Link2, RefreshCw, ChevronDown } from '@lucide/svelte';
  import { m } from '$lib/paraglide/messages.js';
  import { getLinkAnalysis, getProjectHasLinks } from '$lib/api/links';
  import {
    getOrphanPagesPage,
    getDeadEndPagesPage,
    getTopAnchorsPage,
    getExternalDomainsPage,
  } from '$lib/api/links';
  import { getSeoOverview } from '$lib/api/seo';
  import type {
    LinkAnalysis,
    SeoCheckResult,
    SeoOverview,
    AnchorAgg,
    DomainAgg,
  } from '$lib/api/types';
  import { localizeSeoCheck } from '$lib/seo-checks';
  import { seoScoreColor } from '$lib/seo-ui';
  import { cn } from '$lib/utils.js';
  import { Badge } from '$lib/components/ui/badge/index.js';
  import { Button } from '$lib/components/ui/button/index.js';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card/index.js';
  import { Skeleton } from '$lib/components/ui/skeleton/index.js';
  import { Progress } from '$lib/components/ui/progress/index.js';

  let {
    projectId,
  }: {
    projectId: string;
  } = $props();

  let links = $state<LinkAnalysis | null>(null);
  let overview = $state<SeoOverview | null>(null);
  let loading = $state(false);
  let error = $state('');
  let linksSeq = 0;

  // Paginated list state per section. `total` mirrors the aggregate counts from
  // LinkAnalysis (orphans/dead ends) or is discovered after the first
  // show-more (anchors/domains); the first page comes from LinkAnalysis itself.
  type MoreState<T> = { items: T[]; total: number | null; loading: boolean };
  let orphanState = $state<MoreState<string>>({ items: [], total: null, loading: false });
  let deadEndState = $state<MoreState<string>>({ items: [], total: null, loading: false });
  let anchorState = $state<MoreState<AnchorAgg>>({ items: [], total: null, loading: false });
  let domainState = $state<MoreState<DomainAgg>>({ items: [], total: null, loading: false });

  async function loadLinks() {
    if (!projectId) return;
    const seq = ++linksSeq;
    loading = true;
    error = '';
    try {
      const has = await getProjectHasLinks(projectId);
      if (seq !== linksSeq) return;
      if (has) {
        const [data, ov] = await Promise.all([
          getLinkAnalysis(projectId),
          getSeoOverview(projectId),
        ]);
        if (seq !== linksSeq) return;
        links = data;
        overview = ov;
        orphanState = { items: data.orphan_pages, total: data.orphan_count, loading: false };
        deadEndState = { items: data.dead_end_pages, total: data.dead_end_count, loading: false };
        anchorState = { items: data.top_anchors, total: null, loading: false };
        domainState = { items: data.external_domains, total: null, loading: false };
      } else {
        links = null;
        overview = null;
        orphanState = { items: [], total: null, loading: false };
        deadEndState = { items: [], total: null, loading: false };
        anchorState = { items: [], total: null, loading: false };
        domainState = { items: [], total: null, loading: false };
      }
    } catch (e) {
      if (seq === linksSeq) error = String(e);
    } finally {
      if (seq === linksSeq) loading = false;
    }
  }

  $effect(() => {
    if (projectId) loadLinks();
    else {
      links = null;
      overview = null;
    }
  });

  function hasMore(s: MoreState<unknown>, initialFull: boolean): boolean {
    if (s.items.length === 0 || s.loading) return false;
    if (s.total !== null) return s.items.length < s.total;
    return initialFull;
  }

  async function showMore(kind: 'orphan' | 'deadEnd' | 'anchor' | 'domain', page: number) {
    if (!projectId) return;
    const seq = linksSeq;
    const s =
      kind === 'orphan'
        ? orphanState
        : kind === 'deadEnd'
          ? deadEndState
          : kind === 'anchor'
            ? anchorState
            : domainState;
    if (s.loading) return;
    const patch = (next: MoreState<unknown>) => {
      if (seq !== linksSeq) return;
      if (kind === 'orphan') orphanState = next as MoreState<string>;
      else if (kind === 'deadEnd') deadEndState = next as MoreState<string>;
      else if (kind === 'anchor') anchorState = next as MoreState<AnchorAgg>;
      else domainState = next as MoreState<DomainAgg>;
    };
    patch({ ...s, loading: true });
    try {
      if (kind === 'orphan') {
        const [items, total] = await getOrphanPagesPage(projectId, page, 10);
        if (seq === linksSeq)
          orphanState = { items: [...orphanState.items, ...items], total, loading: false };
      } else if (kind === 'deadEnd') {
        const [items, total] = await getDeadEndPagesPage(projectId, page, 10);
        if (seq === linksSeq)
          deadEndState = { items: [...deadEndState.items, ...items], total, loading: false };
      } else if (kind === 'anchor') {
        const [items, total] = await getTopAnchorsPage(projectId, page, 20);
        if (seq === linksSeq)
          anchorState = { items: [...anchorState.items, ...items], total, loading: false };
      } else {
        const [items, total] = await getExternalDomainsPage(projectId, page, 50);
        if (seq === linksSeq)
          domainState = { items: [...domainState.items, ...items], total, loading: false };
      }
    } catch (e) {
      if (seq === linksSeq) {
        error = String(e);
        patch({ ...s, loading: false });
      }
    }
  }

  const linkChecks = $derived<SeoCheckResult[]>(overview?.link_checks ?? []);
  const failingChecks = $derived(linkChecks.filter((c) => !c.passed));

  let expandedChecks = $state<Set<string>>(new Set());

  function toggleCheck(id: string) {
    const next = new Set(expandedChecks);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    expandedChecks = next;
  }

  function severityVariant(severity: string): 'default' | 'warning' | 'destructive' {
    if (severity === 'error') return 'destructive';
    if (severity === 'warning') return 'warning';
    return 'default';
  }

  function pct(part: number, total: number): number {
    return total > 0 ? Math.round((part / total) * 100) : 0;
  }
</script>

{#if loading && !links}
  <div class="links-loading">
    <Skeleton class="h-24 w-full" />
    <Skeleton class="h-24 w-full" />
    <Skeleton class="h-24 w-full" />
  </div>
{:else if error && !links}
  <div class="links-error">{error}</div>
{:else if !links}
  <Card>
    <CardContent class="flex items-center justify-between gap-2">
      <p class="links-empty">{m['links.empty']()}</p>
      <Button variant="outline" size="sm" class="gap-1.5" onclick={loadLinks} disabled={loading}>
        <RefreshCw class={cn('size-3.5', loading && 'animate-spin')} />
        {m['links.refresh']()}
      </Button>
    </CardContent>
  </Card>
{:else}
  <div class="links-panel">
    <Card>
      <CardHeader class="flex-row items-center justify-between gap-2">
        <div class="flex items-center gap-2">
          <Link2 class="size-4" />
          <CardTitle>{m['links.title']()}</CardTitle>
        </div>
        <Button variant="outline" size="sm" class="gap-1.5" onclick={loadLinks} disabled={loading}>
          <RefreshCw class={cn('size-3.5', loading && 'animate-spin')} />
          {m['links.refresh']()}
        </Button>
      </CardHeader>
      <CardContent>
        <p class="links-subtitle">{m['links.subtitle']()}</p>

        {#if overview?.link_score != null}
          <div class="links-score-row">
            <span class="links-score-label">{m['links.score']()}</span>
            <span class="links-score-value" style="color: {seoScoreColor(overview.link_score)}">
              {Math.round(overview.link_score)}
            </span>
            {#if overview.link_grade}
              <span class="links-score-grade">· {overview.link_grade}</span>
            {/if}
          </div>
        {/if}

        <div class="links-stats">
          <div class="links-stat">
            <span class="links-stat-label">{m['links.total']()}</span>
            <span class="links-stat-value">{links.total_links.toLocaleString()}</span>
          </div>
          <div class="links-stat">
            <span class="links-stat-label">{m['links.internal']()}</span>
            <span class="links-stat-value">{links.internal_links.toLocaleString()}</span>
          </div>
          <div class="links-stat">
            <span class="links-stat-label">{m['links.external']()}</span>
            <span class="links-stat-value">{links.external_links.toLocaleString()}</span>
          </div>
          <div class="links-stat">
            <span class="links-stat-label">{m['links.self']()}</span>
            <span class="links-stat-value">{links.self_links.toLocaleString()}</span>
          </div>
          <div class="links-stat">
            <span class="links-stat-label">{m['links.followed']()}</span>
            <span class="links-stat-value">{links.followed_links.toLocaleString()}</span>
          </div>
          <div class="links-stat">
            <span class="links-stat-label">{m['links.nofollow']()}</span>
            <span class="links-stat-value">{links.nofollow_links.toLocaleString()}</span>
          </div>
          <div class="links-stat">
            <span class="links-stat-label">{m['links.sponsored']()}</span>
            <span class="links-stat-value">{links.sponsored_links.toLocaleString()}</span>
          </div>
          <div class="links-stat">
            <span class="links-stat-label">{m['links.ugc']()}</span>
            <span class="links-stat-value">{links.ugc_links.toLocaleString()}</span>
          </div>
          <div class="links-stat">
            <span class="links-stat-label">{m['links.unique_internal']()}</span>
            <span class="links-stat-value">{links.unique_internal_targets.toLocaleString()}</span>
          </div>
          <div class="links-stat">
            <span class="links-stat-label">{m['links.internal_pages']()}</span>
            <span class="links-stat-value">{links.internal_pages.toLocaleString()}</span>
          </div>
        </div>

        <div class="links-grid">
          <div class="links-section">
            <h4 class="links-section-title">
              {m['links.orphans']()}
              <span class="links-section-count">
                {m['links.sample']({
                  shown: orphanState.items.length,
                  total: links.orphan_count,
                })}
              </span>
            </h4>
            <div class="links-section-body">
              <Badge variant={links.orphan_count === 0 ? 'default' : 'warning'}>
                {links.orphan_count.toLocaleString()}
              </Badge>
              {#if orphanState.items.length > 0}
                <ul class="links-list">
                  {#each orphanState.items as url (url)}
                    <li class="links-list-item" title={url}>{url}</li>
                  {/each}
                </ul>
              {/if}
              {#if hasMore(orphanState, true)}
                <button
                  type="button"
                  class="links-more"
                  onclick={() => showMore('orphan', Math.ceil(orphanState.items.length / 10) + 1)}
                  disabled={orphanState.loading}
                >
                  <ChevronDown class={cn('size-3.5', orphanState.loading && 'animate-pulse')} />
                  <span>{m['comparator.show_more']()}</span>
                </button>
              {/if}
            </div>
          </div>

          <div class="links-section">
            <h4 class="links-section-title">
              {m['links.dead_ends']()}
              <span class="links-section-count">
                {m['links.sample']({
                  shown: deadEndState.items.length,
                  total: links.dead_end_count,
                })}
              </span>
            </h4>
            <div class="links-section-body">
              <Badge variant={links.dead_end_count === 0 ? 'default' : 'warning'}>
                {links.dead_end_count.toLocaleString()}
              </Badge>
              {#if deadEndState.items.length > 0}
                <ul class="links-list">
                  {#each deadEndState.items as url (url)}
                    <li class="links-list-item" title={url}>{url}</li>
                  {/each}
                </ul>
              {/if}
              {#if hasMore(deadEndState, true)}
                <button
                  type="button"
                  class="links-more"
                  onclick={() => showMore('deadEnd', Math.ceil(deadEndState.items.length / 10) + 1)}
                  disabled={deadEndState.loading}
                >
                  <ChevronDown class={cn('size-3.5', deadEndState.loading && 'animate-pulse')} />
                  <span>{m['comparator.show_more']()}</span>
                </button>
              {/if}
            </div>
          </div>
        </div>

        <div class="links-grid">
          <div class="links-section">
            <h4 class="links-section-title">{m['links.anchor_quality']()}</h4>
            <div class="links-section-body">
              {#if links.anchor_quality.descriptive + links.anchor_quality.generic > 0}
                <div class="links-quality-row">
                  <span class="links-quality-name">{m['links.anchor.descriptive']()}</span>
                  <Progress
                    value={pct(
                      links.anchor_quality.descriptive,
                      links.anchor_quality.descriptive + links.anchor_quality.generic
                    )}
                    max={100}
                    class="h-1.5 flex-1"
                  />
                  <span class="links-quality-value">{links.anchor_quality.descriptive}</span>
                </div>
                <div class="links-quality-row">
                  <span class="links-quality-name">{m['links.anchor.generic']()}</span>
                  <Progress
                    value={pct(
                      links.anchor_quality.generic,
                      links.anchor_quality.descriptive + links.anchor_quality.generic
                    )}
                    max={100}
                    class="h-1.5 flex-1"
                  />
                  <span class="links-quality-value">{links.anchor_quality.generic}</span>
                </div>
              {/if}
              <div class="links-quality-row">
                <span class="links-quality-name">{m['links.anchor.url']()}</span>
                <span class="links-quality-value">{links.anchor_quality.url_anchors}</span>
              </div>
              <div class="links-quality-row">
                <span class="links-quality-name">{m['links.anchor.empty']()}</span>
                <span class="links-quality-value">{links.anchor_quality.empty}</span>
              </div>
            </div>
          </div>

          {#if anchorState.items.length > 0}
            <div class="links-section">
              <h4 class="links-section-title">
                {m['links.top_anchors']()}
                {#if anchorState.total != null}
                  <span class="links-section-count">
                    {m['links.sample']({
                      shown: anchorState.items.length,
                      total: anchorState.total,
                    })}
                  </span>
                {/if}
              </h4>
              <div class="links-section-body">
                <ul class="links-list">
                  {#each anchorState.items as a (a.anchor)}
                    <li class="links-list-row">
                      <span class="links-anchor-text" title={a.anchor}>{a.anchor}</span>
                      <Badge variant="secondary">{a.count}</Badge>
                    </li>
                  {/each}
                </ul>
                {#if hasMore(anchorState, links.top_anchors.length >= 20)}
                  <button
                    type="button"
                    class="links-more"
                    onclick={() => showMore('anchor', Math.ceil(anchorState.items.length / 20) + 1)}
                    disabled={anchorState.loading}
                  >
                    <ChevronDown class={cn('size-3.5', anchorState.loading && 'animate-pulse')} />
                    <span>{m['comparator.show_more']()}</span>
                  </button>
                {/if}
              </div>
            </div>
          {/if}
        </div>

        {#if domainState.items.length > 0}
          <div class="links-section">
            <h4 class="links-section-title">
              {m['links.external_domains']()}
              <span class="links-section-count">
                {m['links.sample']({
                  shown: domainState.items.length,
                  total: domainState.total ?? domainState.items.length,
                })}
              </span>
            </h4>
            <div class="links-section-body">
              <ul class="links-list">
                {#each domainState.items as d (d.domain)}
                  <li class="links-list-row">
                    <span class="links-domain" title={d.domain}>{d.domain}</span>
                    <span class="links-domain-meta">
                      {#if d.nofollow > 0}<span>NF {d.nofollow}</span>{/if}
                      {#if d.sponsored > 0}<span>SP {d.sponsored}</span>{/if}
                      {#if d.ugc > 0}<span>UGC {d.ugc}</span>{/if}
                    </span>
                    <Badge variant="secondary">{d.count}</Badge>
                  </li>
                {/each}
              </ul>
              {#if hasMore(domainState, links.external_domains.length >= 50)}
                <button
                  type="button"
                  class="links-more"
                  onclick={() => showMore('domain', Math.ceil(domainState.items.length / 50) + 1)}
                  disabled={domainState.loading}
                >
                  <ChevronDown class={cn('size-3.5', domainState.loading && 'animate-pulse')} />
                  <span>{m['comparator.show_more']()}</span>
                </button>
              {/if}
            </div>
          </div>
        {/if}

        {#if linkChecks.length > 0}
          <div class="links-section">
            <h4 class="links-section-title">
              {m['links.checks']()}
              <span class="links-section-count">
                {failingChecks.length > 0 ? failingChecks.length : '0'}
              </span>
            </h4>
            <div class="links-section-body">
              {#if failingChecks.length === 0}
                <p class="links-no-issues">{m['links.no_issues']()}</p>
              {:else}
                {#each failingChecks as check (check.id)}
                  {@const localized = localizeSeoCheck(
                    check.id,
                    check.message,
                    check.guidance,
                    check.evidence
                  )}
                  {@const open = expandedChecks.has(check.id)}
                  <div class="links-check">
                    <Badge variant={severityVariant(check.severity)}>{check.severity}</Badge>
                    <div class="links-check-body">
                      <span class="links-check-message">{localized.message}</span>
                      <span class="links-check-guidance">{localized.guidance}</span>
                      <button
                        type="button"
                        class="links-check-toggle"
                        onclick={() => toggleCheck(check.id)}
                        aria-expanded={open}
                      >
                        <ChevronDown
                          class={cn(
                            'size-3.5 transition-transform duration-200',
                            open && 'rotate-180'
                          )}
                        />
                        <span>{open ? m['links.collapse']() : m['links.expand']()}</span>
                      </button>
                      {#if open && (localized.fix || localized.expected)}
                        <div class="links-fix-block">
                          {#if localized.fix}
                            <div class="links-fix-line">
                              <span class="links-fix-label">{m['seo.fix']()}</span>
                              <span class="links-fix-text">{localized.fix}</span>
                            </div>
                          {/if}
                          {#if localized.expected}
                            <div class="links-fix-line">
                              <span class="links-fix-label">{m['seo.expected']()}</span>
                              <pre class="links-expected-code"><code>{localized.expected}</code
                                ></pre>
                            </div>
                          {/if}
                        </div>
                      {/if}
                    </div>
                  </div>
                {/each}
              {/if}
            </div>
          </div>
        {/if}
      </CardContent>
    </Card>
  </div>
{/if}

<style>
  .links-loading {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .links-error {
    color: var(--danger);
    font-size: 0.85rem;
  }

  .links-panel {
    display: flex;
    flex-direction: column;
  }

  .links-subtitle {
    font-size: 0.78rem;
    color: var(--text-muted);
    margin: 0 0 12px;
  }

  .links-empty {
    font-size: 0.85rem;
    color: var(--text-muted);
    margin: 0;
  }

  .links-score-row {
    display: flex;
    align-items: baseline;
    gap: 8px;
    margin-bottom: 14px;
  }

  .links-score-label {
    font-size: 0.78rem;
    color: var(--text-muted);
    text-transform: uppercase;
  }

  .links-score-value {
    font-size: 1.6rem;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
    line-height: 1;
  }

  .links-score-grade {
    font-size: 1rem;
    font-weight: 700;
    color: var(--text-secondary);
  }

  .links-stats {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));
    gap: 12px;
    margin-bottom: 18px;
  }

  .links-stat {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 12px 14px;
    background: var(--bg-deep);
    border-radius: 12px;
    box-shadow: var(--neu-pressed-sm);
  }

  .links-stat-label {
    font-size: 0.7rem;
    color: var(--text-muted);
    text-transform: uppercase;
  }

  .links-stat-value {
    font-size: 1.15rem;
    font-weight: 700;
    color: var(--text);
    font-variant-numeric: tabular-nums;
  }

  .links-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
    gap: 14px;
    margin-bottom: 16px;
  }

  .links-section {
    margin-bottom: 16px;
  }

  .links-section-title {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 0.82rem;
    font-weight: 600;
    color: var(--text);
    margin: 0 0 8px;
  }

  .links-section-count {
    font-size: 0.72rem;
    font-weight: 500;
    color: var(--text-muted);
  }

  .links-section-body {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .links-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .links-list-item {
    font-size: 0.78rem;
    color: var(--text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-family: var(--font-mono);
  }

  .links-list-row {
    display: flex;
    align-items: center;
    gap: 8px;
    justify-content: space-between;
  }

  .links-anchor-text {
    font-size: 0.8rem;
    color: var(--text);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .links-domain {
    font-size: 0.8rem;
    color: var(--text);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .links-domain-meta {
    display: flex;
    gap: 6px;
    font-size: 0.7rem;
    color: var(--text-muted);
    font-variant-numeric: tabular-nums;
  }

  .links-quality-row {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .links-quality-name {
    font-size: 0.8rem;
    color: var(--text-secondary);
    min-width: 90px;
  }

  .links-quality-value {
    font-size: 0.8rem;
    color: var(--text);
    font-variant-numeric: tabular-nums;
    min-width: 28px;
    text-align: right;
  }

  .links-no-issues {
    font-size: 0.85rem;
    color: var(--text-muted);
    margin: 0;
  }

  .links-more {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    margin-top: 4px;
    padding: 4px 8px;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--bg-deep);
    color: var(--text-muted);
    font-size: 0.75rem;
    cursor: pointer;
    align-self: flex-start;
  }

  .links-more:hover:not(:disabled) {
    color: var(--text);
    border-color: var(--border-strong);
  }

  .links-more:disabled {
    opacity: 0.6;
    cursor: default;
  }

  .links-check {
    display: flex;
    gap: 8px;
    padding: 10px 12px;
    background: var(--bg-deep);
    border-radius: 10px;
  }

  .links-check-body {
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-width: 0;
  }

  .links-check-message {
    font-size: 0.85rem;
    color: var(--text);
  }

  .links-check-guidance {
    font-size: 0.78rem;
    color: var(--text-secondary);
  }

  .links-check-toggle {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    margin-top: 4px;
    padding: 0;
    border: none;
    background: none;
    color: var(--text-muted);
    font-size: 0.75rem;
    cursor: pointer;
  }

  .links-check-toggle:hover {
    color: var(--text);
  }

  .links-fix-block {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin-top: 8px;
    padding-top: 8px;
    border-top: 1px solid var(--border);
  }

  .links-fix-line {
    display: flex;
    align-items: baseline;
    gap: 8px;
  }

  .links-fix-label {
    font-size: 0.72rem;
    font-weight: 600;
    color: var(--text-muted);
    text-transform: uppercase;
  }

  .links-fix-text {
    font-size: 0.8rem;
    color: var(--text-secondary);
  }

  .links-expected-code {
    margin: 0;
    padding: 8px 10px;
    background: var(--bg);
    border-radius: 6px;
    font-size: 0.75rem;
    overflow-x: auto;
  }
</style>
