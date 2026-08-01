<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { m } from '$lib/paraglide/messages.js';
  import { translateIssueName, translateIssueMessage, parseIssueParams, translateSeverity } from '$lib/i18n-issues';
  import { ArrowLeft, X, Copy, Check } from 'lucide-svelte';
  import * as Tabs from '$lib/components/ui/tabs/index.js';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card/index.js';
  import { Button } from '$lib/components/ui/button/index.js';
  import { Skeleton } from '$lib/components/ui/skeleton/index.js';
  import { cn } from '$lib/utils.js';

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
      if (navigator.clipboard?.writeText) {
        await navigator.clipboard.writeText(text);
      } else {
        const ta = document.createElement('textarea');
        ta.value = text;
        ta.style.position = 'fixed';
        ta.style.left = '-9999px';
        document.body.appendChild(ta);
        ta.select();
        document.execCommand('copy');
        document.body.removeChild(ta);
      }
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

</script>

{#if pageId}
  <div class="fullpage">
    <header class="fullpage-header">
      <div class="header-left">
        <Button variant="outline" size="sm" class="flex-shrink-0 gap-1.5" onclick={onClose}>
          <ArrowLeft class="size-4" />
          {m["detail.back"]()}
        </Button>
        <h3 class="page-url" title={detail?.url}>{detail?.url || m["detail.loading"]()}</h3>
      </div>
      <div class="header-right">
        {#if detail}
          <span class="header-badge status-{Math.floor(detail.status_code / 100)}xx">{detail.status_code}</span>
          <span class="header-meta">{detail.size_bytes ? `${(detail.size_bytes / 1024).toFixed(1)} KB` : ''}</span>
          <span class="header-meta">{detail.load_time_ms ? `${detail.load_time_ms}ms` : ''}</span>
        {/if}
        <Button
          variant="ghost"
          size="icon-sm"
          onclick={onClose}
          aria-label={m["detail.close"]()}
          title={m["detail.close"]()}
        >
          <X class="size-4" />
        </Button>
      </div>
    </header>

    <Tabs.Root
      bind:value={activeTab}
      class="flex min-h-0 flex-1 flex-col gap-0 overflow-hidden"
    >
      <Tabs.List
        variant="line"
        class="flex-shrink-0 rounded-none border-b border-border px-4"
      >
        <Tabs.Trigger value="overview">{m["detail.overview"]()}</Tabs.Trigger>
        <Tabs.Trigger value="links">{m["detail.links"]({ count: links.length.toString() })}</Tabs.Trigger>
      </Tabs.List>

      {#if loading}
        <Tabs.Content value="overview" class="min-h-0 flex-1 overflow-hidden">
          <div class="fullpage-body">
            <div class="overview-grid">
              {#each [0, 1] as i (i)}
                <Card size="sm">
                  <CardHeader class="pb-2">
                    <Skeleton class="h-3.5 w-28" />
                  </CardHeader>
                  <CardContent class="flex flex-col gap-3">
                    {#each [0, 1, 2, 3] as j (j)}
                      <div class="flex flex-col gap-1.5">
                        <Skeleton class="h-2.5 w-16" />
                        <Skeleton class="h-4 w-full" />
                      </div>
                    {/each}
                  </CardContent>
                </Card>
              {/each}
            </div>
          </div>
        </Tabs.Content>
      {:else if error}
        <div class="fullpage-error">{error}</div>
      {:else if detail}
        <Tabs.Content value="overview" class="min-h-0 flex-1 overflow-hidden">
          <div class="fullpage-body">
            <div class="overview-grid">
            <Card size="sm">
              <CardHeader class="pb-2">
                <CardTitle class="text-xs uppercase tracking-wider text-muted-foreground">{m["detail.seo_meta"]()}</CardTitle>
              </CardHeader>
              <CardContent class="flex flex-col gap-2">
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
              </CardContent>
            </Card>

            <Card size="sm">
              <CardHeader class="pb-2">
                <CardTitle class="text-xs uppercase tracking-wider text-muted-foreground">{m["detail.crawl_info"]()}</CardTitle>
              </CardHeader>
              <CardContent class="flex flex-col gap-2">
                <div class="field-row stats-row">
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
              </CardContent>
            </Card>

            {#if parseHreflang(detail.hreflang_json).length > 0}
              <Card size="sm">
                <CardHeader class="pb-2">
                  <CardTitle class="text-xs uppercase tracking-wider text-muted-foreground">{m["detail.hreflang"]()}</CardTitle>
                </CardHeader>
                <CardContent>
                  <div class="hreflang-list">
                    {#each parseHreflang(detail.hreflang_json) as hl}
                      <span class="hreflang-tag">{hl.lang}: {hl.href}</span>
                    {/each}
                  </div>
                </CardContent>
              </Card>
            {/if}

            {#if parseIssues(detail.semantic_issues_json).length > 0}
              <Card size="sm" class="overview-full">
                <CardHeader class="pb-2">
                  <CardTitle class="text-xs uppercase tracking-wider text-muted-foreground">
                    {m["detail.semantic_issues"]({ count: parseIssues(detail.semantic_issues_json).length.toString() })}
                  </CardTitle>
                </CardHeader>
                <CardContent>
                  <div class="issue-list">
                    {#each parseIssues(detail.semantic_issues_json) as issue}
                      {@const params = parseIssueParams(issue.message, issue.issue_type)}
                      <div class="issue-card issue-{issue.severity}">
                        <div class="issue-header">
                          <span class="issue-severity issue-severity-{issue.severity}">{translateSeverity(issue.severity)}</span>
                          <span class="issue-type">{translateIssueName(issue.issue_type)}</span>
                        </div>
                        <div class="issue-message">{translateIssueMessage(issue.issue_type, params)}</div>
                        <div class="issue-details">
                          {#if issue.element}
                            <span class="issue-detail"><code>{issue.element}</code></span>
                          {/if}
                          {#if issue.xpath && !issue.issue_type.startsWith('missing_')}
                            <Button
                              variant="outline"
                              size="sm"
                              class={cn(
                                'issue-detail issue-xpath copy-btn h-auto px-2.5 py-1.5',
                                copiedField === `xpath-${issue.xpath}` && 'copied'
                              )}
                              onclick={() => copyToClipboard(issue.xpath, `xpath-${issue.xpath}`)}
                              title={m["detail.copy_xpath"]()}
                            >
                              <span class="copy-icon">
                                {#if copiedField === `xpath-${issue.xpath}`}
                                  <Check class="size-3.5" />
                                {:else}
                                  <Copy class="size-3.5" />
                                {/if}
                              </span>
                              <span class="copy-text">{copiedField === `xpath-${issue.xpath}` ? m["detail.copied"]() : m["detail.copy_xpath"]()}</span>
                              <code class="copy-value">{issue.xpath}</code>
                            </Button>
                          {:else if issue.xpath}
                            <span class="issue-detail issue-xpath">{issue.xpath}</span>
                          {/if}
                        </div>
                      </div>
                    {/each}
                  </div>
                </CardContent>
              </Card>
            {/if}
          </div>
          </div>
        </Tabs.Content>

        <Tabs.Content value="links" class="min-h-0 flex-1 overflow-hidden">
          <div class="fullpage-body">
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
          </div>
        </Tabs.Content>
      {/if}
    </Tabs.Root>
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
    overflow: hidden;
    animation: slide-in var(--transition-slow) ease;
  }

  @keyframes slide-in {
    from { opacity: 0; transform: translateX(24px); }
    to { opacity: 1; transform: translateX(0); }
  }

  /* Header */
  .fullpage-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
    padding: calc(12px + env(safe-area-inset-top)) 20px 12px;
    background: var(--bg-card);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
    z-index: 10;
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 16px;
    min-width: 0;
    flex: 1;
  }

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
  }

  .header-badge {
    font-size: 0.75rem;
    font-weight: 700;
    padding: 3px 10px;
    border-radius: var(--radius-pill);
    font-variant-numeric: tabular-nums;
  }
  .status-2xx { background: var(--bg-status-2xx); color: var(--success); }
  .status-3xx { background: var(--bg-status-3xx); color: var(--warning); }
  .status-4xx { background: var(--bg-status-4xx); color: var(--orange); }
  .status-5xx { background: var(--bg-status-5xx); color: var(--danger); }

  .header-meta {
    font-size: 0.8rem;
    color: var(--text-muted);
    white-space: nowrap;
  }

  /* Body */
  .fullpage-body {
    height: 100%;
    overflow-y: auto;
    padding: 24px;
    padding-bottom: calc(24px + env(safe-area-inset-bottom, 0px));
    overscroll-behavior: contain;
  }

  .fullpage-error {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--danger);
    font-size: 0.9rem;
  }

  /* Overview */
  .overview-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(min(100%, 340px), 1fr));
    gap: 24px;
    max-width: 1200px;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-width: 0;
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
  a.field-value:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
    border-radius: 2px;
  }

  .field-row {
    display: flex;
    gap: 16px;
    flex-wrap: wrap;
  }
  .field-row .field { flex: 1 1 0; min-width: 140px; }

  .status-code { font-weight: 600; }

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
    flex-wrap: wrap;
  }

  .issue-severity {
    font-size: 0.65rem;
    font-weight: 700;
    text-transform: uppercase;
    padding: 2px 8px;
    border-radius: 4px;
    letter-spacing: 0.5px;
  }

  .issue-severity-error {
    background: var(--bg-issue-error);
    color: var(--danger);
    border: 1px solid var(--danger);
  }

  .issue-severity-warning {
    background: var(--bg-issue-warning);
    color: var(--warning);
    border: 1px solid var(--warning);
  }

  .issue-severity-info {
    background: var(--bg-issue-info);
    color: var(--info);
    border: 1px solid var(--info);
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
    font-family: var(--font-mono);
    color: var(--info);
    background: var(--bg-issue-info);
    padding: 2px 6px;
    border-radius: 3px;
    font-size: 0.72rem;
  }

  .issue-xpath {
    color: var(--text-muted);
    font-family: var(--font-mono);
    font-size: 0.7rem;
    max-width: 400px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .copy-btn {
    gap: 6px;
    color: var(--text-secondary);
  }
  .copy-btn:hover {
    border-color: var(--accent);
    color: var(--text);
  }
  .copy-btn.copied {
    border-color: var(--success);
    color: var(--success);
  }

  .copy-icon {
    display: inline-flex;
    flex-shrink: 0;
  }

  .copy-text {
    font-weight: 600;
    white-space: nowrap;
  }

  .copy-value {
    font-family: var(--font-mono);
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
    grid-template-columns: 60px minmax(0, 1fr) 150px;
    gap: 12px;
    padding: 8px 12px;
    font-size: 0.72rem;
    text-transform: uppercase;
    color: var(--text-muted);
    font-weight: 600;
    border-bottom: 1px solid var(--border);
  }

  .links-row {
    display: grid;
    grid-template-columns: 60px minmax(0, 1fr) 150px;
    gap: 12px;
    padding: 8px 12px;
    font-size: 0.82rem;
    align-items: center;
    border-bottom: 1px solid var(--border);
    transition: background var(--transition-fast);
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
  .link-type-internal { background: var(--bg-link-internal); color: var(--info); }
  .link-type-external { background: var(--bg-link-external); color: var(--purple); }

  .link-url {
    color: var(--text-secondary);
    text-decoration: none;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .link-url:hover { color: var(--accent); }
  .link-url:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
    border-radius: 2px;
  }

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

  /* ========== Responsive ========== */

  @media (max-width: 767px) {
    .fullpage-header {
      padding: calc(10px + env(safe-area-inset-top)) 12px 10px;
      gap: 8px;
    }

    .header-left {
      gap: 10px;
    }

    .header-right {
      gap: 8px;
    }

    .header-meta {
      display: none;
    }

    .fullpage-body {
      padding: 16px;
      padding-bottom: calc(16px + env(safe-area-inset-bottom, 0px));
    }

    .overview-grid {
      gap: 16px;
    }

    .field-row .field {
      min-width: 120px;
    }
  }

  @media (max-width: 640px) {
    .links-header {
      display: none;
    }

    .links-row {
      grid-template-columns: 1fr;
      gap: 4px;
      padding: 10px 12px;
      border: 1px solid var(--border);
      border-radius: var(--radius-md);
      margin-bottom: 8px;
      background: var(--bg-card);
    }

    .link-type {
      justify-self: start;
    }

    .link-anchor {
      font-size: 0.78rem;
      white-space: normal;
      overflow-wrap: break-word;
    }

    .link-url {
      white-space: normal;
      overflow-wrap: break-word;
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .fullpage {
      animation: none;
    }
  }
</style>
