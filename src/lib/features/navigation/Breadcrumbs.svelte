<script lang="ts">
  import { ArrowLeft, ChevronRight } from '@lucide/svelte';
  import { m } from '#lib/paraglide/messages.js';
  import { navigation } from '#lib/navigation.svelte.ts';
  import { translateIssueName } from '#lib/i18n-issues.ts';
  import type { TabValue } from '#lib/app.svelte.ts';

  let {
    projectName,
  }: {
    projectName: string;
  } = $props();

  const ctx = $derived(navigation.current);
  const canGoBack = $derived(navigation.historyLength > 0);

  function tabLabel(tab: TabValue): string {
    switch (tab) {
      case 'results':
        return m['nav.results']();
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
      case 'links':
        return m['tabs.links']();
      case 'extraction':
        return m['tabs.extraction']();
    }
  }

  /** Short human summary of the active result filters, if any. */
  const filterSummary = $derived.by(() => {
    const parts: string[] = [];
    const f = ctx.filters;
    if (f.statusCodes.length > 0) parts.push(f.statusCodes.join(', '));
    if (f.severities.length > 0)
      parts.push(
        f.severities
          .map((s) => {
            if (s === 'error') return m['severity.error']();
            if (s === 'warning') return m['severity.warning']();
            if (s === 'info') return m['severity.info']();
            if (s === 'ok') return m['severity.ok']();
            return s;
          })
          .join(', ')
      );
    if (f.issueType) parts.push(translateIssueName(f.issueType));
    if (f.is404) parts.push('404');
    if (ctx.search) parts.push(`“${ctx.search}”`);
    return parts.length > 0 ? parts.join(' · ') : null;
  });
</script>

{#if ctx.projectId}
  <nav class="breadcrumbs" aria-label={m['nav.breadcrumb_label']()}>
    <button
      type="button"
      class="crumb-back"
      disabled={!canGoBack}
      onclick={() => navigation.goBack()}
      title={m['nav.back']()}
    >
      <ArrowLeft class="size-3.5" />
      <span>{m['nav.back']()}</span>
    </button>
    <span class="crumb-sep" aria-hidden="true">/</span>
    <button type="button" class="crumb" onclick={() => navigation.goToTab('results')}>
      {projectName || m['nav.project']()}
    </button>
    <span class="crumb-sep" aria-hidden="true"><ChevronRight class="size-3" /></span>
    <span class="crumb-current">{tabLabel(ctx.tab)}</span>
    {#if filterSummary}
      <span class="crumb-sep" aria-hidden="true"><ChevronRight class="size-3" /></span>
      <button
        type="button"
        class="crumb crumb-filter"
        onclick={() => navigation.goToResults({})}
        title={m['nav.clear_filter']()}
      >
        {filterSummary}
      </button>
    {/if}
  </nav>
{/if}

<style>
  .breadcrumbs {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 6px;
    margin-bottom: 12px;
    font-size: 0.85rem;
    color: var(--text-secondary);
  }

  .crumb-back {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    background: none;
    border: none;
    padding: 2px 6px 2px 2px;
    cursor: pointer;
    font: inherit;
    color: var(--accent);
    border-radius: 6px;
  }

  .crumb-back:disabled {
    color: var(--text-muted);
    cursor: default;
    opacity: 0.5;
  }

  .crumb {
    background: none;
    border: none;
    padding: 2px 4px;
    cursor: pointer;
    font: inherit;
    color: var(--accent);
    border-radius: 6px;
  }

  .crumb:hover,
  .crumb-back:not(:disabled):hover {
    background: var(--bg-deep);
  }

  .crumb-current {
    font-weight: 600;
    color: var(--text);
    padding: 2px 4px;
  }

  .crumb-filter {
    max-width: 320px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .crumb-sep {
    display: inline-flex;
    align-items: center;
    color: var(--text-muted);
  }
</style>
