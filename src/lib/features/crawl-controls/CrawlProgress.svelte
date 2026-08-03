<script lang="ts">
  import { m } from '$lib/paraglide/messages.js';
  import { Progress } from '$lib/components/ui/progress/index.js';
  import type { CrawlProgressState } from '$lib/app.svelte';

  let {
    progress,
    streamedCount,
  }: {
    progress: CrawlProgressState;
    streamedCount: number;
  } = $props();

  const progressPct = $derived(
    progress.crawled > 0
      ? Math.min((progress.crawled / (progress.crawled + progress.queued)) * 100, 100)
      : 0
  );
</script>

{#if progress.crawled > 0 || progress.queued > 0}
  <section class="progress-section">
    <div class="progress-head">
      <h2>{m['progress.title']()}</h2>
      <span class="progress-pct">{Math.round(progressPct)}%</span>
    </div>
    <Progress
      value={progressPct}
      class="h-2 transition-all duration-300"
    />
    <div class="progress-stats">
      <span>{m['progress.crawled']({ count: progress.crawled.toString() })}</span>
      <span>{m['progress.queued']({ count: progress.queued.toString() })}</span>
      <span>{m['progress.errors']({ count: progress.errors.toString() })}</span>
      <span class="current-url">{progress.current || '...'}</span>
    </div>
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
    border: 1px solid var(--border);
    border-radius: var(--radius-xl);
    padding: var(--space-lg);
    box-shadow: var(--shadow-xs);
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
    gap: 24px;
    font-size: 0.9rem;
    color: var(--text-secondary);
  }

  .current-url {
    flex: 1;
    text-align: right;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    direction: rtl;
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
