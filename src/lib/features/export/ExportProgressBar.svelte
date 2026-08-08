<script lang="ts">
  import { m } from '$lib/paraglide/messages.js';
  import { Progress } from '$lib/components/ui/progress/index.js';
  import type { ExportProgress } from '$lib/app.svelte';

  let {
    exportProgress,
  }: {
    exportProgress: ExportProgress;
  } = $props();
</script>

{#if exportProgress.running || exportProgress.percent === 100}
  <div class="export-progress-bar" aria-live="polite">
    <div class="progress-head">
      <span class="export-title">{m['export.progress']()}</span>
      <span class="progress-pct">{Math.round(exportProgress.percent)}%</span>
    </div>
    <Progress value={exportProgress.percent} class="h-2 transition-all duration-300" />
    <div class="progress-stats">
      <span>
        {#if exportProgress.stage === 'pages'}
          {m['export.stage.pages']()}
        {:else if exportProgress.stage === 'issues'}
          {m['export.stage.issues']()}
        {:else if exportProgress.stage === 'links'}
          {m['export.stage.links']()}
        {:else}
          …
        {/if}
      </span>
    </div>
  </div>
{/if}

<style>
  .export-progress-bar {
    position: fixed;
    bottom: calc(16px + env(safe-area-inset-bottom, 0px));
    left: 50%;
    transform: translateX(-50%);
    z-index: 60;
    display: flex;
    flex-direction: column;
    gap: 8px;
    width: min(420px, calc(100vw - 32px));
    padding: 12px 16px;
    background: var(--bg-card);
    border: none;
    border-radius: 16px;
    box-shadow: var(--neu-float);
  }

  .export-title {
    font-size: 0.85rem;
    font-weight: 600;
    color: var(--text-primary);
  }

  .progress-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
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
</style>
