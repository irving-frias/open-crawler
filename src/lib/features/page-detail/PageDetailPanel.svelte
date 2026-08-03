<script lang="ts">
  import { getPageDetail, getPagespeedScore } from '$lib/api';
  import type { PageLink, PageSpeedData } from '$lib/api/types';
  import { m } from '$lib/paraglide/messages.js';
  import { translateIssueName, translateIssueMessage, parseIssueParams, translateSeverity } from '$lib/i18n-issues';
  import { ArrowLeft, X, Copy, Check, Gauge, Loader2, RotateCw, Share2 } from 'lucide-svelte';
  import * as Tabs from '$lib/components/ui/tabs/index.js';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card/index.js';
  import { Button } from '$lib/components/ui/button/index.js';
  import { Skeleton } from '$lib/components/ui/skeleton/index.js';
  import { Badge } from '$lib/components/ui/badge/index.js';
  import { ScrollArea } from '$lib/components/ui/scroll-area/index.js';
  import { Separator } from '$lib/components/ui/separator/index.js';
  import { Tooltip, TooltipTrigger, TooltipContent, TooltipProvider } from '$lib/components/ui/tooltip/index.js';
  import { Accordion, AccordionItem, AccordionHeader, AccordionTrigger, AccordionContent } from '$lib/components/ui/accordion/index.js';
  import { Alert } from '$lib/components/ui/alert/index.js';
  import { Popover, PopoverTrigger, PopoverContent } from '$lib/components/ui/popover/index.js';
  import { Progress } from '$lib/components/ui/progress/index.js';
  import { cn } from '$lib/utils.js';
  import LinksSection from './sections/LinksSection.svelte';

  let {
    pageId = $bindable(''),
    onClose,
  }: {
    pageId: string;
    onClose: () => void;
  } = $props();

  let detail = $state<any>(null);
  let links = $state<PageLink[]>([]);
  let loading = $state(false);
  let error = $state('');
  let activeTab = $state<'overview' | 'links' | 'preview'>('overview');
  let pagespeed = $state<PageSpeedData | null>(null);
  let pagespeedLoading = $state(false);
  let pagespeedError = $state('');

  $effect(() => {
    if (pageId) loadDetail();
    else { detail = null; links = []; activeTab = 'overview'; pagespeed = null; pagespeedError = ''; }
  });

  async function loadDetail() {
    loading = true;
    error = '';
    activeTab = 'overview';
    pagespeed = null;
    pagespeedError = '';
    try {
      const result = await getPageDetail(pageId);
      detail = result.page;
      links = result.links;
      pagespeed = parsePagespeed(detail?.pagespeed_json);
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  function parsePagespeed(json: string | null): any | null {
    if (!json) return null;
    try { return JSON.parse(json); } catch { return null; }
  }

  async function runPagespeedAudit() {
    if (!detail?.url) return;
    pagespeedLoading = true;
    pagespeedError = '';
    try {
      pagespeed = await getPagespeedScore(detail.project_id, detail.url);
      if (pagespeed?.score != null) {
        detail.pagespeed_score = pagespeed.score;
        detail.pagespeed_json = JSON.stringify(pagespeed);
      }
    } catch (e) {
      pagespeedError = String(e);
    } finally {
      pagespeedLoading = false;
    }
  }

  function scoreColor(score: number): string {
    if (score >= 90) return 'var(--success)';
    if (score >= 50) return 'var(--warning)';
    return 'var(--danger)';
  }

  function readabilityLabel(score: number): string {
    if (score >= 70) return m["dashboard.readability.easy"]();
    if (score >= 40) return m["dashboard.readability.medium"]();
    return m["dashboard.readability.hard"]();
  }

  function readabilityColor(score: number): string {
    if (score >= 70) return 'var(--success)';
    if (score >= 40) return 'var(--warning)';
    return 'var(--danger)';
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

  function parseOg(json: string | null): any {
    if (!json) return {};
    try { return JSON.parse(json); } catch { return {}; }
  }

  function ogFields(og: any): { key: string; value: string }[] {
    const out: { key: string; value: string }[] = [];
    const mapping: [string, string][] = [
      ['og:title', og.og_title],
      ['og:description', og.og_description],
      ['og:image', og.og_image],
      ['og:image:alt', og.og_image_alt],
      ['og:type', og.og_type],
      ['og:url', og.og_url],
      ['og:site_name', og.og_site_name],
      ['twitter:card', og.twitter_card],
      ['twitter:title', og.twitter_title],
      ['twitter:description', og.twitter_description],
      ['twitter:image', og.twitter_image],
    ];
    for (const [key, value] of mapping) {
      if (value) out.push({ key, value: String(value) });
    }
    return out;
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

                <AccordionItem value="pagespeed" class="overview-full">
                  <AccordionHeader>
                    <AccordionTrigger>
                      <span class="inline-flex items-center gap-2">
                        <Gauge class="size-4" />
                        {m["detail.pagespeed"]()}
                        {#if pagespeed?.score != null}
                          <span class="pagespeed-chip" style="color: {scoreColor(pagespeed.score)}">{pagespeed.score}</span>
                        {/if}
                      </span>
                    </AccordionTrigger>
                  </AccordionHeader>
                  <AccordionContent>
                    {@render pagespeedContent()}
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

                {@const og = parseOg(detail.og_json)}
                {#if ogFields(og).length > 0}
                  <AccordionItem value="social">
                    <AccordionHeader>
                      <AccordionTrigger>
                        <span class="inline-flex items-center gap-2">
                          <Share2 class="size-4" />
                          {m["detail.social"]()}
                        </span>
                      </AccordionTrigger>
                    </AccordionHeader>
                    <AccordionContent>
                      {@render socialContent()}
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
          <LinksSection links={links} />
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
      {#if detail.readability_score != null}
        <div class="flex items-center gap-2">
          <span class="field-label">{m["dashboard.readability.label"]()}</span>
          <Progress
            value={detail.readability_score}
            max={100}
            class="flex-1"
          />
          <span class="readability-value" style="color: {readabilityColor(detail.readability_score)}">
            {Math.round(detail.readability_score)} · {readabilityLabel(detail.readability_score)}
          </span>
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

{#snippet socialContent()}
  {@const og = parseOg(detail.og_json)}
  <Card size="sm">
    <CardContent class="flex flex-col gap-2 pt-4">
      {#if og.og_image}
        <div class="og-image">
          <img src={og.og_image} alt={og.og_image_alt || ''} loading="lazy" />
        </div>
      {/if}
      {#each ogFields(og) as field (field.key)}
        <div class="field">
          <span class="field-label">{field.key}</span>
          {#if field.key === 'og:image' || field.key === 'og:url' || field.key === 'twitter:image'}
            <a href={field.value} target="_blank" rel="noreferrer" class="field-value">{field.value}</a>
          {:else}
            <span class="field-value">{field.value}</span>
          {/if}
        </div>
      {/each}
    </CardContent>
  </Card>
{/snippet}

{#snippet pagespeedContent()}
  <Card size="sm">
    <CardContent class="flex flex-col gap-3 pt-4">
      {#if pagespeedLoading}
        <div class="flex items-center gap-2 text-sm text-muted-foreground">
          <Loader2 class="size-4 animate-spin" />
          {m["detail.pagespeed_running"]()}
        </div>
      {:else if pagespeedError}
        <Alert variant="destructive">{pagespeedError}</Alert>
        <div>
          <Button variant="outline" size="sm" class="gap-1.5" onclick={runPagespeedAudit}>
            <RotateCw class="size-3.5" />
            {m["detail.pagespeed_run"]()}
          </Button>
        </div>
      {:else if pagespeed?.score != null}
        <div class="pagespeed-row">
          <div
            class="pagespeed-score"
            style="--score-color: {scoreColor(pagespeed.score)}"
          >
            <span class="pagespeed-score-value">{pagespeed.score}</span>
            <span class="pagespeed-score-label">{m["detail.pagespeed_performance"]()}</span>
          </div>
          <div class="pagespeed-metrics">
            {#if pagespeed.fcp}
              <div class="field">
                <span class="field-label">{m["detail.pagespeed_fcp"]()}</span>
                <span class="field-value">{pagespeed.fcp}</span>
              </div>
            {/if}
            {#if pagespeed.lcp}
              <div class="field">
                <span class="field-label">{m["detail.pagespeed_lcp"]()}</span>
                <span class="field-value">{pagespeed.lcp}</span>
              </div>
            {/if}
            {#if pagespeed.cls}
              <div class="field">
                <span class="field-label">{m["detail.pagespeed_cls"]()}</span>
                <span class="field-value">{pagespeed.cls}</span>
              </div>
            {/if}
            {#if pagespeed.tbt}
              <div class="field">
                <span class="field-label">{m["detail.pagespeed_tbt"]()}</span>
                <span class="field-value">{pagespeed.tbt}</span>
              </div>
            {/if}
            {#if pagespeed.speed_index}
              <div class="field">
                <span class="field-label">{m["detail.pagespeed_speed_index"]()}</span>
                <span class="field-value">{pagespeed.speed_index}</span>
              </div>
            {/if}
          </div>
        </div>
        <div class="flex items-center gap-2">
          <Button variant="outline" size="sm" class="gap-1.5" onclick={runPagespeedAudit} disabled={pagespeedLoading}>
            <RotateCw class="size-3.5" />
            {m["detail.pagespeed_rerun"]()}
          </Button>
        </div>
      {:else}
        <p class="text-sm text-muted-foreground">{m["detail.pagespeed_empty"]()}</p>
        <div>
          <Button variant="outline" size="sm" class="gap-1.5" onclick={runPagespeedAudit} disabled={pagespeedLoading}>
            <Gauge class="size-3.5" />
            {m["detail.pagespeed_run"]()}
          </Button>
        </div>
      {/if}
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
                      onclick={() => copyToClipboard(issue.xpath, `xpath-${issue.xpath}`)}
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

  .pagespeed-chip {
    font-weight: 700;
    font-size: 0.85rem;
  }

  .pagespeed-row {
    display: flex;
    gap: 24px;
    align-items: center;
    flex-wrap: wrap;
  }

  .pagespeed-score {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 2px;
    width: 120px;
    height: 120px;
    border-radius: 9999px;
    border: 6px solid var(--score-color);
    flex-shrink: 0;
  }

  .pagespeed-score-value {
    font-size: 2rem;
    font-weight: 800;
    color: var(--score-color);
    font-variant-numeric: tabular-nums;
    line-height: 1;
  }

  .pagespeed-score-label {
    font-size: 0.68rem;
    text-transform: uppercase;
    color: var(--text-muted);
    font-weight: 600;
  }

  .pagespeed-metrics {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));
    gap: 16px;
    flex: 1;
    min-width: 280px;
  }

  .readability-value {
    font-size: 0.8rem;
    font-weight: 600;
    white-space: nowrap;
    font-variant-numeric: tabular-nums;
  }

  .og-image {
    max-width: 320px;
    border-radius: 12px;
    overflow: hidden;
    border: none;
    box-shadow: var(--neu-raised-sm);
  }

  .og-image img {
    display: block;
    width: 100%;
    height: auto;
  }

  .issue-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .issue-card {
    padding: 12px 14px;
    background: var(--bg-deep);
    border-radius: 12px;
    border-left: 3px solid transparent;
    box-shadow: var(--neu-pressed-sm);
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