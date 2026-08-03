<script lang="ts">
  import { getSiteTree } from '$lib/api/results';
  import type { SiteTreeNode as TreeNode } from '$lib/api/types';
  import { m } from '$lib/paraglide/messages.js';
  import { ChevronRight, ChevronDown, FileText, Loader2, TriangleAlert, RefreshCw } from 'lucide-svelte';
  import { Card, CardContent } from '$lib/components/ui/card/index.js';
  import { Badge } from '$lib/components/ui/badge/index.js';
  import { Button } from '$lib/components/ui/button/index.js';
  import { Skeleton } from '$lib/components/ui/skeleton/index.js';
  import * as Select from '$lib/components/ui/select/index.js';
  import { cn } from '$lib/utils.js';

  let { projectId }: { projectId: string } = $props();

  type TreeNode = {
    url: string;
    title: string | null;
    status_code: number | null;
    depth: number;
    has_children: boolean;
    issue_count: number;
  };

  type NodeState = {
    node: TreeNode;
    children: NodeState[] | null;
    loading: boolean;
    expanded: boolean;
  };

  type IssueFilter = 'all' | 'issues' | 'clean';

  let roots = $state<NodeState[]>([]);
  let loading = $state(false);
  let error = $state('');
  let issueFilter = $state<IssueFilter>('all');

  async function loadRoots() {
    if (!projectId) return;
    loading = true;
    error = '';
    try {
      const data = await getSiteTree(projectId, null, 200);
      roots = data.map(n => ({ node: n, children: null, loading: false, expanded: false }));
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  async function toggleNode(ns: NodeState) {
    ns.expanded = !ns.expanded;
    if (ns.expanded && ns.children === null && ns.node.has_children) {
      ns.loading = true;
      try {
        const data = await getSiteTree(projectId, ns.node.url, 200);
        ns.children = data.map(n => ({ node: n, children: null, loading: false, expanded: false }));
      } catch (e) {
        error = String(e);
      } finally {
        ns.loading = false;
      }
    }
  }

  const visibleNodes = $derived.by(() => {
    const out: { ns: NodeState; level: number }[] = [];
    const seen = new Set<string>();
    function walk(list: NodeState[], level: number) {
      for (const ns of list) {
        const matches =
          issueFilter === 'all' ||
          (issueFilter === 'issues' && ns.node.issue_count > 0) ||
          (issueFilter === 'clean' && ns.node.issue_count === 0);
        if (!matches) continue;
        if (seen.has(ns.node.url)) continue;
        seen.add(ns.node.url);
        out.push({ ns, level });
        if (ns.expanded && ns.children) {
          walk(ns.children, level + 1);
        }
      }
    }
    walk(roots, 0);
    return out;
  });

  function statusVariant(code: number): 'default' | 'warning' | 'destructive' {
    if (code >= 400) return 'destructive';
    if (code >= 300) return 'warning';
    return 'default';
  }

  $effect(() => {
    loadRoots();
  });
</script>

<div class="site-tree">
  <div class="tree-head">
    <div class="flex items-center gap-2 text-sm font-semibold">
      <FileText class="size-4" />
      {m["tree.title"]()}
    </div>
    <div class="tree-tools">
      <Select.Root
        type="single"
        value={issueFilter}
        onValueChange={(v) => {
          if (v) issueFilter = v as IssueFilter;
        }}
      >
        <Select.Trigger class="h-7 w-40 justify-between text-xs">
          {issueFilter === 'all' ? m["tree.filter_all"]() : issueFilter === 'issues' ? m["tree.filter_issues"]() : m["tree.filter_clean"]()}
        </Select.Trigger>
        <Select.Content>
          <Select.Item value="all">{m["tree.filter_all"]()}</Select.Item>
          <Select.Item value="issues">{m["tree.filter_issues"]()}</Select.Item>
          <Select.Item value="clean">{m["tree.filter_clean"]()}</Select.Item>
        </Select.Content>
      </Select.Root>
      <Button
        variant="ghost"
        size="icon"
        class="size-7"
        onclick={() => loadRoots()}
        aria-label={m["config.refresh"]()}
        title={m["config.refresh"]()}
        disabled={loading}
      >
        <RefreshCw class={cn('size-3.5', loading && 'animate-spin')} />
      </Button>
    </div>
  </div>

  {#if loading && roots.length === 0}
    <div class="flex flex-col gap-2 p-3">
      <Skeleton class="h-6 w-full" />
      <Skeleton class="h-6 w-4/5" />
      <Skeleton class="h-6 w-3/5" />
    </div>
  {:else if error}
    <div class="flex items-center gap-2 p-3 text-sm text-destructive">
      <TriangleAlert class="size-4" />
      {m["tree.error"]()}: {error}
    </div>
  {:else if visibleNodes.length === 0}
    <div class="p-3 text-sm text-muted-foreground">{m["tree.empty"]()}</div>
  {:else}
    <ul class="tree-list">
      {#each visibleNodes as row (row.ns.node.url)}
        <li
          class="tree-row"
          style="--tree-depth: {row.level}"
          class:expanded={row.ns.expanded}
          class:leaf={!row.ns.node.has_children}
        >
          {#if row.ns.node.has_children}
            <Button
              variant="ghost"
              size="icon"
              class="tree-toggle size-5"
              onclick={() => toggleNode(row.ns)}
              aria-expanded={row.ns.expanded}
              aria-label={row.ns.expanded ? m["tree.collapse"]() : m["tree.expand"]()}
            >
              {#if row.ns.loading}
                <Loader2 class="size-3.5 animate-spin" />
              {:else if row.ns.expanded}
                <ChevronDown class="size-3.5" />
              {:else}
                <ChevronRight class="size-3.5" />
              {/if}
            </Button>
          {:else}
            <span class="tree-dot"></span>
          {/if}

          {#if row.ns.node.status_code != null}
            <Badge variant={statusVariant(row.ns.node.status_code)} class="tree-status">
              {row.ns.node.status_code}
            </Badge>
          {/if}

          <span class="tree-title">{row.ns.node.title || row.ns.node.url}</span>
          <span class="tree-url" title={row.ns.node.url}>{row.ns.node.url}</span>

          {#if row.ns.node.issue_count > 0}
            <Badge variant="destructive" class="gap-1 shrink-0" title={`${row.ns.node.issue_count} issues`}>
              <TriangleAlert class="size-3" />
              {row.ns.node.issue_count}
            </Badge>
          {:else}
            <span class="tree-clean"></span>
          {/if}
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .site-tree {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .tree-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding: 4px 8px 0;
    flex-wrap: wrap;
  }

  .tree-tools {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .tree-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    max-height: 520px;
    overflow-y: auto;
  }

  .tree-row {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 8px;
    padding-left: calc(8px + var(--tree-depth) * 18px);
    border-radius: 6px;
    font-size: 0.85rem;
  }

  .tree-row:hover {
    background: var(--bg-hover, rgba(128, 128, 128, 0.08));
  }

  .tree-dot {
    width: 20px;
    height: 20px;
    flex-shrink: 0;
    display: inline-flex;
    align-items: center;
    justify-content: center;
  }

  .tree-dot::before {
    content: '';
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: var(--border);
  }

  .tree-title {
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 32ch;
  }

  .tree-url {
    color: var(--text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 40ch;
    flex: 1;
  }

  .tree-clean {
    flex-shrink: 0;
    width: 4px;
  }
</style>
