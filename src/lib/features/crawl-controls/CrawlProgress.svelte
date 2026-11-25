<script lang="ts">
  import { listen } from '@tauri-apps/api/event';
  import { m } from '#lib/paraglide/messages.js';
  import { Progress } from '#lib/components/ui/progress/index.js';
  import { Button } from '#lib/components/ui/button/index.js';
  import SeverityBadge from '#lib/components/SeverityBadge.svelte';
  import type { CrawlProgressState } from '#lib/app.svelte.ts';
  import type { CrawlProgress, FetchErrorKind } from '#lib/api/types.ts';

  let {
    progress,
    streamedCount,
    projectId,
    onStop,
  }: {
    progress: CrawlProgressState;
    streamedCount: number;
    /** Current project id, to ignore progress events of other projects. */
    projectId?: string;
    /** Optional stop handler (the page passes the same one as CrawlControls). */
    onStop?: () => void;
  } = $props();

  // Extended progress fields (T16) arrive on the same crawl-progress event
  // but are NOT copied into app.svelte.ts state (that file belongs to the
  // pending T9 push). This component listens for them itself and keeps
  // them local, so the progress bar never re-renders the results table
  // (UI_UX_PLAN §3.2). Folded into the global state in T16b.
  let errors4xx = $state(0);
  let errors5xx = $state(0);
  let errorsTimeout = $state(0);
  let currentDepth = $state(0);
  let elapsedSecs = $state(0);
  let prevQueued = $state(0);
  let discovering = $state(false);
  // Last failed fetch and the retry in flight (T11), same local listener
  // as the T16 fields (app.svelte.ts belongs to the pending T9 push).
  let lastError = $state<CrawlProgress['last_error']>(undefined);
  let retrying = $state<CrawlProgress['retrying']>(undefined);
  let retriesTotal = $state(0);

  $effect(() => {
    let unlisten: (() => void) | null = null;
    let disposed = false;
    listen<CrawlProgress>('crawl-progress', (event) => {
      const p = event.payload;
      if (projectId && p.project_id !== projectId) return;
      errors4xx = p.errors_4xx ?? 0;
      errors5xx = p.errors_5xx ?? 0;
      errorsTimeout = p.errors_timeout ?? 0;
      currentDepth = p.current_depth ?? 0;
      elapsedSecs = p.elapsed_secs ?? 0;
      lastError = p.last_error;
      retrying = p.retrying;
      retriesTotal = p.retries_total ?? 0;
      // The total is unknown while the crawler keeps discovering URLs:
      // the queue grows instead of shrinking. Show "discovering…" and
      // the pace instead of a false percentage (UI_UX_PLAN §1.3).
      discovering = p.urls_queued > prevQueued && p.urls_crawled > 0;
      prevQueued = p.urls_queued;
    }).then((u) => {
      if (disposed) u();
      else unlisten = u;
    });
    return () => {
      disposed = true;
      unlisten?.();
    };
  });

  const progressPct = $derived(
    progress.crawled > 0
      ? Math.min((progress.crawled / (progress.crawled + progress.queued)) * 100, 100)
      : 0
  );

  /** Crawled pages per second, from the event's elapsed time. */
  const rate = $derived(elapsedSecs > 0 ? progress.crawled / elapsedSecs : 0);

  /** Estimated remaining time from the queue and the current rate. */
  const etaSecs = $derived(
    rate > 0 && progress.queued > 0 ? Math.round(progress.queued / rate) : null
  );

  function formatDuration(totalSecs: number): string {
    const mins = Math.floor(totalSecs / 60);
    const secs = totalSecs % 60;
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  }

  /** Localized label for a fetch-error kind (the raw cause stays English). */
  function errorKindLabel(kind: FetchErrorKind): string {
    switch (kind) {
      case 'timeout':
        return m['progress.error_timeout']();
      case 'connection':
        return m['progress.error_connection']();
      case 'server_5xx':
        return m['progress.error_server_5xx']();
      case 'blocked':
        return m['progress.error_blocked']();
      default:
        return m['progress.error_other']();
    }
  }
</script>

{#if progress.crawled > 0 || progress.queued > 0}
  <section class="progress-section">
    <div class="progress-head">
      <h2>{m['progress.title']()}</h2>
      <span class="progress-pct">
        {#if discovering}
          {m['progress.discovering']()}
        {:else}
          {Math.round(progressPct)}%
        {/if}
      </span>
      {#if onStop}
        <Button variant="destructive" size="sm" onclick={onStop}>
          {m['progress.stop']()}
        </Button>
      {/if}
    </div>
    <Progress
      value={discovering ? undefined : progressPct}
      class="h-2 transition-all duration-300"
    />
    <div class="progress-stats">
      <span>{m['progress.crawled']({ count: progress.crawled.toString() })}</span>
      <span>{m['progress.queued']({ count: progress.queued.toString() })}</span>
      {#if rate > 0}
        <span>{m['progress.rate']({ rate: rate.toFixed(1) })}</span>
      {/if}
      {#if etaSecs !== null && !discovering}
        <span>{m['progress.eta']({ time: formatDuration(etaSecs) })}</span>
      {/if}
      <span>{m['progress.depth']({ depth: currentDepth.toString() })}</span>
      <span class="current-url">{progress.current || '...'}</span>
    </div>
    {#if progress.errors > 0 || errors4xx > 0 || errors5xx > 0 || errorsTimeout > 0 || retriesTotal > 0}
      <div class="progress-errors">
        <SeverityBadge
          severity="error"
          count={progress.errors}
          label={m['progress.errors_total']()}
        />
        {#if errors4xx > 0}
          <SeverityBadge severity="error" count={errors4xx} label={m['progress.errors_4xx']()} />
        {/if}
        {#if errors5xx > 0}
          <SeverityBadge severity="error" count={errors5xx} label={m['progress.errors_5xx']()} />
        {/if}
        {#if errorsTimeout > 0}
          <SeverityBadge
            severity="warning"
            count={errorsTimeout}
            label={m['progress.errors_timeout']()}
          />
        {/if}
        {#if retriesTotal > 0}
          <SeverityBadge severity="info" count={retriesTotal} label={m['progress.retries']()} />
        {/if}
      </div>
    {/if}
    {#if retrying}
      <div class="progress-retry" title={retrying.cause}>
        {m['progress.retrying']({
          attempt: retrying.attempt.toString(),
          max: retrying.max_attempts.toString(),
        })}
        <span class="retry-url">{retrying.url}</span>
        <span class="retry-cause">— {errorKindLabel(retrying.kind)}</span>
      </div>
    {:else if lastError}
      <div class="progress-last-error" title={lastError.cause}>
        {m['progress.last_error']()}
        <span class="retry-url">{lastError.url}</span>
        <span class="retry-cause">— {errorKindLabel(lastError.kind)}</span>
      </div>
    {/if}
    {#if streamedCount > 0}
      <div class="streamed-info">
        {m['progress.streamed']({ count: streamedCount.toString() })}
      </div>
    {/if}
  </section>
{/if}

<style>
  section {
    background: var(--bg-card);
    border: none;
    border-radius: var(--radius-xl);
    padding: var(--space-lg);
    box-shadow: var(--neu-raised-md);
  }

  .progress-section {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .progress-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }

  .progress-head h2 {
    margin-bottom: 0;
  }

  h2 {
    font-size: 1.15rem;
    margin-bottom: 16px;
    color: var(--text);
  }

  .progress-pct {
    font-size: 0.85rem;
    font-weight: 700;
    color: var(--accent);
    font-variant-numeric: tabular-nums;
  }

  .progress-stats {
    display: flex;
    flex-wrap: wrap;
    gap: 4px 24px;
    font-size: 0.9rem;
    color: var(--text-secondary);
    font-variant-numeric: tabular-nums;
  }

  .current-url {
    flex: 1;
    text-align: right;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    direction: rtl;
  }

  .progress-errors {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    align-items: center;
  }

  .progress-retry,
  .progress-last-error {
    font-size: 0.85rem;
    color: var(--text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .retry-url {
    direction: rtl;
    unicode-bidi: plaintext;
  }

  .retry-cause {
    color: var(--text-muted);
  }

  .streamed-info {
    font-size: 0.85rem;
    color: var(--success);
    font-style: italic;
  }

  @media (max-width: 767px) {
    .progress-stats {
      flex-wrap: wrap;
      gap: var(--space-sm);
    }

    section {
      padding: var(--section-padding);
      border-radius: var(--radius-lg);
    }
  }
</style>
