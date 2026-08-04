<script lang="ts">
  import { getDuplicateGroups } from '$lib/api/analytics';
  import type { DuplicateGroup } from '$lib/api/types';
  import { m } from '$lib/paraglide/messages.js';
  import { Copy, Check, RefreshCw, TriangleAlert, Files } from 'lucide-svelte';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card/index.js';
  import { Badge } from '$lib/components/ui/badge/index.js';
  import { Button } from '$lib/components/ui/button/index.js';
  import { Skeleton } from '$lib/components/ui/skeleton/index.js';
  import { cn } from '$lib/utils.js';

  let {
    projectId,
  }: {
    projectId: string;
  } = $props();

  let groups = $state<DuplicateGroup[]>([]);
  let loading = $state(false);
  let error = $state('');
  let groupsSeq = 0;

  async function loadGroups() {
    if (!projectId) return;
    const seq = ++groupsSeq;
    loading = true;
    error = '';
    try {
      const data = await getDuplicateGroups(projectId);
      if (seq !== groupsSeq) return;
      groups = data;
    } catch (e) {
      if (seq === groupsSeq) error = String(e);
    } finally {
      if (seq === groupsSeq) loading = false;
    }
  }

  $effect(() => {
    if (projectId) loadGroups();
    else groups = [];
  });

  let copiedUrl = $state('');
  async function copyUrl(url: string) {
    try {
      if (navigator.clipboard?.writeText) {
        await navigator.clipboard.writeText(url);
      } else {
        const ta = document.createElement('textarea');
        ta.value = url;
        document.body.appendChild(ta);
        ta.select();
        document.execCommand('copy');
        document.body.removeChild(ta);
      }
      copiedUrl = url;
      setTimeout(() => { if (copiedUrl === url) copiedUrl = ''; }, 1500);
    } catch {}
  }
</script>

<div class="duplicates">
  <div class="duplicates-head">
    <div class="flex items-center gap-2 text-sm font-semibold">
      <Files class="size-4" />
      {m["duplicates.title"]()}
      {#if groups.length > 0}
        <Badge variant="warning" class="ml-1">{groups.length}</Badge>
      {/if}
    </div>
    <div class="flex items-center gap-2">
      {#if groups.length > 0}
        <Button variant="outline" size="sm" class="gap-1.5" onclick={loadGroups}>
          <RefreshCw class={cn('size-3.5', loading && 'animate-spin')} />
          {m["config.refresh"]()}
        </Button>
      {/if}
    </div>
  </div>

  {#if loading && groups.length === 0}
    <div class="flex flex-col gap-2">
      <Skeleton class="h-20 w-full" />
      <Skeleton class="h-20 w-full" />
    </div>
  {:else if error}
    <div class="flex items-center gap-2 p-3 text-sm text-destructive">
      <TriangleAlert class="size-4" />
      {error}
    </div>
  {:else if groups.length === 0}
    <div class="p-4 text-sm text-muted-foreground">{m["duplicates.empty"]()}</div>
  {:else}
    <div class="groups-list">
      {#each groups as group (group.id)}
        <Card>
          <CardHeader class="pb-2">
            <CardTitle class="flex items-center gap-2 text-sm">
              <Badge variant="warning">{m["duplicates.group"]()}</Badge>
              <span class="text-muted-foreground font-normal">{group.size} {m["duplicates.pages"]()}</span>
            </CardTitle>
          </CardHeader>
          <CardContent>
            <ul class="dup-list">
              {#each group.urls as u (u.url)}
                <li class="dup-row">
                  {#if u.status_code != null}
                    <Badge variant="secondary" class="shrink-0">{u.status_code}</Badge>
                  {/if}
                  <div class="dup-meta">
                    <span class="dup-title">{u.title || u.url}</span>
                    <span class="dup-url" title={u.url}>{u.url}</span>
                  </div>
                  <Button
                    variant="ghost"
                    size="icon-xs"
                    class="shrink-0"
                    title={m["duplicates.copy"]()}
                    onclick={() => copyUrl(u.url)}
                  >
                    {#if copiedUrl === u.url}
                      <Check class="size-3.5 text-success" />
                    {:else}
                      <Copy class="size-3.5" />
                    {/if}
                  </Button>
                </li>
              {/each}
            </ul>
          </CardContent>
        </Card>
      {/each}
    </div>
  {/if}
</div>

<style>
  .duplicates {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .duplicates-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding: 4px 8px 0;
    flex-wrap: wrap;
  }

  .groups-list {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .dup-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
  }

  .dup-row {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 6px 0;
    border-bottom: 1px solid var(--border);
  }

  .dup-row:last-child {
    border-bottom: none;
  }

  .dup-meta {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .dup-title {
    font-size: 0.85rem;
    font-weight: 500;
    color: var(--text);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .dup-url {
    font-size: 0.78rem;
    color: var(--text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
