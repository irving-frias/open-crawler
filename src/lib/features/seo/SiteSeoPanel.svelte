<script lang="ts">
  import { listen } from '@tauri-apps/api/event';
  import { RefreshCw, Loader2, Globe, Square, ChevronDown, Copy } from 'lucide-svelte';
  import { m } from '$lib/paraglide/messages.js';
  import { getSeoOverview, runSeoAuditAll, getSeoAuditStatus, stopSeoAudit } from '$lib/api/seo';
  import type { SeoOverview, SeoAuditProgress } from '$lib/api/types';
  import { localizeSeoCheck } from '$lib/seo-checks';
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

  let overview = $state<SeoOverview | null>(null);
  let loading = $state(false);
  let error = $state('');
  let overviewSeq = 0;

  let rerunning = $state(false);
  let rerunProgress = $state<SeoAuditProgress | null>(null);
  let progressUnlisten: (() => void) | null = null;

  const CATEGORY_LABELS: Record<string, () => string> = {
    meta: m['seo.category.meta'],
    technical: m['seo.category.technical'],
    social: m['seo.category.social'],
    accessibility: m['seo.category.accessibility'],
    performance: m['seo.category.performance'],
    ai_readability: m['seo.category.ai_readability'],
    sxo: m['seo.category.sxo'],
  };

  function categoryLabel(category: string): string {
    return CATEGORY_LABELS[category]?.() ?? category;
  }

  function severityVariant(severity: string): 'default' | 'warning' | 'destructive' {
    if (severity === 'error') return 'destructive';
    if (severity === 'warning') return 'warning';
    return 'default';
  }

  function scoreColor(score: number): string {
    if (score >= 80) return 'var(--success)';
    if (score >= 60) return 'var(--warning)';
    return 'var(--danger)';
  }

  $effect(() => {
    if (!projectId) {
      overview = null;
      error = '';
      rerunning = false;
      rerunProgress = null;
      return;
    }
    const seq = ++overviewSeq;
    rerunning = false;
    rerunProgress = null;
    loading = true;

    let disposed = false;
    const cleanupListeners = () => {
      if (progressUnlisten) {
        progressUnlisten();
        progressUnlisten = null;
      }
    };
    cleanupListeners();

    loadOverview(seq);
    checkRunningAudit(seq);

    (async () => {
      const unProgress = await listen<SeoAuditProgress>('seo-audit-progress', (event) => {
        if (event.payload.project_id !== projectId) return;
        rerunProgress = event.payload;
      });
      const unComplete = await listen<{ project_id: string; cancelled: boolean }>('seo-audit-complete', (event) => {
        if (event.payload.project_id !== projectId) return;
        rerunning = false;
        setTimeout(() => {
          rerunProgress = null;
        }, 800);
        loadOverview(++overviewSeq);
      });
      if (disposed) {
        unProgress();
        unComplete();
      } else {
        progressUnlisten = () => {
          unProgress();
          unComplete();
        };
      }
    })().catch((e) => console.error('[Seo] Failed to subscribe:', e));

    return () => {
      disposed = true;
      cleanupListeners();
    };
  });

  async function loadOverview(seq?: number) {
    const current = seq ?? ++overviewSeq;
    loading = true;
    error = '';
    try {
      const data = await getSeoOverview(projectId);
      if (current !== overviewSeq) return;
      overview = data;
    } catch (e) {
      if (current === overviewSeq) error = String(e);
    } finally {
      if (current === overviewSeq) loading = false;
    }
  }

  // Re-attach to an audit that is already running in another window or that
  // was started before this panel mounted, so the progress bar stays live.
  async function checkRunningAudit(seq: number) {
    try {
      const status = await getSeoAuditStatus(projectId);
      if (seq !== overviewSeq) return;
      if (status) {
        rerunning = true;
        rerunProgress = status;
      }
    } catch (e) {
      console.error('[Seo] Failed to check running audit:', e);
    }
  }

  async function onReauditAll() {
    if (rerunning) return;
    rerunning = true;
    rerunProgress = { project_id: projectId, processed: 0, total: 0, errors: 0, percent: 0 };
    try {
      await runSeoAuditAll(projectId);
    } catch (e) {
      error = String(e);
    } finally {
      rerunning = false;
      setTimeout(() => {
        rerunProgress = null;
      }, 1200);
      loadOverview();
    }
  }

  async function onCancelAudit() {
    try {
      await stopSeoAudit(projectId);
    } catch (e) {
      console.error('[Seo] Failed to stop audit:', e);
    }
  }

  let expandedIssues = $state<Set<string>>(new Set());

  function toggleIssue(id: string) {
    const next = new Set(expandedIssues);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    expandedIssues = next;
  }

  async function copyToClipboard(text: string) {
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
    } catch {
      // Clipboard unavailable
    }
  }
</script>

{#if loading && !overview}
  <div class="seo-site-loading">
    <Skeleton class="h-24 w-full" />
    <Skeleton class="h-24 w-full" />
    <Skeleton class="h-24 w-full" />
  </div>
{:else if error && !overview}
  <div class="seo-site-error">{error}</div>
{:else if overview}
  <div class="seo-site">
    <Card>
      <CardHeader class="flex-row items-center justify-between gap-2">
        <div class="flex items-center gap-2">
          <Globe class="size-4" />
          <CardTitle>{m['seo.site.title']()}</CardTitle>
        </div>
        <Button
          variant="outline"
          size="sm"
          class="gap-1.5"
          onclick={onReauditAll}
          disabled={rerunning}
        >
          {#if rerunning}
            <Loader2 class="size-3.5 animate-spin" />
          {:else}
            <RefreshCw class="size-3.5" />
          {/if}
          {m['seo.site.rerun_all']()}
        </Button>
      </CardHeader>
      <CardContent>
        <div class="seo-concepts">
          <h4 class="seo-concepts-title">{m['seo.concepts.title']()}</h4>
          <p class="seo-concepts-subtitle">{m['seo.concepts.subtitle']()}</p>
          <div class="seo-concepts-grid">
            <div class="seo-concept">
              <span class="seo-concept-name">SEO</span>
              <span class="seo-concept-desc">{m['seo.concepts.seo']()}</span>
            </div>
            <div class="seo-concept">
              <span class="seo-concept-name">AEO</span>
              <span class="seo-concept-desc">{m['seo.concepts.aeo']()}</span>
            </div>
            <div class="seo-concept">
              <span class="seo-concept-name">GEO</span>
              <span class="seo-concept-desc">{m['seo.concepts.geo']()}</span>
            </div>
            <div class="seo-concept">
              <span class="seo-concept-name">SXO</span>
              <span class="seo-concept-desc">{m['seo.concepts.sxo']()}</span>
            </div>
            <div class="seo-concept">
              <span class="seo-concept-name">AIO</span>
              <span class="seo-concept-desc">{m['seo.concepts.aio']()}</span>
            </div>
          </div>
        </div>

        {#if rerunning && rerunProgress}
          <div class="seo-site-progress">
            <div class="seo-site-progress-row">
              <span class="seo-site-progress-label">
                {m['seo.site.rerun_all_running']({
                  processed: rerunProgress.processed,
                  total: rerunProgress.total,
                })}
              </span>
              <div class="seo-site-progress-actions">
                {#if rerunProgress.errors > 0}
                  <span class="seo-site-progress-errors">
                    {m['seo.site.progress_errors']({ errors: rerunProgress.errors })}
                  </span>
                {/if}
                <Button
                  variant="ghost"
                  size="xs"
                  class="gap-1 text-muted-foreground"
                  onclick={onCancelAudit}
                >
                  <Square class="size-3" />
                  {m['seo.site.cancel']()}
                </Button>
              </div>
            </div>
            <Progress value={rerunProgress.percent} max={100} />
          </div>
        {/if}

        {#if overview.audited_pages === 0}
          <p class="seo-site-empty">{m['seo.site.empty']()}</p>
        {:else}
          <div class="seo-site-grid">
            <div class="seo-site-stat">
              <span class="seo-site-stat-label">{m['seo.site.average']()}</span>
              <span
                class="seo-site-stat-value"
                style="color: {scoreColor(overview.avg_score ?? 0)}"
              >
                {Math.round(overview.avg_score ?? 0)}
                {#if overview.avg_grade}<span class="seo-site-grade">· {overview.avg_grade}</span
                  >{/if}
              </span>
            </div>
            <div class="seo-site-stat">
              <span class="seo-site-stat-label">{m['seo.site.audited']()}</span>
              <span class="seo-site-stat-value">{overview.audited_pages.toLocaleString()}</span>
            </div>
            <div class="seo-site-stat">
              <span class="seo-site-stat-label">{m['seo.site.coverage']()}</span>
              <span class="seo-site-stat-value">
                {Math.round((overview.audited_pages / Math.max(1, overview.total_pages)) * 100)}%
              </span>
              <Progress
                value={(overview.audited_pages / Math.max(1, overview.total_pages)) * 100}
                max={100}
                class="h-1.5 mt-1"
              />
            </div>
          </div>

          {#if overview.category_averages.length > 0}
            <div class="seo-site-section">
              <h4 class="seo-site-title">{m['seo.site.categories']()}</h4>
              <div class="seo-site-cats">
                {#each overview.category_averages as cat (cat.category)}
                  <div class="seo-site-cat">
                    <div class="seo-site-cat-head">
                      <span class="seo-site-cat-name">{categoryLabel(cat.category)}</span>
                      <span class="seo-site-cat-score" style="color: {scoreColor(cat.avg_score)}">
                        {Math.round(cat.avg_score)}
                      </span>
                    </div>
                    <Progress value={cat.avg_score} max={100} class="h-1.5" />
                  </div>
                {/each}
              </div>
            </div>
          {/if}

          {#if overview.top_issues.length > 0}
            <div class="seo-site-section">
              <h4 class="seo-site-title">{m['seo.site.top_issues']()}</h4>
              <div class="seo-site-issues">
                {#each overview.top_issues as issue (issue.id)}
                  {@const localized = localizeSeoCheck(
                    issue.id,
                    issue.message,
                    issue.guidance,
                    issue.evidence
                  )}
                  {@const open = expandedIssues.has(issue.id)}
                  <div class="seo-site-issue">
                    <Badge variant={severityVariant(issue.severity)}>
                      {issue.occurrences}
                    </Badge>
                    <div class="seo-site-issue-body">
                      <span class="seo-site-issue-message">
                        {localized.message}
                        <span class="seo-site-issue-count">
                          · {m['seo.site.occurrences']({ count: issue.occurrences })}
                        </span>
                      </span>
                      <span class="seo-site-issue-guidance">{localized.guidance}</span>
                      <button
                        type="button"
                        class="seo-site-issue-toggle"
                        onclick={() => toggleIssue(issue.id)}
                        aria-expanded={open}
                      >
                        <ChevronDown
                          class={cn(
                            'size-3.5 transition-transform duration-200',
                            open && 'rotate-180'
                          )}
                        />
                        <span>{open ? m['seo.site.collapse']() : m['seo.site.expand']()}</span>
                      </button>
                      {#if open}
                        <div class="seo-site-issue-detail">
                          {#if localized.fix || localized.expected}
                            <div class="seo-fix-block">
                              {#if localized.fix}
                                <div class="seo-fix-line">
                                  <span class="seo-fix-label">{m['seo.fix']()}</span>
                                  <span class="seo-fix-text">{localized.fix}</span>
                                </div>
                              {/if}
                              {#if localized.expected}
                                <div class="seo-fix-line">
                                  <span class="seo-fix-label">{m['seo.expected']()}</span>
                                  <pre class="seo-expected-code"><code>{localized.expected}</code
                                    ></pre>
                                </div>
                              {/if}
                            </div>
                          {/if}
                          {#if issue.examples?.length}
                            <div class="seo-examples">
                              {#each issue.examples.slice(0, 3) as ex, i (i)}
                                <div class="seo-example">
                                  {#if ex.element}
                                    <code class="seo-example-element">{ex.element}</code>
                                  {/if}
                                  {#if ex.snippet}
                                    <pre class="seo-example-snippet"><code>{ex.snippet}</code></pre>
                                  {/if}
                                  {#if ex.xpath}
                                    <button
                                      type="button"
                                      class="seo-example-xpath"
                                      onclick={() => copyToClipboard(ex.xpath ?? '')}
                                    >
                                      <Copy class="size-3" />
                                      <code>{ex.xpath}</code>
                                    </button>
                                  {/if}
                                </div>
                              {/each}
                            </div>
                          {/if}
                        </div>
                      {/if}
                    </div>
                  </div>
                {/each}
              </div>
            </div>
          {:else}
            <p class="seo-site-empty">{m['seo.site.no_issues']()}</p>
          {/if}
        {/if}
      </CardContent>
    </Card>
  </div>
{/if}

<style>
  .seo-site-loading {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .seo-site-error {
    color: var(--danger);
    font-size: 0.85rem;
  }

  .seo-site {
    display: flex;
    flex-direction: column;
  }

  .seo-site-progress {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 10px 12px;
    margin-bottom: 14px;
    background: var(--bg-deep);
    border-radius: 10px;
  }

  .seo-concepts {
    margin-bottom: 18px;
  }

  .seo-concepts-title {
    font-size: 0.8rem;
    font-weight: 600;
    color: var(--text);
    margin: 0;
  }

  .seo-concepts-subtitle {
    font-size: 0.75rem;
    color: var(--text-muted);
    margin: 2px 0 12px;
  }

  .seo-concepts-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
    gap: 8px;
  }

  .seo-concept {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 10px 12px;
    background: var(--bg-deep);
    border-radius: 10px;
    box-shadow: var(--neu-pressed-sm);
  }

  .seo-concept-name {
    font-size: 0.82rem;
    font-weight: 700;
    color: var(--text);
  }

  .seo-concept-desc {
    font-size: 0.74rem;
    color: var(--text-secondary);
    line-height: 1.4;
  }

  .seo-site-progress-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }

  .seo-site-progress-label {
    font-size: 0.8rem;
    color: var(--text);
    font-variant-numeric: tabular-nums;
  }

  .seo-site-progress-errors {
    font-size: 0.72rem;
    color: var(--danger);
  }

  .seo-site-progress-actions {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .seo-site-empty {
    font-size: 0.85rem;
    color: var(--text-muted);
    margin: 0;
  }

  .seo-site-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
    gap: 12px;
    margin-bottom: 18px;
  }

  .seo-site-stat {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 14px 16px;
    background: var(--bg-deep);
    border-radius: 12px;
    box-shadow: var(--neu-pressed-sm);
  }

  .seo-site-stat-label {
    font-size: 0.72rem;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .seo-site-stat-value {
    font-size: 1.6rem;
    font-weight: 700;
    color: var(--text);
    font-variant-numeric: tabular-nums;
  }

  .seo-site-grade {
    font-size: 0.9rem;
    font-weight: 600;
    margin-left: 2px;
  }

  .seo-site-section {
    margin-top: 4px;
  }

  .seo-site-title {
    font-size: 0.8rem;
    font-weight: 600;
    color: var(--text);
    margin: 0 0 10px;
  }

  .seo-site-cats {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 12px;
  }

  .seo-site-cat {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .seo-site-cat-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }

  .seo-site-cat-name {
    font-size: 0.78rem;
    color: var(--text-secondary);
  }

  .seo-site-cat-score {
    font-size: 0.82rem;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
  }

  .seo-site-issues {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .seo-site-issue {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    padding: 10px 12px;
    background: var(--bg-deep);
    border-radius: 10px;
    box-shadow: var(--neu-pressed-sm);
  }

  .seo-site-issue-body {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .seo-site-issue-message {
    font-size: 0.82rem;
    font-weight: 500;
    color: var(--text);
  }

  .seo-site-issue-count {
    font-weight: 400;
    color: var(--text-muted);
  }

  .seo-site-issue-guidance {
    font-size: 0.75rem;
    color: var(--text-secondary);
    line-height: 1.4;
  }

  .seo-site-issue-toggle {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    margin-top: 4px;
    font-size: 0.7rem;
    font-weight: 600;
    color: var(--text-secondary);
    background: none;
    border: none;
    cursor: pointer;
    padding: 3px 6px;
    border-radius: 6px;
    width: fit-content;
    transition:
      color var(--transition-fast),
      background var(--transition-fast);
  }
  .seo-site-issue-toggle:hover {
    color: var(--accent);
    background: var(--bg-card);
  }

  .seo-site-issue-detail {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin-top: 8px;
    padding-top: 8px;
    border-top: 1px dashed var(--border, var(--bg-card));
  }

  .seo-fix-block {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 8px 10px;
    background: var(--bg-card);
    border-radius: 8px;
    border: 1px solid var(--border, var(--bg-deep));
  }

  .seo-fix-line {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .seo-fix-label {
    font-size: 0.66rem;
    text-transform: uppercase;
    font-weight: 700;
    color: var(--text-muted);
  }

  .seo-fix-text {
    font-size: 0.78rem;
    color: var(--text-secondary);
    line-height: 1.4;
  }

  .seo-expected-code {
    margin: 0;
    font-family: var(--font-mono);
    font-size: 0.72rem;
    color: var(--success);
    background: var(--bg-deep);
    padding: 6px 8px;
    border-radius: 6px;
    overflow-x: auto;
    white-space: pre-wrap;
    word-break: break-word;
  }

  .seo-examples {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .seo-example {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 6px 8px;
    background: var(--bg-card);
    border-radius: 8px;
    border: 1px solid var(--border, var(--bg-deep));
  }

  .seo-example-element {
    font-family: var(--font-mono);
    font-size: 0.72rem;
    color: var(--info);
  }

  .seo-example-snippet {
    margin: 0;
    font-family: var(--font-mono);
    font-size: 0.7rem;
    color: var(--text-secondary);
    background: var(--bg-deep);
    padding: 6px 8px;
    border-radius: 6px;
    overflow-x: auto;
    white-space: pre-wrap;
    word-break: break-word;
  }

  .seo-example-xpath {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    font-size: 0.68rem;
    color: var(--purple);
    background: none;
    border: none;
    cursor: pointer;
    padding: 2px 4px;
    border-radius: 4px;
    text-align: left;
    max-width: 100%;
  }
  .seo-example-xpath:hover {
    color: var(--text);
    background: var(--bg-deep);
  }
  .seo-example-xpath code {
    font-family: var(--font-mono);
    font-size: 0.68rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
