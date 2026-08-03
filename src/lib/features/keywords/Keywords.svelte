<script lang="ts">
  import { getProjectKeywords } from '$lib/api/analytics';
  import type { KeywordAggregate } from '$lib/api/types';
  import { m } from '$lib/paraglide/messages.js';
  import { RefreshCw, TriangleAlert, Search } from 'lucide-svelte';
  import { Card, CardContent } from '$lib/components/ui/card/index.js';
  import { Button } from '$lib/components/ui/button/index.js';
  import { Badge } from '$lib/components/ui/badge/index.js';
  import { Skeleton } from '$lib/components/ui/skeleton/index.js';
  import { cn } from '$lib/utils.js';

  let {
    projectId,
  }: {
    projectId: string;
  } = $props();

  let keywords = $state<KeywordAggregate[]>([]);
  let loading = $state(false);
  let error = $state('');

  async function loadKeywords() {
    if (!projectId) return;
    loading = true;
    error = '';
    try {
      keywords = await getProjectKeywords(projectId, 200);
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    if (projectId) loadKeywords();
    else keywords = [];
  });

  const maxCount = $derived(keywords.length > 0 ? keywords[0].count : 1);
</script>

<div class="keywords">
  <div class="keywords-head">
    <div class="flex items-center gap-2 text-sm font-semibold">
      <Search class="size-4" />
      {m["keywords.title"]()}
      {#if keywords.length > 0}
        <Badge variant="secondary" class="ml-1">{keywords.length}</Badge>
      {/if}
    </div>
    <Button
      variant="ghost"
      size="icon"
      class="size-7"
      onclick={loadKeywords}
      aria-label={m["config.refresh"]()}
      title={m["config.refresh"]()}
      disabled={loading}
    >
      <RefreshCw class={cn('size-3.5', loading && 'animate-spin')} />
    </Button>
  </div>

  {#if loading && keywords.length === 0}
    <div class="flex flex-col gap-2">
      <Skeleton class="h-6 w-full" />
      <Skeleton class="h-6 w-4/5" />
      <Skeleton class="h-6 w-3/5" />
    </div>
  {:else if error}
    <div class="flex items-center gap-2 p-3 text-sm text-destructive">
      <TriangleAlert class="size-4" />
      {error}
    </div>
  {:else if keywords.length === 0}
    <div class="p-4 text-sm text-muted-foreground">{m["keywords.empty"]()}</div>
  {:else}
    <div class="keywords-grid">
      {#each keywords as kw (kw.keyword)}
        <div class="kw-row">
          <span class="kw-name" title={kw.keyword}>{kw.keyword}</span>
          <div class="kw-bar-track">
            <div
              class="kw-bar"
              style="width: {Math.max(4, (kw.count / maxCount) * 100)}%"
            ></div>
          </div>
          <span class="kw-count">{kw.count}</span>
          <span class="kw-pages" title={m["keywords.pages"]()}>{kw.pages}</span>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .keywords {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .keywords-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4px 8px 0;
  }

  .keywords-grid {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .kw-row {
    display: grid;
    grid-template-columns: minmax(120px, 220px) 1fr 56px 48px;
    align-items: center;
    gap: 10px;
    padding: 6px 8px;
    border-radius: 6px;
    font-size: 0.85rem;
  }

  .kw-row:hover {
    background: var(--bg-hover);
  }

  .kw-name {
    color: var(--text);
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .kw-bar-track {
    height: 10px;
    background: var(--bg-hover);
    border-radius: 5px;
    overflow: hidden;
  }

  .kw-bar {
    height: 100%;
    background: var(--accent-gradient);
    border-radius: 5px;
    transition: width 0.3s ease;
  }

  .kw-count {
    text-align: right;
    font-variant-numeric: tabular-nums;
    color: var(--text-secondary);
  }

  .kw-pages {
    text-align: right;
    color: var(--text-muted);
    font-size: 0.78rem;
  }
</style>
