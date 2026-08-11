<script lang="ts">
  import { m } from '$lib/paraglide/messages.js';
  import { translateIssueMessage, parseIssueParams } from '$lib/i18n-issues';
  import {
    Search,
    X,
    ChevronDown,
    CircleX,
    TriangleAlert,
    Info,
    SearchX,
    Database,
    ExternalLink,
    BookOpenText,
    Gauge,
  } from '@lucide/svelte';
  import { Input } from '$lib/components/ui/input/index.js';
  import { Button } from '$lib/components/ui/button/index.js';
  import { Badge } from '$lib/components/ui/badge/index.js';
  import { parseIssues, getIssueCounts } from '$lib/features/results/issue-cache';
  import { seoVariant as seoUiVariant, seoGrade as seoUiGrade } from '$lib/seo-ui';

  let {
    items,
    expandedUrl = $bindable(''),
    onDetail,
    searchQuery = '',
    onSearch,
  }: {
    items: any[];
    expandedUrl: string;
    onDetail?: (pageId: string) => void;
    searchQuery?: string;
    onSearch?: (query: string) => void;
  } = $props();

  // eslint-disable-next-line svelte/prefer-writable-derived -- localSearch is user-editable, searchQuery prop is not bindable
  let localSearch = $state('');
  // Highlighting runs on every row; debounce it so rapid keystrokes don't
  // re-render every title with <mark> tags.
  let highlightSearch = $state('');
  let highlightTimer: ReturnType<typeof setTimeout> | null = null;
  let scrollContainer = $state<HTMLElement | null>(null);

  $effect(() => {
    localSearch = searchQuery;
  });

  $effect(() => {
    const q = localSearch;
    if (highlightTimer) clearTimeout(highlightTimer);
    highlightTimer = setTimeout(() => {
      highlightSearch = q;
    }, 250);
  });

  function handleInput(e: Event) {
    const val = (e.target as HTMLInputElement).value;
    localSearch = val;
    onSearch?.(val);
  }

  const highlightRegex = $derived(
    highlightSearch
      ? new RegExp(`(${highlightSearch.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')})`, 'gi')
      : null
  );

  function escapeHtml(s: string): string {
    return s
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;')
      .replace(/'/g, '&#39;');
  }

  function applyHighlight(text: string | null): string {
    if (!text) return '';
    // Escape first (the title comes from a crawled, attacker-controlled page),
    // then wrap matches in <mark> so the injected markup is never raw HTML.
    const escaped = escapeHtml(text);
    if (!highlightRegex) return escaped;
    return escaped.replace(highlightRegex, '<mark>$1</mark>');
  }

  function toggleIssues(url: string) {
    expandedUrl = expandedUrl === url ? '' : url;
  }

  function resultCountLabel(count: number): string {
    return count === 1
      ? m['results.search_match_one']({ count: count.toString() })
      : m['results.search_matches']({ count: count.toString() });
  }

  function readabilityLabel(score: number): string {
    if (score >= 70) return m['dashboard.readability.easy']();
    if (score >= 40) return m['dashboard.readability.medium']();
    return m['dashboard.readability.hard']();
  }

  function readabilityVariant(score: number): 'default' | 'warning' | 'destructive' {
    if (score >= 70) return 'default';
    if (score >= 40) return 'warning';
    return 'destructive';
  }

  function seoVariant(score: number): 'default' | 'warning' | 'destructive' {
    return seoUiVariant(score);
  }

  function seoGrade(score: number): string {
    return seoUiGrade(score);
  }

  interface RowModel {
    page: any;
    issues: any[];
    issueCounts: { errors: number; warnings: number; infos: number };
    readabilityVariant: 'default' | 'warning' | 'destructive' | undefined;
    readabilityLabel: string | null;
    seoVariant: 'default' | 'warning' | 'destructive' | undefined;
    seoGrade: string | null;
    titleHtml: string;
  }

  const rows = $derived<RowModel[]>(
    items.map((page) => {
      // All severities (not just errors): warning/info badges and the expandable
      // detail row previously never rendered because issues were error-filtered.
      const issues = parseIssues(page.semantic_issues_json);
      const issueCounts = getIssueCounts(issues);
      const hasScore = page.readability_score != null;
      const hasSeo = page.seo_score != null;
      return {
        page,
        issues,
        issueCounts,
        readabilityVariant: hasScore ? readabilityVariant(page.readability_score) : undefined,
        readabilityLabel: hasScore ? readabilityLabel(page.readability_score) : null,
        seoVariant: hasSeo ? seoVariant(page.seo_score) : undefined,
        seoGrade: hasSeo ? seoGrade(page.seo_score) : null,
        titleHtml: applyHighlight(page.title || m['detail.no_title']()),
      };
    })
  );
</script>

<div class="table-wrapper">
  <div class="search-bar flex items-center gap-2 px-4 py-2">
    <div class="relative flex-1">
      <Search class="absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
      <Input
        type="text"
        placeholder={m['results.search_placeholder']()}
        value={localSearch}
        oninput={handleInput}
        class="pl-9 pr-9"
      />
      {#if localSearch}
        <Button
          variant="ghost"
          size="icon-xs"
          class="absolute right-1.5 top-1/2 -translate-y-1/2"
          onclick={() => {
            localSearch = '';
            onSearch?.('');
          }}
          aria-label={m['results.clear_search']()}
          title={m['results.clear_search']()}
        >
          <X class="size-4" />
        </Button>
      {/if}
    </div>
    {#if localSearch}
      <span class="result-count">
        {resultCountLabel(items.length)}
      </span>
    {/if}
  </div>

  <div class="header-row">
    <div class="col-url">{m['results.col.url']()}</div>
    <div class="col-status">{m['results.col.status']()}</div>
    <div class="col-issues">{m['results.col.issues']()}</div>
  </div>

  {#if items.length === 0}
    <div class="empty-state">
      {#if localSearch}
        <SearchX class="empty-icon" />
        <span class="empty-title">{m['results.no_search_matches']({ query: localSearch })}</span>
      {:else}
        <Database class="empty-icon" />
        <span class="empty-title">{m['results.no_results']()}</span>
      {/if}
    </div>
  {:else}
    <div class="rows-body" bind:this={scrollContainer}>
      {#each rows as row (row.page.id)}
        {@const isExpanded = expandedUrl === row.page.url}
        <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
        <div
          class="table-row main-row"
          class:has-issues={row.issues.length > 0}
          class:expanded={isExpanded}
          role={row.issues.length > 0 ? 'button' : undefined}
          tabindex={row.issues.length > 0 ? 0 : undefined}
          aria-expanded={row.issues.length > 0 ? isExpanded : undefined}
          onclick={() => row.issues.length > 0 && toggleIssues(row.page.url)}
          onkeydown={(e) => {
            if (row.issues.length > 0 && (e.key === 'Enter' || e.key === ' ')) {
              e.preventDefault();
              toggleIssues(row.page.url);
            }
          }}
        >
          <div class="col-url">
            <a
              class="url-link"
              href={row.page.url}
              target="_blank"
              rel="noreferrer"
              title={row.page.url}
              onclick={(e) => e.stopPropagation()}
            >
              <span class="url-text">{row.page.url}</span>
              <ExternalLink class="url-external size-3.5" />
            </a>
            <button
              class="btn-title"
              onclick={() => onDetail?.(row.page.id)}
              title={m['results.view_details']()}
            >
              <!-- eslint-disable-next-line svelte/no-at-html-tags -->
              {@html row.titleHtml}
            </button>
          </div>
          <div class="col-status status-{Math.floor(row.page.status_code / 100)}xx">
            {#if row.page.blocked}
              <span class="status-blocked">{m['results.status.blocked']()}</span>
            {:else}
              {row.page.status_code}
            {/if}
          </div>
          <div class="col-issues">
            {#if row.page.seo_score != null}
              <Badge
                variant={row.seoVariant}
                class="seo-badge gap-1"
                title={`${m['seo.label']()}: ${m['seo.grade']({ grade: row.seoGrade ?? '' })} (${Math.round(row.page.seo_score)})`}
              >
                <Gauge class="size-3.5" />
                {Math.round(row.page.seo_score)} · {row.seoGrade}
              </Badge>
            {/if}
            {#if row.page.readability_score != null}
              <Badge
                variant={row.readabilityVariant}
                class="readability-badge gap-1"
                title={`${m['dashboard.readability.label']()}: ${row.readabilityLabel} (${Math.round(row.page.readability_score)})`}
              >
                <BookOpenText class="size-3.5" />
                {Math.round(row.page.readability_score)}
              </Badge>
            {/if}
            {#if row.issues.length > 0}
              <div class="issue-badges">
                {#if row.issueCounts.errors > 0}
                  <Badge variant="outline" class="issue-badge-error gap-1">
                    <CircleX class="size-3.5" />
                    {row.issueCounts.errors}
                  </Badge>
                {/if}
                {#if row.issueCounts.warnings > 0}
                  <Badge variant="outline" class="issue-badge-warning gap-1">
                    <TriangleAlert class="size-3.5" />
                    {row.issueCounts.warnings}
                  </Badge>
                {/if}
                {#if row.issueCounts.infos > 0}
                  <Badge variant="outline" class="issue-badge-info gap-1">
                    <Info class="size-3.5" />
                    {row.issueCounts.infos}
                  </Badge>
                {/if}
              </div>
            {:else}
              <span class="no-issues">{m['results.ok']()}</span>
            {/if}
            {#if row.issues.length > 0}
              <span class="row-chevron" class:rotated={isExpanded}>
                <ChevronDown class="size-4" />
              </span>
            {/if}
          </div>
        </div>
        {#if isExpanded}
          <div class="table-row detail-row">
            <div class="issue-detail">
              {#each row.issues as issue, i (i)}
                {@const params = parseIssueParams(issue.message, issue.issue_type)}
                <div class="issue-item issue-{issue.severity}">
                  <span class="issue-icon">
                    {#if issue.severity === 'error'}
                      <CircleX class="size-4" />
                    {:else if issue.severity === 'warning'}
                      <TriangleAlert class="size-4" />
                    {:else}
                      <Info class="size-4" />
                    {/if}
                  </span>
                  <span class="issue-element">{issue.element}</span>
                  <span class="issue-message"
                    >{translateIssueMessage(issue.issue_type, params)}</span
                  >
                  {#if issue.xpath && !issue.issue_type.startsWith('missing_')}
                    <code class="issue-selector">{issue.xpath}</code>
                  {:else if issue.xpath}
                    <span class="issue-selector">{issue.xpath}</span>
                  {/if}
                </div>
              {/each}
            </div>
          </div>
        {/if}
      {/each}
    </div>
  {/if}
</div>

<style>
  .table-wrapper {
    overflow: auto;
    max-height: min(600px, 60vh);
    border-radius: var(--radius-lg);
    border: none;
    background: var(--bg-card);
    box-shadow: var(--neu-pressed-sm);
    scrollbar-width: thin;
    scrollbar-color: var(--border-muted) transparent;
  }

  .table-wrapper::-webkit-scrollbar {
    width: 8px;
    height: 8px;
  }
  .table-wrapper::-webkit-scrollbar-track {
    background: transparent;
  }
  .table-wrapper::-webkit-scrollbar-thumb {
    background: var(--border-muted);
    border-radius: 4px;
  }
  .table-wrapper::-webkit-scrollbar-thumb:hover {
    background: var(--text-muted);
  }

  .result-count {
    flex-shrink: 0;
    border-radius: var(--radius-md);
    border: none;
    background: var(--bg-card);
    box-shadow: var(--neu-raised-sm);
    padding: 2px 10px;
    font-size: 0.78rem;
    color: var(--text-secondary);
    white-space: nowrap;
  }

  .header-row {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 90px 176px;
    position: sticky;
    top: 0;
    z-index: 10;
    background: var(--bg-card);
    background: color-mix(in srgb, var(--bg-card) 92%, transparent);
    backdrop-filter: blur(8px);
    -webkit-backdrop-filter: blur(8px);
    border-bottom: 1px solid var(--border);
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
  }

  .header-row > div {
    padding: 14px 16px;
    font-weight: 600;
    color: var(--text-primary);
    font-size: 0.75rem;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    border-right: 1px solid var(--border);
    white-space: nowrap;
    user-select: none;
    display: flex;
    align-items: center;
    min-height: 48px;
  }

  .header-row > div:last-child {
    border-right: none;
  }

  .header-row > div:first-child {
    border-top-left-radius: var(--radius-lg);
  }

  .header-row > div:last-child {
    border-top-right-radius: var(--radius-lg);
  }

  .table-row {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 90px 176px;
    grid-template-rows: auto;
    min-height: 52px;
    padding: 0 16px;
    align-items: center;
    border-bottom: 1px solid var(--border);
    font-size: 0.9rem;
    box-sizing: border-box;
    transition: background var(--transition-base);
    content-visibility: auto;
    contain-intrinsic-size: auto 52px;
  }

  .detail-row {
    contain-intrinsic-size: auto 96px;
  }

  .main-row.has-issues {
    cursor: pointer;
  }

  .main-row:hover {
    background: var(--bg-hover);
  }

  .main-row.has-issues:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
  }

  .detail-row {
    display: block;
    background: var(--bg-deep);
    border-bottom: 1px solid var(--border);
    padding: 0;
    width: 100%;
    animation: detail-in var(--transition-slow);
  }

  @keyframes detail-in {
    from {
      opacity: 0;
      transform: translateY(-4px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .col-url {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
    align-items: flex-start;
    justify-content: center;
    padding: 6px 0;
    overflow: hidden;
  }

  .url-link {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    max-width: 100%;
    font-size: 0.85rem;
    color: var(--accent);
    text-decoration: none;
    overflow: hidden;
  }

  .url-text {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .url-external {
    flex-shrink: 0;
    opacity: 0.45;
    transition: opacity var(--transition-fast);
  }

  .url-link:hover {
    text-decoration: underline;
    color: var(--accent-hover);
  }

  :global(.url-link:hover .url-external) {
    opacity: 1;
  }

  .btn-title {
    background: none;
    border: none;
    padding: 0;
    color: var(--accent);
    font-size: 0.9rem;
    text-align: left;
    cursor: pointer;
    font-weight: 500;
    line-height: 1.3;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 100%;
    text-decoration: underline;
    text-decoration-color: transparent;
    transition: text-decoration-color var(--transition-fast);
  }

  .btn-title:hover {
    text-decoration-color: var(--accent);
    color: var(--accent-hover);
  }

  .btn-title:focus-visible,
  .url-link:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
    border-radius: 2px;
  }

  .col-status {
    width: fit-content;
    font-size: 0.78rem;
    font-weight: 700;
    padding: 3px 10px;
    border-radius: var(--radius-pill);
    line-height: 1.2;
    font-variant-numeric: tabular-nums;
  }

  .status-2xx {
    background: var(--bg-status-2xx);
    color: var(--success);
  }
  .status-3xx {
    background: var(--bg-status-3xx);
    color: var(--warning);
  }
  .status-4xx {
    background: var(--bg-status-4xx);
    color: var(--orange);
  }
  .status-5xx {
    background: var(--bg-status-5xx);
    color: var(--danger);
  }
  .status-0xx {
    background: var(--bg-hover);
    color: var(--text-muted);
  }
  .status-blocked {
    background: var(--bg-status-4xx);
    color: var(--warning);
  }

  .col-issues {
    display: flex;
    align-items: center;
    gap: 6px;
    min-width: 0;
  }

  .row-chevron {
    margin-left: auto;
    color: var(--text-muted);
    display: inline-flex;
    flex-shrink: 0;
    transition: transform var(--transition-base);
  }

  .row-chevron.rotated {
    transform: rotate(180deg);
  }

  .issue-badges {
    display: flex;
    gap: 4px;
    flex-shrink: 0;
  }

  .issue-badge-error {
    background: var(--bg-issue-error);
    color: var(--danger);
    border-color: var(--danger);
  }
  .issue-badge-warning {
    background: var(--bg-issue-warning);
    color: var(--warning);
    border-color: var(--warning);
  }
  .issue-badge-info {
    background: var(--bg-issue-info);
    color: var(--info);
    border-color: var(--info);
  }

  .no-issues {
    color: var(--success);
    font-size: 0.85rem;
    font-weight: 500;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 10px;
    padding: 48px 20px;
    text-align: center;
    color: var(--text-muted);
    font-size: 0.9rem;
  }

  .empty-icon {
    width: 36px;
    height: 36px;
    color: var(--border-muted);
  }

  .empty-title {
    font-size: 0.9rem;
    max-width: 420px;
    overflow-wrap: break-word;
  }

  .issue-detail {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 12px 16px;
    width: 100%;
  }

  .issue-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 10px;
    border-radius: 6px;
    font-size: 0.85rem;
  }

  .issue-item.issue-error {
    background: var(--bg-issue-error);
  }
  .issue-item.issue-warning {
    background: var(--bg-issue-warning);
  }
  .issue-item.issue-info {
    background: var(--bg-issue-info);
  }

  .issue-icon {
    display: inline-flex;
    flex-shrink: 0;
    color: var(--text-muted);
  }
  .issue-item.issue-error .issue-icon {
    color: var(--danger);
  }
  .issue-item.issue-warning .issue-icon {
    color: var(--warning);
  }
  .issue-item.issue-info .issue-icon {
    color: var(--info);
  }

  .issue-element {
    font-weight: 600;
    color: var(--text);
    min-width: 80px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .issue-message {
    color: var(--text-secondary);
    flex: 1;
    min-width: 0;
    overflow-wrap: break-word;
  }

  .issue-selector {
    padding: 2px 6px;
    background: var(--bg-hover);
    border-radius: 4px;
    font-size: 0.75rem;
    color: var(--info);
    font-family: var(--font-mono);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 320px;
  }

  a {
    color: var(--accent);
    text-decoration: none;
  }
  a:hover {
    text-decoration: underline;
  }

  /* ==========================================
     RESPONSIVE — Mobile First
     ========================================== */

  /* Mobile base (<= 767px): card layout */
  .header-row {
    display: none;
  }

  .table-row {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 12px 16px;
    position: relative;
  }

  .col-url {
    width: 100%;
    padding-right: 76px;
  }

  .col-status {
    position: absolute;
    top: 12px;
    right: 16px;
  }

  .col-issues {
    width: 100%;
  }

  .issue-detail {
    padding: 8px 0;
  }

  .issue-item {
    flex-wrap: wrap;
  }

  .issue-element {
    min-width: 60px;
  }

  /* Tablet+ (768px+): 3-column table */
  @media (min-width: 768px) {
    .header-row {
      display: grid;
      grid-template-columns: minmax(0, 1fr) 90px 176px;
    }

    .table-row {
      display: grid;
      grid-template-columns: minmax(0, 1fr) 90px 176px;
      min-height: 52px;
      padding: 0 16px;
      align-items: center;
    }

    .col-url {
      width: auto;
      padding-right: 0;
    }

    .col-status {
      position: static;
    }

    .col-issues {
      width: auto;
    }
  }

  :global(.table-wrapper mark) {
    background: var(--bg-mark);
    color: inherit;
    padding: 0 1px;
    border-radius: 2px;
  }

  @media (prefers-reduced-motion: reduce) {
    .detail-row {
      animation: none;
    }

    .row-chevron,
    .url-external,
    .btn-title,
    .table-row {
      transition: none;
    }
  }
</style>
