<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { m } from '$lib/paraglide/messages.js';
  import { translateIssueName, translateIssueMessage, parseIssueParams, translateSeverity } from '$lib/i18n-issues';

  let {
    pageId = $bindable(''),
    onClose,
  }: {
    pageId: string;
    onClose: () => void;
  } = $props();

  let detail = $state<any>(null);
  let links = $state<any[]>([]);
  let loading = $state(false);
  let error = $state('');
  let activeTab = $state<'overview' | 'links'>('overview');

  $effect(() => {
    if (pageId) loadDetail();
    else { detail = null; links = []; activeTab = 'overview'; }
  });

  async function loadDetail() {
    loading = true;
    error = '';
    activeTab = 'overview';
    try {
      const result = await invoke<[any, any[]]>('get_page_detail', { pageId });
      detail = result[0];
      links = result[1];
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  let copiedField = $state('');
  async function copyToClipboard(text: string, field: string) {
    try {
      // Tauri webview doesn't support navigator.clipboard — use textarea fallback
      const ta = document.createElement('textarea');
      ta.value = text;
      ta.style.position = 'fixed';
      ta.style.left = '-9999px';
      document.body.appendChild(ta);
      ta.select();
      document.execCommand('copy');
      document.body.removeChild(ta);
      copiedField = field;
      setTimeout(() => { if (copiedField === field) copiedField = ''; }, 1500);
    } catch {}
  }

  function parseIssues(json: string | null): any[] {
    if (!json) return [];
    try { return JSON.parse(json); } catch { return []; }
  }

  function parseHreflang(json: string | null): { lang: string; href: string }[] {
    if (!json) return [];
    try { return JSON.parse(json); } catch { return []; }
  }

  function getSeverityColor(severity: string): string {
    if (severity === 'error') return '#ff6b6b';
    if (severity === 'warning') return '#ffd43b';
    return '#74c0fc';
  }

</script>

{#if pageId}
  <div class="fullpage">
    <header class="fullpage-header">
      <div class="header-left">
        <button class="btn-back" onclick={onClose} aria-label="Back to results">{m["detail.back"]()}</button>
        <h3 class="page-url" title={detail?.url}>{detail?.url || m["detail.loading"]()}</h3>
      </div>
      <div class="header-right">
        {#if detail}
          <span class="header-badge status-{Math.floor(detail.status_code / 100)}xx">{detail.status_code}</span>
          <span class="header-meta">{detail.size_bytes ? `${(detail.size_bytes / 1024).toFixed(1)} KB` : ''}</span>
          <span class="header-meta">{detail.load_time_ms ? `${detail.load_time_ms}ms` : ''}</span>
        {/if}
      </div>
    </header>

    <!-- Tab Bar -->
    <div class="tab-bar">
      <button
        class="tab"
        class:active={activeTab === 'overview'}
        onclick={() => activeTab = 'overview'}
      >
        {m["detail.overview"]()}
      </button>
      <button
        class="tab"
        class:active={activeTab === 'links'}
        onclick={() => activeTab = 'links'}
      >
        {m["detail.links"]({ count: links.length.toString() })}
      </button>
    </div>

    {#if loading}
      <div class="fullpage-loading">{m["detail.loading"]()}</div>
    {:else if error}
      <div class="fullpage-error">{error}</div>
    {:else if detail}
      <div class="fullpage-body">
        {#if activeTab === 'overview'}
          <div class="overview-grid">
            <div class="overview-section">
              <h4>{m["detail.seo_meta"]()}</h4>
              <div class="field">
                <span class="field-label">{m["detail.title"]()}</span>
                <span class="field-value">{detail.title || m["detail.missing"]()}</span>
              </div>
              <div class="field">
                <span class="field-label">{m["detail.meta_description"]()}</span>
                <span class="field-value">{detail.meta_description || m["detail.missing"]()}</span>
              </div>
              <div class="field">
                <span class="field-label">{m["detail.h1"]()}</span>
                <span class="field-value">{detail.h1 || m["detail.missing"]()}</span>
              </div>
              <div class="field-row">
                <div class="field">
                  <span class="field-label">{m["detail.canonical"]()}</span>
                  <span class="field-value">{detail.canonical || m["detail.none"]()}</span>
                </div>
                <div class="field">
                  <span class="field-label">{m["detail.html_lang"]()}</span>
                  <span class="field-value">{detail.html_lang || m["detail.none"]()}</span>
                </div>
              </div>
              <div class="field">
                <span class="field-label">{m["detail.indexable"]()}</span>
                <span class="field-value">{detail.is_indexable === true ? m["detail.yes"]() : detail.is_indexable === false ? m["detail.no"]() : m["detail.unknown"]()}</span>
              </div>
            </div>

            <div class="overview-section">
              <h4>{m["detail.crawl_info"]()}</h4>
              <div class="field-row">
                <div class="field">
                  <span class="field-label">{m["detail.status"]()}</span>
                  <span class="field-value status-code status-{Math.floor(detail.status_code / 100)}xx">{detail.status_code}</span>
                </div>
                <div class="field">
                  <span class="field-label">{m["detail.depth"]()}</span>
                  <span class="field-value">{detail.depth}</span>
                </div>
                <div class="field">
                  <span class="field-label">{m["detail.size"]()}</span>
                  <span class="field-value">{detail.size_bytes ? `${(detail.size_bytes / 1024).toFixed(1)} KB` : '-'}</span>
                </div>
                <div class="field">
                  <span class="field-label">{m["detail.load_time"]()}</span>
                  <span class="field-value">{detail.load_time_ms ? `${detail.load_time_ms}ms` : '-'}</span>
                </div>
              </div>
              {#if detail.parent_url}
                <div class="field">
                  <span class="field-label">{m["detail.discovered_from"]()}</span>
                  <a href={detail.parent_url} target="_blank" class="field-value">{detail.parent_url}</a>
                </div>
              {/if}
            </div>

            {#if parseHreflang(detail.hreflang_json).length > 0}
              <div class="overview-section">
                <h4>{m["detail.hreflang"]()}</h4>
                <div class="hreflang-list">
                  {#each parseHreflang(detail.hreflang_json) as hl}
                    <span class="hreflang-tag">{hl.lang}: {hl.href}</span>
                  {/each}
                </div>
              </div>
            {/if}

            {#if parseIssues(detail.semantic_issues_json).length > 0}
              <div class="overview-section overview-full">
                <h4>{m["detail.semantic_issues"]({ count: parseIssues(detail.semantic_issues_json).length.toString() })}</h4>
                <div class="issue-list">
                  {#each parseIssues(detail.semantic_issues_json) as issue}
                    {@const params = parseIssueParams(issue.message, issue.issue_type)}
                    <div class="issue-card issue-{issue.severity}">
                      <div class="issue-header">
                        <span class="issue-severity" style="background: {getSeverityColor(issue.severity)}20; color: {getSeverityColor(issue.severity)}; border: 1px solid {getSeverityColor(issue.severity)}40">{translateSeverity(issue.severity)}</span>
                        <span class="issue-type">{translateIssueName(issue.issue_type)}</span>
                      </div>
                      <div class="issue-message">{translateIssueMessage(issue.issue_type, params)}</div>
                      <div class="issue-details">
                        {#if issue.element}
                          <span class="issue-detail"><code>{issue.element}</code></span>
                        {/if}
                        {#if issue.selector && !issue.issue_type.startsWith('missing_')}
                          <button
                            class="issue-detail issue-selector copy-btn"
                            class:copied={copiedField === `sel-${issue.selector}`}
                            onclick={() => copyToClipboard(issue.selector, `sel-${issue.selector}`)}
                            title={m["detail.copy_selector"]()}
                          >
                            <span class="copy-icon">{copiedField === `sel-${issue.selector}` ? '✓' : '⎘'}</span>
                            <span class="copy-text">{copiedField === `sel-${issue.selector}` ? m["detail.copied"]() : m["detail.copy_selector"]()}</span>
                            <code class="copy-value">{issue.selector}</code>
                          </button>
                        {:else if issue.selector}
                          <span class="issue-detail issue-selector">{issue.selector}</span>
                        {/if}
                      </div>
                    </div>
                  {/each}
                </div>
              </div>
            {/if}
          </div>

        {:else if activeTab === 'links'}
          {#if links.length > 0}
            <div class="links-table">
              <div class="links-header">
                <span>{m["detail.links_type"]()}</span>
                <span>{m["detail.links_url"]()}</span>
                <span>{m["detail.links_anchor"]()}</span>
              </div>
              {#each links as link}
                <div class="links-row">
                  <span class="link-type link-type-{link.link_type}">{link.link_type}</span>
                  <a href={link.to_url} target="_blank" class="link-url">{link.to_url}</a>
                  <span class="link-anchor">{link.anchor_text || '-'}</span>
                </div>
              {/each}
            </div>
          {:else}
            <div class="empty-tab">{m["detail.no_links"]()}</div>
          {/if}
        {/if}
      </div>
    {/if}
  </div>
{/if}

<style>
  .fullpage {
    position: fixed;
    inset: 0;
    background: var(--bg-deep);
    z-index: 200;
    display: flex;
    flex-direction: column;
  }

  /* Header */
  .fullpage-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 20px;
    background: var(--bg-card);
    border-bottom: 1px solid var(--bg-hover);
    flex-shrink: 0;
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 16px;
    min-width: 0;
    flex: 1;
  }

  .btn-back {
    padding: 6px 12px;
    background: var(--border);
    border: 1px solid var(--text-muted);
    border-radius: 6px;
    color: var(--text);
    font-size: 0.8rem;
    cursor: pointer;
    white-space: nowrap;
    flex-shrink: 0;
  }
  .btn-back:hover { background: #4a4d54; }

  .page-url {
    margin: 0;
    font-size: 0.9rem;
    color: var(--text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .header-right {
    display: flex;
    align-items: center;
    gap: 12px;
    flex-shrink: 0;
    margin-left: 16px;
  }

  .header-badge {
    font-size: 0.75rem;
    font-weight: 700;
    padding: 2px 8px;
    border-radius: 4px;
  }
  .status-2xx { background: #1a3a2a; color: var(--success); }
  .status-3xx { background: #3d3520; color: var(--warning); }
  .status-4xx { background: #3d2a1a; color: var(--orange); }
  .status-5xx { background: #3d1f1f; color: var(--danger); }

  .header-meta {
    font-size: 0.8rem;
    color: var(--text-muted);
  }

  /* Tabs */
  .tab-bar {
    display: flex;
    border-bottom: 1px solid var(--bg-hover);
    padding: 0 20px;
    background: var(--bg-deep);
    flex-shrink: 0;
  }

  .tab {
    padding: 10px 16px;
    background: none;
    border: none;
    border-bottom: 2px solid transparent;
    color: var(--text-muted);
    font-size: 0.82rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s;
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .tab:hover { color: var(--text-secondary); }
  .tab.active { color: var(--accent); border-bottom-color: var(--accent); }

  /* Body */
  .fullpage-body {
    flex: 1;
    overflow-y: auto;
    padding: 24px;
  }

  .fullpage-loading, .fullpage-error {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-muted);
    font-size: 0.9rem;
  }
  .fullpage-error { color: var(--danger); }

  /* Overview */
  .overview-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(340px, 1fr));
    gap: 24px;
    max-width: 1200px;
  }

  .overview-section {
    background: var(--bg-card);
    border-radius: 10px;
    padding: 20px;
  }

  .overview-section h4 {
    margin: 0 0 14px 0;
    font-size: 0.8rem;
    text-transform: uppercase;
    color: var(--text-muted);
    font-weight: 600;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin-bottom: 10px;
  }

  .field-label {
    font-size: 0.72rem;
    text-transform: uppercase;
    color: var(--text-muted);
    font-weight: 600;
  }

  .field-value {
    font-size: 0.88rem;
    color: var(--text);
    word-break: break-all;
  }

  a.field-value {
    color: var(--accent);
    text-decoration: none;
  }
  a.field-value:hover { text-decoration: underline; }

  .field-row {
    display: flex;
    gap: 16px;
  }
  .field-row .field { flex: 1; }

  .status-2xx { color: var(--success); font-weight: 600; }
  .status-3xx { color: var(--warning); font-weight: 600; }
  .status-4xx { color: var(--orange); font-weight: 600; }
  .status-5xx { color: var(--danger); font-weight: 600; }

  .hreflang-list {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .hreflang-tag {
    font-size: 0.8rem;
    padding: 3px 10px;
    background: var(--bg-deep);
    border-radius: 4px;
    color: var(--text-secondary);
  }

  .overview-full {
    grid-column: 1 / -1;
  }

  .issue-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .issue-card {
    padding: 12px 14px;
    background: var(--bg-deep);
    border-radius: 8px;
    border-left: 3px solid transparent;
  }
  .issue-card.issue-error { border-left-color: var(--danger); }
  .issue-card.issue-warning { border-left-color: var(--warning); }
  .issue-card.issue-info { border-left-color: var(--info); }

  .issue-header {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-bottom: 4px;
  }

  .issue-severity {
    font-size: 0.65rem;
    font-weight: 700;
    text-transform: uppercase;
    padding: 2px 8px;
    border-radius: 4px;
    letter-spacing: 0.5px;
  }

  .issue-type {
    font-size: 0.82rem;
    font-weight: 600;
    color: var(--text);
    text-transform: capitalize;
  }

  .issue-message {
    font-size: 0.8rem;
    color: var(--text-secondary);
    margin-bottom: 6px;
    line-height: 1.4;
  }

  .issue-details {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    font-size: 0.72rem;
  }

  .issue-detail {
    color: var(--text-muted);
  }
  .issue-detail code {
    font-family: 'SF Mono', 'Fira Code', monospace;
    color: var(--info);
    background: #1a2a3a;
    padding: 2px 6px;
    border-radius: 3px;
    font-size: 0.72rem;
  }

  .issue-selector {
    color: var(--purple);
    font-family: 'SF Mono', 'Fira Code', monospace;
    font-size: 0.7rem;
  }

  .issue-xpath {
    color: var(--text-muted);
    font-family: 'SF Mono', 'Fira Code', monospace;
    font-size: 0.7rem;
    max-width: 400px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .copy-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    background: var(--bg-deep);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 4px 10px;
    cursor: pointer;
    transition: all 0.15s;
    text-align: left;
    font-size: 0.72rem;
    color: var(--text-secondary);
  }
  .copy-btn:hover {
    background: var(--bg-card);
    border-color: var(--accent);
    color: var(--text);
  }
  .copy-btn.copied {
    border-color: var(--success);
    color: var(--success);
  }

  .copy-icon {
    font-size: 0.85rem;
    flex-shrink: 0;
  }

  .copy-text {
    font-weight: 600;
    white-space: nowrap;
  }

  .copy-value {
    font-family: 'SF Mono', 'Fira Code', monospace;
    font-size: 0.68rem;
    color: var(--purple);
    max-width: 350px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    background: none;
    padding: 0;
  }

  /* Links */
  .links-table {
    max-width: 1000px;
  }

  .links-header {
    display: grid;
    grid-template-columns: 60px 1fr 150px;
    gap: 12px;
    padding: 8px 12px;
    font-size: 0.72rem;
    text-transform: uppercase;
    color: var(--text-muted);
    font-weight: 600;
    border-bottom: 1px solid var(--bg-hover);
  }

  .links-row {
    display: grid;
    grid-template-columns: 60px 1fr 150px;
    gap: 12px;
    padding: 8px 12px;
    font-size: 0.82rem;
    align-items: center;
    border-bottom: 1px solid var(--bg-card);
  }
  .links-row:hover { background: var(--bg-card); }

  .link-type {
    font-size: 0.7rem;
    padding: 2px 6px;
    border-radius: 3px;
    font-weight: 600;
    text-transform: uppercase;
    text-align: center;
  }
  .link-type-internal { background: #1a2a3a; color: var(--info); }
  .link-type-external { background: #2a1a3a; color: var(--purple); }

  .link-url {
    color: var(--text-secondary);
    text-decoration: none;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .link-url:hover { color: var(--accent); }

  .link-anchor {
    color: var(--text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .empty-tab {
    text-align: center;
    color: var(--text-muted);
    padding: 60px 20px;
    font-size: 0.9rem;
  }
</style>
