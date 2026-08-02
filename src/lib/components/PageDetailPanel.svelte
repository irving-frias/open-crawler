<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { m } from '$lib/paraglide/messages.js';
  import { translateIssueName, translateIssueMessage, parseIssueParams, translateSeverity } from '$lib/i18n-issues';
  import { ArrowLeft, X, Copy, Check } from 'lucide-svelte';
  import * as Tabs from '$lib/components/ui/tabs/index.js';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card/index.js';
  import { Button } from '$lib/components/ui/button/index.js';
  import { Skeleton } from '$lib/components/ui/skeleton/index.js';
  import { Badge } from '$lib/components/ui/badge/index.js';
  import { ScrollArea } from '$lib/components/ui/scroll-area/index.js';
  import { Separator } from '$lib/components/ui/separator/index.js';
  import { Tooltip, TooltipTrigger, TooltipContent, TooltipProvider } from '$lib/components/ui/tooltip/index.js';
  import { Accordion, AccordionItem, AccordionHeader, AccordionTrigger, AccordionContent } from '$lib/components/ui/accordion/index.js';
  import { Table, TableHeader, TableBody, TableRow, TableHead, TableCell } from '$lib/components/ui/table/index.js';
  import { Alert } from '$lib/components/ui/alert/index.js';
  import { Popover, PopoverTrigger, PopoverContent } from '$lib/components/ui/popover/index.js';
  import { Progress } from '$lib/components/ui/progress/index.js';
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
  let activeTab = $state<'overview' | 'links' | 'preview'>('overview');
  let previewHtml = $state<string | null>(null);
  let previewLoading = $state(false);
  let previewError = $state('');

  type Overlay = {
    xpath: string;
    x: number;
    y: number;
    severity: string;
    issueType: string;
    label: string;
  };

  let previewIframe = $state<HTMLIFrameElement | null>(null);
  let overlays = $state<Overlay[]>([]);
  let highlightedXpath = $state('');

  function evalXPath(doc: Document, xpath: string): Element | null {
    try {
      const result = doc.evaluate(xpath, doc, null, XPathResult.FIRST_ORDERED_NODE_TYPE, null);
      const node = result.singleNodeValue;
      return node?.nodeType === Node.ELEMENT_NODE ? (node as Element) : null;
    } catch {
      return null;
    }
  }

  function computeOverlays() {
    const frame = previewIframe;
    const doc = frame?.contentDocument;
    const win = frame?.contentWindow;
    if (!doc || !win || !detail) {
      overlays = [];
      return;
    }
    const root = doc.documentElement;
    const height = Math.max(root.scrollHeight, root.clientHeight, win.innerHeight || 0);
    frame.style.height = `${height}px`;
    overlays = computeOverlayList(doc, win);
  }

  function computeOverlayList(doc: Document, win: Window): Overlay[] {
    const issues = parseIssues(detail.semantic_issues_json);
    const list: Overlay[] = [];
    for (const issue of issues) {
      if (!issue.xpath) continue;
      const el = evalXPath(doc, issue.xpath);
      if (!el) continue;
      const rect = el.getBoundingClientRect();
      if (rect.width === 0 && rect.height === 0) continue;
      list.push({
        xpath: issue.xpath,
        x: Math.round(rect.left + win.scrollX),
        y: Math.round(rect.top + win.scrollY),
        severity: issue.severity || 'info',
        issueType: issue.issue_type,
        label: translateIssueName(issue.issue_type),
      });
    }
    return list;
  }

  function onPreviewLoad() {
    computeOverlays();
    setTimeout(computeOverlays, 600);
  }

  function jumpToOverlay(overlay: Overlay) {
    const doc = previewIframe?.contentDocument;
    if (!doc) return;
    if (highlightedXpath) {
      evalXPath(doc, highlightedXpath)?.classList.remove('oc-overlay-hl');
    }
    const el = evalXPath(doc, overlay.xpath);
    if (!el) return;
    el.classList.add('oc-overlay-hl');
    highlightedXpath = overlay.xpath;
    el.scrollIntoView({ block: 'center', inline: 'center' });
  }

  $effect(() => {
    if (pageId) loadDetail();
    else { detail = null; links = []; activeTab = 'overview'; previewHtml = null; }
  });

  $effect(() => {
    if (activeTab === 'preview' && pageId) loadPreview();
  });

  async function loadDetail() {
    loading = true;
    error = '';
    activeTab = 'overview';
    previewHtml = null;
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

  async function loadPreview() {
    previewLoading = true;
    previewError = '';
    previewHtml = null;
    overlays = [];
    try {
      const html = await invoke<string | null>('get_page_html', { pageId });
      let preview = html;
      if (html && detail?.url) {
        try {
          preview = await invoke<string>('inline_assets', { html, baseUrl: detail.url });
        } catch {
          preview = html;
        }
      }
      if (preview) {
        preview = preview.replace(/<script[\s\S]*?<\/script>/gi, '');
      }
      previewHtml = preview;
    } catch (e) {
      previewError = String(e);
    } finally {
      previewLoading = false;
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

  function statusBadgeVariant(code: number): 'default' | 'warning' | 'destructive' {
    if (code >= 500) return 'destructive';
    if (code >= 400) return 'warning';
    return 'default';
  }

  function severityBadgeVariant(severity: string): 'default' | 'warning' | 'destructive' {
    if (severity === 'error') return 'destructive';
    if (severity === 'warning') return 'warning';
    return 'default';
  }

  function truncateUrl(url: string, maxLen: number = 80): string {
    if (url.length <= maxLen) return url;
    return url.slice(0, maxLen - 3) + '...';
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
        {#if detail}
          <TooltipProvider>
            <Tooltip>
              <TooltipTrigger>
                <h3 class="page-url">
                  {detail.url}
                </h3>
              </TooltipTrigger>
              <TooltipContent>
                {detail.url}
              </TooltipContent>
            </Tooltip>
          </TooltipProvider>
        {/if}
      </div>
      <div class="header-right">
        {#if detail}
          <Badge variant={statusBadgeVariant(detail.status_code)}>{detail.status_code}</Badge>
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
        <Tabs.Trigger value="preview">{m["detail.preview"]()}</Tabs.Trigger>
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
        <div class="fullpage-error">
          <Alert variant="destructive">
            {error}
          </Alert>
        </div>
      {:else if detail}
        <Tabs.Content value="overview" class="min-h-0 flex-1 overflow-hidden">
          <div class="fullpage-body">
            <ScrollArea class="h-full">
              <Accordion type="single" class="w-full max-w-[1200px] mx-auto">
                <AccordionItem value="seo-meta">
                  <AccordionHeader>
                    <AccordionTrigger>{m["detail.seo_meta"]()}</AccordionTrigger>
                  </AccordionHeader>
                  <AccordionContent>
                    {@render seoMetaContent()}
                  </AccordionContent>
                </AccordionItem>

                <AccordionItem value="crawl-info">
                  <AccordionHeader>
                    <AccordionTrigger>{m["detail.crawl_info"]()}</AccordionTrigger>
                  </AccordionHeader>
                  <AccordionContent>
                    {@render crawlInfoContent()}
                  </AccordionContent>
                </AccordionItem>

                {#if parseHreflang(detail.hreflang_json).length > 0}
                  <AccordionItem value="hreflang">
                    <AccordionHeader>
                      <AccordionTrigger>{m["detail.hreflang"]()}</AccordionTrigger>
                    </AccordionHeader>
                    <AccordionContent>
                      {@render hreflangContent()}
                    </AccordionContent>
                  </AccordionItem>
                {/if}

                {#if parseIssues(detail.semantic_issues_json).length > 0}
                  <AccordionItem value="semantic-issues" class="overview-full">
                    <AccordionHeader>
                      <AccordionTrigger>{m["detail.semantic_issues"]({ count: parseIssues(detail.semantic_issues_json).length.toString() })}</AccordionTrigger>
                    </AccordionHeader>
                    <AccordionContent>
                      {@render issuesContent()}
                    </AccordionContent>
                  </AccordionItem>
                {/if}
              </Accordion>
            </ScrollArea>
          </div>
        </Tabs.Content>

        <Tabs.Content value="links" class="min-h-0 flex-1 overflow-hidden">
          <div class="fullpage-body">
            {#if links.length > 0}
              <ScrollArea class="h-full">
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>{m["detail.links_type"]()}</TableHead>
                      <TableHead>{m["detail.links_url"]()}</TableHead>
                      <TableHead>{m["detail.links_anchor"]()}</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {#each links as link}
                      <TableRow>
                        <TableCell>
                          <Badge variant="secondary" class="text-xs">{link.link_type}</Badge>
                        </TableCell>
                        <TableCell>
                          <TooltipProvider>
                            <Tooltip>
                              <TooltipTrigger>
                                <a href={link.to_url} target="_blank" class="link-url">
                                  {truncateUrl(link.to_url)}
                                </a>
                              </TooltipTrigger>
                              <TooltipContent>
                                {link.to_url}
                              </TooltipContent>
                            </Tooltip>
                          </TooltipProvider>
                        </TableCell>
                        <TableCell>
                          <span class="link-anchor">{link.anchor_text || '-'}</span>
                        </TableCell>
                      </TableRow>
                    {/each}
                  </TableBody>
                </Table>
              </ScrollArea>
            {:else}
              <div class="empty-tab">{m["detail.no_links"]()}</div>
            {/if}
          </div>
        </Tabs.Content>

        <Tabs.Content value="preview" class="min-h-0 flex-1 overflow-hidden">
          <div class="fullpage-body">
            {#if previewLoading}
              <div class="flex items-center justify-center h-full">
                <Skeleton class="h-64 w-full" />
              </div>
            {:else if previewError}
              <Alert variant="destructive">
                {previewError}
              </Alert>
            {:else if previewHtml}
              <div class="preview-container">
                <div class="preview-iframe-wrap">
                  {#if overlays.length > 0}
                    <div class="overlay-legend">
                      {#each ['error', 'warning', 'info'] as sev}
                        {#if overlays.some((o) => o.severity === sev)}
                          <span class="overlay-legend-item">
                            <span class="overlay-dot ov-{sev}"></span>
                            {translateSeverity(sev)}
                          </span>
                        {/if}
                      {/each}
                      <span class="overlay-legend-count">{overlays.length}</span>
                    </div>
                  {/if}
                  {#each overlays as overlay}
                    <button
                      type="button"
                      class="overlay-marker ov-{overlay.severity}"
                      style="left: {overlay.x}px; top: {overlay.y}px"
                      title={overlay.label}
                      aria-label={overlay.label}
                      onclick={() => jumpToOverlay(overlay)}
                    ></button>
                  {/each}
                  <iframe
                    title="HTML Preview"
                    srcdoc={previewHtml}
                    sandbox="allow-same-origin"
                    class="preview-iframe"
                    bind:this={previewIframe}
                    onload={onPreviewLoad}
                  ></iframe>
                </div>
              </div>
            {:else}
              <div class="empty-tab">{m["detail.no_preview"]()}</div>
            {/if}
          </div>
        </Tabs.Content>
      {/if}
    </Tabs.Root>
  </div>
{/if}

{#snippet seoMetaContent()}
  <Card size="sm">
    <CardContent class="flex flex-col gap-2 pt-4">
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
{/snippet}

{#snippet crawlInfoContent()}
  <Card size="sm">
    <CardContent class="flex flex-col gap-2 pt-4">
      <div class="field-row stats-row">
        <div class="field">
          <span class="field-label">{m["detail.status"]()}</span>
          <Badge variant={statusBadgeVariant(detail.status_code)}>{detail.status_code}</Badge>
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
      {#if detail.load_time_ms !== undefined}
        <div class="flex items-center gap-2">
          <span class="field-label">{m["detail.load_time"]()}</span>
          <Progress
            value={Math.min(100, (detail.load_time_ms / 10000) * 100)}
            max={100}
            class="flex-1"
          />
        </div>
      {/if}
      {#if detail.parent_url}
        <div class="field">
          <span class="field-label">{m["detail.discovered_from"]()}</span>
          <a href={detail.parent_url} target="_blank" class="field-value">{detail.parent_url}</a>
        </div>
      {/if}
    </CardContent>
  </Card>
{/snippet}

{#snippet hreflangContent()}
  <Card size="sm">
    <CardContent class="pt-4">
      <div class="hreflang-list">
        {#each parseHreflang(detail.hreflang_json) as hl}
          <span class="hreflang-tag">{hl.lang}: {hl.href}</span>
        {/each}
      </div>
    </CardContent>
  </Card>
{/snippet}

{#snippet issuesContent()}
  <Card size="sm">
    <CardContent class="pt-4">
      <div class="issue-list">
        {#each parseIssues(detail.semantic_issues_json) as issue}
          {@const params = parseIssueParams(issue.message, issue.issue_type)}
          <div class="issue-card issue-{issue.severity}">
            <div class="issue-header">
              <Badge variant={severityBadgeVariant(issue.severity)}>{translateSeverity(issue.severity)}</Badge>
              <span class="issue-type">{translateIssueName(issue.issue_type)}</span>
            </div>
            <div class="issue-message">{translateIssueMessage(issue.issue_type, params)}</div>
            <div class="issue-details">
              {#if issue.element}
                <span class="issue-detail"><code>{issue.element}</code></span>
              {/if}
              {#if issue.xpath}
                <Popover>
                  <PopoverTrigger>
                    <Button
                      variant="outline"
                      size="sm"
                      class={cn(
                        'issue-detail issue-xpath copy-btn h-auto px-2.5 py-1.5',
                        copiedField === `xpath-${issue.xpath}` && 'copied'
                      )}
                    >
                      {#if copiedField === `xpath-${issue.xpath}`}
                        <Check class="size-3.5" />
                        <span class="copy-text">{m["detail.copied"]()}</span>
                      {:else}
                        <Copy class="size-3.5" />
                        <span class="copy-text">{m["detail.copy_xpath"]()}</span>
                      {/if}
                      <code class="copy-value">{issue.xpath}</code>
                    </Button>
                  </PopoverTrigger>
                  <PopoverContent class="w-80">
                    <div class="flex flex-col gap-2">
                      <span class="text-xs font-medium text-muted-foreground">{m["detail.copy_xpath"]()}</span>
                      <code class="text-xs font-mono break-all">{issue.xpath}</code>
                      <Button
                        variant="outline"
                        size="sm"
                        class="w-fit"
                        onclick={() => copyToClipboard(issue.xpath, `xpath-${issue.xpath}`)}
                      >
                        {#if copiedField === `xpath-${issue.xpath}`}
                          <Check class="size-3.5" />
                          {m["detail.copied"]()}
                        {:else}
                          <Copy class="size-3.5" />
                          {m["detail.copy_xpath"]()}
                        {/if}
                      </Button>
                    </div>
                  </PopoverContent>
                </Popover>
              {/if}
            </div>
          </div>
        {/each}
      </div>
    </CardContent>
  </Card>
{/snippet}

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
    padding: 24px;
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

  .stats-row {
    align-items: center;
  }

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
    align-items: center;
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

  /* Preview */
  .preview-container {
    display: flex;
    flex-direction: column;
    gap: 16px;
    max-width: 1200px;
    margin: 0 auto;
  }

  .preview-iframe-wrap {
    position: relative;
    border-radius: 8px;
    overflow: hidden;
    border: 1px solid var(--border);
  }

  .preview-iframe {
    width: 100%;
    border: none;
    background: white;
    display: block;
  }

  .overlay-marker {
    position: absolute;
    width: 16px;
    height: 16px;
    transform: translate(-50%, -50%);
    border-radius: 9999px;
    border: 2px solid var(--bg-card);
    box-shadow: 0 1px 4px rgba(0, 0, 0, 0.45);
    cursor: pointer;
    padding: 0;
    transition: transform 0.15s ease;
  }
  .overlay-marker:hover {
    transform: translate(-50%, -50%) scale(1.4);
    z-index: 5;
  }
  .overlay-marker.ov-error { background: var(--danger); }
  .overlay-marker.ov-warning { background: var(--warning); }
  .overlay-marker.ov-info { background: var(--info); }

  .overlay-legend {
    position: absolute;
    top: 10px;
    right: 10px;
    z-index: 6;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 10px;
    background: color-mix(in srgb, var(--bg-card) 88%, transparent);
    border: 1px solid var(--border);
    border-radius: 9999px;
    font-size: 0.72rem;
    color: var(--text-secondary);
    backdrop-filter: blur(4px);
  }
  .overlay-legend-item {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    text-transform: capitalize;
  }
  .overlay-legend-count {
    font-weight: 600;
    color: var(--text);
    padding-left: 4px;
    border-left: 1px solid var(--border);
  }
  .overlay-dot {
    width: 8px;
    height: 8px;
    border-radius: 9999px;
    display: inline-block;
  }
  .overlay-dot.ov-error { background: var(--danger); }
  .overlay-dot.ov-warning { background: var(--warning); }
  .overlay-dot.ov-info { background: var(--info); }

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

  @media (prefers-reduced-motion: reduce) {
    .fullpage {
      animation: none;
    }
  }
</style>