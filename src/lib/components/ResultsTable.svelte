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
  let scrollContainer = $state<HTMLElement | null>(null);

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
    if (scrollContainer) {
      scrollContainer.scrollTop = scrollContainer.scrollTop;
    }
  }

  function truncateWords(text: string, maxLength: number = 160) {
    if (!text || text.length <= maxLength) return text?.trim() || "";

    let truncated = text.slice(0, maxLength);
    // Busca el último espacio para no cortar palabras
    let lastSpace = truncated.lastIndexOf(' ');
    if (lastSpace > 0) {
      truncated = truncated.slice(0, lastSpace);
    }
    return truncated.trim() + "...";
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
  <div class="header-row">
    <div class="col-url">{m["results.col.title"]()}</div>
    <div class="col-status">{m["results.col.status"]()}</div>
    <div class="col-issues">{m["results.col.issues"]()}</div>
  </div>

  {#if items.length === 0 && localSearch}
    <div class="empty-state">No results match "{localSearch}"</div>
  {:else if items.length === 0}
    <div class="empty-state">{m["results.no_results"]()}</div>
  {:else}
    <div class="rows-body" bind:this={scrollContainer}>
      {#each items as page (page.id)}
        {@const issues = parseIssues(page.semantic_issues_json)}
        {@const issueCounts = getIssueCounts(issues)}
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
            <button class="btn-title" onclick={() => onDetail?.(page.id)} title="View details">
              {@html highlight(page.title || 'Sin título', localSearch)}
            </button>
          </div>
          <div class="col-status status-{Math.floor(page.status_code / 100)}xx">
            {page.status_code}
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
        {#if expandedUrl === page.url}
          {@const issues = parseIssues(page.semantic_issues_json)}
          <div class="table-row detail-row">
            <div class="issue-detail">
              {#each issues as issue}
                {@const params = parseIssueParams(issue.message, issue.issue_type)}
                <div class="issue-item issue-{issue.severity}">
                  <span class="issue-icon">{getSeverityIcon(issue.severity)}</span>
                  <span class="issue-element">{issue.element}</span>
                  <span class="issue-message">{translateIssueMessage(issue.issue_type, params)}</span>
                  {#if issue.xpath && !issue.issue_type.startsWith('missing_')}
                    <code class="issue-selector">{issue.xpath}</code>
                  {:else if issue.xpath}
                    <span class="issue-detail issue-selector">{issue.xpath}</span>
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

  .table-wrapper {
    overflow: auto;
    max-height: 600px;
    border-radius: 8px;
    border: 1px solid var(--border);
  }

  .header-row {
    display: grid;
    grid-template-columns: 70% 10% 20%;
    position: sticky;
    top: 0;
    z-index: 10;
    background: var(--bg-card);
    border-bottom: 2px solid var(--border);
    box-shadow: 0 2px 8px rgba(0,0,0,0.08);
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
    border-top-left-radius: 8px;
  }

  .header-row > div:last-child {
    border-top-right-radius: 8px;
  }

  [data-theme="dark"] .header-row {
    background: #2f323a;
    border-bottom-color: #5c5f66;
    box-shadow: 0 2px 8px rgba(0,0,0,0.45);
  }

  [data-theme="dark"] .header-row > div {
    color: #f1f3f5;
    border-right-color: #5c5f66;
  }

  [data-theme="dark"] .table-row {
    border-bottom-color: #5c5f66;
  }

  [data-theme="dark"] .main-row:hover {
    background: #3a3d47;
    box-shadow: inset 0 0 0 1px #5c5f66;
  }

  [data-theme="dark"] .detail-row {
    border-bottom-color: #5c5f66;
    background: #1a1b1e;
  }

  .rows-body {
    overflow: visible;
  }

  .table-row {
    display: grid;
    grid-template-columns: 70% 10% 20%;
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

  [data-theme="dark"] .main-row:hover {
    background: #353840;
  }

  .detail-row {
    display: block;
    background: var(--bg-deep);
    border-bottom: 2px solid var(--border);
    padding: 0;
    grid-column: 1 / -1;
    width: 100%;
  }

  [data-theme="dark"] .detail-row {
    background: #1e2028;
    border-bottom-color: #4a4d55;
  }

  .col-url {
    width: 70%;
    display: flex;
    flex-direction: column;
    gap: 2px;
    align-items: flex-start;
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
    transition: text-decoration-color 0.15s;
  }

  .btn-title:hover {
    text-decoration-color: var(--accent);
    color: var(--accent-hover);
  }

  .col-status { width: 10%; }
  .col-issues { width: 20%; }

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

  /* Mobile base (<= 767px): card layout, minimal columns */
  .header-row { display: none; }

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

  /* Tablet (768px+): show 3-column table */
  @media (min-width: 768px) {
    .header-row {
      display: grid;
      grid-template-columns: 70% 10% 20%;
    }

    .table-row {
      display: grid;
      grid-template-columns: 70% 10% 20%;
      min-height: 48px;
      padding: 0 16px;
      align-items: center;
    }

    .col-title,
    .col-desc,
    .col-h1,
    .col-lang,
    .col-hreflang {
      display: none;
    }

    .col-url { width: auto; }
    .col-status { width: auto; position: static; }
    .col-issues { width: auto; }
  }

  /* Desktop (1024px+): keep 3 columns */
  @media (min-width: 1024px) {
    .header-row,
    .table-row {
      grid-template-columns: 70% 10% 20%;
    }

    .col-title,
    .col-desc,
    .col-h1,
    .col-lang,
    .col-hreflang {
      display: none;
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
