<script lang="ts">
  import { m } from '$lib/paraglide/messages.js';
  import { translateIssueMessage, parseIssueParams } from '$lib/i18n-issues';

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

  let localSearch = $state('');

  $effect(() => {
    localSearch = searchQuery;
  });

  function handleInput(e: Event) {
    const val = (e.target as HTMLInputElement).value;
    localSearch = val;
    onSearch?.(val);
  }

  function highlight(text: string | null, query: string): string {
    if (!query || !text) return text ?? '';
    const escaped = query.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
    return text.replace(new RegExp(`(${escaped})`, 'gi'), '<mark>$1</mark>');
  }

  function parseIssues(issuesJson: string | null): any[] {
    if (!issuesJson) return [];
    try { return JSON.parse(issuesJson); } catch { return []; }
  }

  function parseHreflang(hreflangJson: string | null): { lang: string; href: string }[] {
    if (!hreflangJson) return [];
    try { return JSON.parse(hreflangJson); } catch { return []; }
  }

  function getIssueCounts(issues: any[]): { errors: number; warnings: number; infos: number } {
    let errors = 0, warnings = 0, infos = 0;
    for (const issue of issues) {
      if (issue.severity === 'error') errors++;
      else if (issue.severity === 'warning') warnings++;
      else infos++;
    }
    return { errors, warnings, infos };
  }

  function getSeverityIcon(severity: string): string {
    if (severity === 'error') return '\u2716';
    if (severity === 'warning') return '\u26A0';
    return '\u2139';
  }

  function toggleIssues(url: string) {
    expandedUrl = expandedUrl === url ? '' : url;
  }
</script>

<div class="table-wrapper">
  <div class="search-bar">
    <div class="search-input-wrap">
      <svg class="search-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg>
      <input
        type="text"
        class="search-input"
        placeholder="Search URL, title, or H1..."
        value={localSearch}
        oninput={handleInput}
      />
      {#if localSearch}
        <button class="search-clear" onclick={() => { localSearch = ''; onSearch?.(''); }} aria-label="Clear search">&times;</button>
      {/if}
    </div>
    {#if localSearch}
      <span class="search-count">{items.length} result{items.length !== 1 ? 's' : ''}</span>
    {/if}
  </div>
  <table class="header-table">
    <thead>
      <tr>
        <th class="col-url">{m["results.col.url"]()}</th>
        <th class="col-status">{m["results.col.status"]()}</th>
        <th class="col-title">{m["results.col.title"]()}</th>
        <th class="col-desc">{m["results.col.description"]()}</th>
        <th class="col-h1">{m["results.col.h1"]()}</th>
        <th class="col-lang">{m["results.col.lang"]()}</th>
        <th class="col-hreflang">{m["results.col.hreflang"]()}</th>
        <th class="col-issues">{m["results.col.issues"]()}</th>
      </tr>
    </thead>
  </table>

  {#if items.length === 0 && localSearch}
    <div class="empty-state">No results match "{localSearch}"</div>
  {:else if items.length === 0}
    <div class="empty-state">{m["results.no_results"]()}</div>
  {:else}
    <div class="rows-body">
      {#each items as page (page.id)}
        {@const issues = parseIssues(page.semantic_issues_json)}
        {@const issueCounts = getIssueCounts(issues)}
        <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
        <div
          class="table-row main-row"
          class:has-issues={issues.length > 0}
          role={issues.length > 0 ? 'button' : undefined}
          tabindex={issues.length > 0 ? 0 : undefined}
          onclick={() => issues.length > 0 && toggleIssues(page.url)}
          onkeydown={(e) => {
            if (issues.length > 0 && (e.key === 'Enter' || e.key === ' ')) {
              e.preventDefault();
              toggleIssues(page.url);
            }
          }}
        >
          <div class="col-url">
            <a href={page.url} target="_blank" onclick={(e) => e.stopPropagation()}>
              {@html highlight(page.url, localSearch)}
            </a>
            {#if onDetail}
              <button
                class="btn-detail"
                title="View page details"
                onclick={(e) => { e.stopPropagation(); onDetail(page.id); }}
              >&#8599;</button>
            {/if}
          </div>
          <div class="col-status status-{Math.floor(page.status_code / 100)}xx">
            {page.status_code}
          </div>
          <div class="col-title">{@html highlight(page.title, localSearch) || '-'}</div>
          <div class="col-desc">{page.meta_description || '-'}</div>
          <div class="col-h1">{@html highlight(page.h1, localSearch) || '-'}</div>
          <div class="col-lang">{page.html_lang || '-'}</div>
          <div class="col-hreflang">
            {#each parseHreflang(page.hreflang_json) as hl}
              <span class="hreflang-badge">{hl.lang}</span>
            {/each}
            {#if !page.hreflang_json}
              -
            {/if}
          </div>
          <div class="col-issues">
            {#if issues.length > 0}
              <div class="issue-badges">
                {#if issueCounts.errors > 0}
                  <span class="issue-badge issue-error">{issueCounts.errors}</span>
                {/if}
                {#if issueCounts.warnings > 0}
                  <span class="issue-badge issue-warning">{issueCounts.warnings}</span>
                {/if}
                {#if issueCounts.infos > 0}
                  <span class="issue-badge issue-info">{issueCounts.infos}</span>
                {/if}
              </div>
            {:else}
              <span class="no-issues">{m["results.ok"]()}</span>
            {/if}
          </div>
        </div>

        {#if expandedUrl === page.url && issues.length > 0}
          <div class="table-row detail-row">
            <div class="issue-detail">
              {#each issues as issue}
                {@const params = parseIssueParams(issue.message, issue.issue_type)}
                <div class="issue-item issue-{issue.severity}">
          <span class="issue-icon">{getSeverityIcon(issue.severity)}</span>
          <span class="issue-element">{issue.element}</span>
          <span class="issue-message">{translateIssueMessage(issue.issue_type, params)}</span>
                  {#if issue.selector}
                    <code class="issue-selector">{issue.selector}</code>
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
    overflow-x: auto;
  }

  .header-table {
    width: 100%;
    border-collapse: collapse;
    table-layout: fixed;
  }

  .header-table th {
    padding: 12px 16px;
    text-align: left;
    border-bottom: 1px solid var(--border);
    font-weight: 600;
    color: var(--text-secondary);
    font-size: 0.85rem;
    text-transform: uppercase;
    background: var(--bg-card);
    position: sticky;
    top: 0;
    z-index: 1;
  }

  .rows-body {
    max-height: 600px;
    overflow-y: auto;
  }

  .table-row {
    display: grid;
    grid-template-columns: 30% 6% 14% 16% 10% 5% 8% 11%;
    grid-template-rows: auto;
    min-height: 48px;
    padding: 0 16px;
    align-items: center;
    border-bottom: 1px solid var(--border);
    font-size: 0.9rem;
    box-sizing: border-box;
  }

  .main-row.has-issues {
    cursor: pointer;
  }

  .main-row:hover {
    background: var(--bg-hover);
  }

  .detail-row {
    display: block;
    background: var(--bg-deep);
    border-bottom: 2px solid var(--border);
    padding: 0;
  }

  .col-url {
    width: 30%;
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .col-url a {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
  }

  .btn-detail {
    flex-shrink: 0;
    width: 22px;
    height: 22px;
    background: var(--border);
    border: none;
    border-radius: 4px;
    color: var(--text-secondary);
    font-size: 0.8rem;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    opacity: 0;
    transition: opacity 0.15s;
  }
  .main-row:hover .btn-detail { opacity: 1; }
  .btn-detail:hover { background: var(--accent); color: white; }
  .col-status { width: 6%; }
  .col-title { width: 14%; }
  .col-desc { width: 16%; }
  .col-h1 { width: 10%; }
  .col-lang { width: 5%; }
  .col-hreflang { width: 8%; }
  .col-issues { width: 11%; }

  .empty-state {
    padding: 40px;
    text-align: center;
    color: var(--text-muted);
    font-size: 0.9rem;
  }

  a {
    color: var(--accent);
    text-decoration: none;
  }
  a:hover { text-decoration: underline; }

  .status-2xx { color: var(--success); }
  .status-3xx { color: var(--warning); }
  .status-4xx { color: var(--orange); }
  .status-5xx { color: var(--danger); }

  .hreflang-badge {
    display: inline-block;
    padding: 1px 6px;
    background: var(--border);
    border-radius: 4px;
    font-size: 0.75rem;
    margin-right: 4px;
    color: var(--text-secondary);
  }

  .issue-badges { display: flex; gap: 4px; }

  .issue-badge {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 20px;
    height: 20px;
    padding: 0 5px;
    border-radius: 10px;
    font-size: 0.75rem;
    font-weight: 600;
  }
  .issue-error { background: var(--bg-issue-error); color: var(--danger); border: 1px solid var(--danger); }
  .issue-warning { background: var(--bg-issue-warning); color: var(--warning); border: 1px solid var(--warning); }
  .issue-info { background: var(--bg-issue-info); color: var(--info); border: 1px solid var(--info); }
  .no-issues { color: var(--success); font-size: 0.85rem; font-weight: 500; }

  .issue-detail {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 12px 16px;
  }

  .issue-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 10px;
    border-radius: 6px;
    font-size: 0.85rem;
  }

  .issue-item.issue-error { background: var(--bg-issue-error); }
  .issue-item.issue-warning { background: var(--bg-issue-warning); }
  .issue-item.issue-info { background: var(--bg-issue-info); }

  .issue-icon { font-size: 0.9rem; }
  .issue-element { font-weight: 600; color: var(--text); min-width: 80px; }
  .issue-message { color: var(--text-secondary); flex: 1; }
  .issue-selector {
    padding: 2px 6px;
    background: var(--bg-hover);
    border-radius: 4px;
    font-size: 0.75rem;
    color: var(--info);
    font-family: monospace;
  }

  /* ==========================================
     RESPONSIVE — Mobile First
     ========================================== */

  /* Mobile base (≤ 767px): card layout, minimal columns */
  .header-table { display: none; }

  .table-row {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 12px 16px;
    min-height: auto;
  }

  .col-url { width: 100%; }
  .col-url a { font-size: 0.85rem; }

  .col-status {
    position: absolute;
    top: 12px;
    right: 16px;
  }

  .col-title {
    width: 100%;
    font-size: 0.82rem;
    color: var(--text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .col-desc,
  .col-h1,
  .col-lang,
  .col-hreflang {
    display: none;
  }

  .col-issues {
    width: 100%;
  }

  .btn-detail {
    opacity: 1;
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

  /* Tablet (768px+): show more columns */
  @media (min-width: 768px) {
    .header-table {
      display: table;
    }

    .table-row {
      display: grid;
      grid-template-columns: 28% 8% 20% 14% 12% 18%;
      min-height: 48px;
      padding: 0 16px;
      align-items: center;
    }

    .col-desc,
    .col-h1,
    .col-lang,
    .col-hreflang {
      display: none;
    }

    .col-url { width: auto; }
    .col-status { width: auto; position: static; }
    .col-title { width: auto; font-size: 0.9rem; }
    .col-issues { width: auto; }
  }

  /* Desktop (1024px+): show all columns */
  @media (min-width: 1024px) {
    .table-row {
      grid-template-columns: 30% 6% 14% 16% 10% 5% 8% 11%;
    }

    .col-desc,
    .col-h1,
    .col-lang,
    .col-hreflang {
      display: block;
    }
  }

  .search-bar {
    display: flex;
    align-items: center;
    gap: var(--space-sm, 0.5rem);
    padding: var(--space-sm, 0.5rem) var(--space-md, 1rem);
    border-bottom: 1px solid var(--border, #e0e0e0);
    background: var(--bg-secondary, #f9fafb);
  }

  .search-input-wrap {
    display: flex;
    align-items: center;
    gap: 6px;
    flex: 1;
    background: var(--bg-primary, #fff);
    border: 1px solid var(--border, #d1d5db);
    border-radius: var(--radius-md, 8px);
    padding: 6px 10px;
    transition: border-color 0.15s, box-shadow 0.15s;
  }
  .search-input-wrap:focus-within {
    border-color: var(--accent, #3b82f6);
    box-shadow: 0 0 0 3px rgba(59,130,246,0.1);
  }

  .search-icon {
    flex-shrink: 0;
    color: var(--text-secondary, #9ca3af);
  }

  .search-input {
    flex: 1;
    border: none;
    background: transparent;
    font-size: 0.875rem;
    color: var(--text-primary, #111);
    outline: none;
    padding: 0;
  }
  .search-input::placeholder {
    color: var(--text-secondary, #9ca3af);
  }

  .search-clear {
    background: none;
    border: none;
    font-size: 1.1rem;
    cursor: pointer;
    color: var(--text-secondary, #9ca3af);
    padding: 0;
    line-height: 1;
    flex-shrink: 0;
  }
  .search-clear:hover {
    color: var(--text-primary, #374151);
  }

  .search-count {
    font-size: 0.8rem;
    color: var(--text-secondary, #6b7280);
    white-space: nowrap;
    padding: 2px 8px;
    background: var(--bg-primary, #fff);
    border: 1px solid var(--border, #e5e7eb);
    border-radius: var(--radius-sm, 4px);
  }

  :global(.table-wrapper mark) {
    background: var(--bg-mark);
    color: inherit;
    padding: 0 1px;
    border-radius: 2px;
  }
</style>
