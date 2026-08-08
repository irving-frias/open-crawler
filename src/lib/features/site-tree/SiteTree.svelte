<script lang="ts">
  import { getSiteTreeFull } from '$lib/api/results';
  import type { SiteTreeFullNode as TreeNode } from '$lib/api/types';
  import { m } from '$lib/paraglide/messages.js';
  import { ChevronRight, ChevronDown, FileText, TriangleAlert, RefreshCw } from 'lucide-svelte';
  import { Badge } from '$lib/components/ui/badge/index.js';
  import { Button } from '$lib/components/ui/button/index.js';
  import { Skeleton } from '$lib/components/ui/skeleton/index.js';
  import * as Select from '$lib/components/ui/select/index.js';
  import { cn } from '$lib/utils.js';

  let { projectId }: { projectId: string } = $props();

  type NodeState = {
    node: TreeNode;
    children: NodeState[] | null;
    expanded: boolean;
  };

  type IssueFilter = 'all' | 'issues' | 'clean';

  let roots = $state<NodeState[]>([]);
  let loading = $state(false);
  let error = $state('');
  let issueFilter = $state<IssueFilter>('all');
  let treeSeq = 0;

  function toState(nodes: TreeNode[]): NodeState[] {
    return nodes.map((n) => ({
      node: n,
      children: n.children?.length ? toState(n.children) : null,
      expanded: false,
    }));
  }

  async function loadTree() {
    if (!projectId) return;
    const seq = ++treeSeq;
    loading = true;
    error = '';
    try {
      const data = await getSiteTreeFull(projectId);
      if (seq !== treeSeq) return;
      roots = toState(data);
    } catch (e) {
      if (seq === treeSeq) error = String(e);
    } finally {
      if (seq === treeSeq) loading = false;
    }
  }

  function toggleNode(ns: NodeState) {
    ns.expanded = !ns.expanded;
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

  const ROW_HEIGHT = 28;

  let listEl = $state<HTMLDivElement | null>(null);
  let scrollTop = $state(0);
  let viewportHeight = $state(0);

  const totalHeight = $derived(visibleNodes.length * ROW_HEIGHT);
  const startIndex = $derived(Math.max(0, Math.floor(scrollTop / ROW_HEIGHT) - 10));
  const endIndex = $derived(
    Math.min(visibleNodes.length, Math.ceil((scrollTop + viewportHeight) / ROW_HEIGHT) + 10)
  );
  const windowRows = $derived(visibleNodes.slice(startIndex, endIndex));

  function onScroll(e: Event) {
    scrollTop = (e.target as HTMLElement).scrollTop;
  }

  $effect(() => {
    const el = listEl;
    if (!el) return;
    viewportHeight = el.clientHeight;
    const ro = new ResizeObserver(() => {
      viewportHeight = el.clientHeight;
    });
    ro.observe(el);
    return () => ro.disconnect();
  });

  function statusVariant(code: number): 'default' | 'warning' | 'destructive' {
    if (code >= 400) return 'destructive';
    if (code >= 300) return 'warning';
    return 'default';
  }

  $effect(() => {
    loadTree();
  });
</script>

<div class="site-tree">
  <div class="tree-head">
    <div class="flex items-center gap-2 text-sm font-semibold">
      <FileText class="size-4" />
      {m['tree.title']()}
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
          {issueFilter === 'all'
            ? m['tree.filter_all']()
            : issueFilter === 'issues'
              ? m['tree.filter_issues']()
              : m['tree.filter_clean']()}
        </Select.Trigger>
        <Select.Content>
          <Select.Item value="all">{m['tree.filter_all']()}</Select.Item>
          <Select.Item value="issues">{m['tree.filter_issues']()}</Select.Item>
          <Select.Item value="clean">{m['tree.filter_clean']()}</Select.Item>
        </Select.Content>
      </Select.Root>
      <Button
        variant="ghost"
        size="icon"
        class="size-7"
        onclick={() => loadTree()}
        aria-label={m['config.refresh']()}
        title={m['config.refresh']()}
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
      {m['tree.error']()}: {error}
    </div>
  {:else if visibleNodes.length === 0}
    <div class="p-3 text-sm text-muted-foreground">{m['tree.empty']()}</div>
  {:else}
    <div class="tree-list" bind:this={listEl} onscroll={onScroll}>
      <div class="tree-spacer" style="height: {totalHeight}px">
        {#each windowRows as row, i (row.ns.node.url)}
          <div
            class="tree-row"
            style="--tree-depth: {row.level}; transform: translateY({(startIndex + i) *
              ROW_HEIGHT}px)"
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
                aria-label={row.ns.expanded ? m['tree.collapse']() : m['tree.expand']()}
              >
                {#if row.ns.expanded}
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
              <Badge
                variant="destructive"
                class="gap-1 shrink-0"
                title={`${row.ns.node.issue_count} issues`}
              >
                <TriangleAlert class="size-3" />
                {row.ns.node.issue_count}
              </Badge>
            {:else}
              <span class="tree-clean"></span>
            {/if}
          </div>
        {/each}
      </div>
    </div>
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
    position: relative;
    max-height: 520px;
    overflow-y: auto;
  }

  .tree-spacer {
    position: relative;
    width: 100%;
  }

  .tree-row {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    display: flex;
    align-items: center;
    gap: 6px;
    height: 28px;
    padding: 0 8px;
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
