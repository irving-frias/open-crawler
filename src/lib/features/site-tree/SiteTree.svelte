<script lang="ts">
  import { getSiteTreeFull } from '$lib/api/results';
  import type { SiteTreeFullNode as TreeNode } from '$lib/api/types';
  import { m } from '$lib/paraglide/messages.js';
  import {
    ChevronRight,
    ChevronDown,
    Expand,
    FileText,
    Folder,
    Languages,
    RefreshCw,
    Search,
    Shrink,
    TriangleAlert,
    X,
  } from '@lucide/svelte';
  import { Badge } from '$lib/components/ui/badge/index.js';
  import { Button } from '$lib/components/ui/button/index.js';
  import { Input } from '$lib/components/ui/input/index.js';
  import { Skeleton } from '$lib/components/ui/skeleton/index.js';
  import * as Select from '$lib/components/ui/select/index.js';
  import { cn } from '$lib/utils.js';
  import {
    LANGUAGE_CODES,
    legendColor,
    segmentsOf,
    statusClass,
    statusLabel,
    statusVariant,
    type StatusClass,
  } from '$lib/features/site-map/shared.js';
  import { siteMapFilters, type StatusFilter } from '$lib/features/site-map/filters.svelte.js';

  let { projectId }: { projectId: string } = $props();

  type IssueFilter = 'all' | 'issues' | 'clean';

  // One directory in the URL-path hierarchy. `pages` holds the real crawled
  // pages that live exactly at this path; `dirs` are its sub-directories. A
  // directory with no pages is a virtual folder used purely for grouping.
  // `isLang` marks a top-level language group (e.g. `/en`, `/es`).
  type DirState = {
    key: string;
    name: string;
    pages: TreeNode[];
    dirs: DirState[];
    expanded: boolean;
    isLang: boolean;
  };

  type Row =
    | { kind: 'page'; page: TreeNode; dir: DirState | null; depth: number; hasChildren: boolean }
    | { kind: 'folder'; dir: DirState; depth: number; hasChildren: boolean };

  type Agg = { count: number; issues: number; worst: StatusClass };

  const STATUS_RANK: Record<StatusClass, number> = {
    '5xx': 5,
    '4xx': 4,
    '3xx': 3,
    blocked: 2,
    unknown: 1,
    '2xx': 0,
  };

  let roots = $state<DirState[]>([]);
  let rootPages = $state<TreeNode[]>([]);
  let rootExpanded = $state(true);
  let loading = $state(false);
  let error = $state('');
  let issueFilter = $state<IssueFilter>('all');
  let selectedPage = $state<TreeNode | null>(null);
  let flashUrl = $state<string | null>(null);
  let treeSeq = 0;

  // Flattens the link-based tree returned by the API, then re-assembles it as a
  // hierarchy grouped by URL path. Path prefixes without a crawled page become
  // virtual folders, e.g. `/page-1/page` nests under `/page-1`. A leading
  // language code (`/en`, `/es`, ...) becomes its own top-level group so that
  // translated pages are separated from the default-language content.
  function buildTree(nodes: TreeNode[]): { roots: DirState[]; pages: TreeNode[] } {
    const flat: TreeNode[] = [];
    const collect = (list: TreeNode[]) => {
      for (const n of list) {
        flat.push(n);
        if (n.children?.length) collect(n.children);
      }
    };
    collect(nodes);

    const root: DirState = {
      key: '',
      name: '',
      pages: [],
      dirs: [],
      expanded: true,
      isLang: false,
    };
    const byKey = new Map<string, DirState>([['', root]]);

    const items = flat.map((page) => ({ page, segs: segmentsOf(page.url) }));
    items.sort((a, b) => a.segs.length - b.segs.length || a.page.url.localeCompare(b.page.url));

    for (const { page, segs } of items) {
      let cur = root;
      let key = '';
      let rest = segs;
      if (segs.length > 0 && LANGUAGE_CODES.has(segs[0])) {
        key = '/' + segs[0];
        let langDir = byKey.get(key);
        if (!langDir) {
          langDir = { key, name: segs[0], pages: [], dirs: [], expanded: true, isLang: true };
          byKey.set(key, langDir);
          root.dirs.push(langDir);
        }
        cur = langDir;
        rest = segs.slice(1);
      }
      for (const seg of rest) {
        key = key ? key + '/' + seg : '/' + seg;
        let d = byKey.get(key);
        if (!d) {
          const segCount = key.split('/').filter(Boolean).length;
          d = { key, name: seg, pages: [], dirs: [], expanded: segCount <= 1, isLang: false };
          byKey.set(key, d);
          cur.dirs.push(d);
        }
        cur = d;
      }
      cur.pages.push(page);
    }

    const sortTree = (d: DirState) => {
      d.dirs.sort((a, b) => Number(b.isLang) - Number(a.isLang) || a.name.localeCompare(b.name));
      d.pages.sort((a, b) => a.url.localeCompare(b.url));
      for (const c of d.dirs) sortTree(c);
    };
    sortTree(root);
    return { roots: root.dirs, pages: root.pages };
  }

  async function loadTree() {
    if (!projectId) return;
    const seq = ++treeSeq;
    loading = true;
    error = '';
    try {
      const data = await getSiteTreeFull(projectId);
      if (seq !== treeSeq) return;
      const built = buildTree(data);
      roots = built.roots;
      rootPages = built.pages;
      rootExpanded = true;
    } catch (e) {
      if (seq === treeSeq) error = String(e);
    } finally {
      if (seq === treeSeq) loading = false;
    }
  }

  function toggleDir(d: DirState) {
    d.expanded = !d.expanded;
  }

  function rowExpanded(row: Row): boolean {
    if (row.kind === 'folder') return row.dir.expanded;
    return row.dir ? row.dir.expanded : rootExpanded;
  }

  function toggleRow(row: Row) {
    if (row.kind === 'folder') {
      toggleDir(row.dir);
      return;
    }
    if (row.dir) toggleDir(row.dir);
    else rootExpanded = !rootExpanded;
  }

  function setDirExpanded(d: DirState, expanded: boolean) {
    d.expanded = expanded;
    for (const c of d.dirs) setDirExpanded(c, expanded);
  }

  function expandAll() {
    rootExpanded = true;
    for (const c of roots) setDirExpanded(c, true);
  }

  function collapseAll() {
    for (const c of roots) setDirExpanded(c, false);
    rootExpanded = false;
  }

  const treeView = $derived.by(() => {
    const rows: Row[] = [];
    const seen = new Set<string>();
    const aggs = new Map<DirState, Agg>();

    const q = siteMapFilters.search.trim().toLowerCase();
    const matches = (page: TreeNode): boolean => {
      if (issueFilter === 'issues' && page.issue_count <= 0) return false;
      if (issueFilter === 'clean' && page.issue_count > 0) return false;
      if (
        siteMapFilters.status !== 'all' &&
        statusClass(page.status_code, null) !== siteMapFilters.status
      ) {
        return false;
      }
      if (q && !`${page.url} ${page.title ?? ''}`.toLowerCase().includes(q)) return false;
      return true;
    };
    const dirVisible = (d: DirState): boolean => d.pages.some(matches) || d.dirs.some(dirVisible);

    const computeAgg = (d: DirState): Agg => {
      const cached = aggs.get(d);
      if (cached) return cached;
      let count = 0;
      let issues = 0;
      let worst: StatusClass = '2xx';
      for (const p of d.pages) {
        if (!matches(p)) continue;
        count += 1;
        issues += p.issue_count;
        const sc = statusClass(p.status_code, null);
        if (STATUS_RANK[sc] > STATUS_RANK[worst]) worst = sc;
      }
      for (const c of d.dirs) {
        const sub = computeAgg(c);
        count += sub.count;
        issues += sub.issues;
        if (STATUS_RANK[sub.worst] > STATUS_RANK[worst]) worst = sub.worst;
      }
      const res: Agg = { count, issues, worst };
      aggs.set(d, res);
      return res;
    };

    const walk = (d: DirState, level: number) => {
      if (!dirVisible(d)) return;
      computeAgg(d);
      const pages = d.pages.filter(matches);
      const hasCh = d.dirs.some(dirVisible);
      pages.forEach((page, i) => {
        if (seen.has(page.url)) return;
        seen.add(page.url);
        rows.push({ kind: 'page', page, dir: d, depth: level, hasChildren: hasCh && i === 0 });
      });
      if (pages.length === 0 && hasCh) {
        rows.push({ kind: 'folder', dir: d, depth: level, hasChildren: true });
      }
      if (d.expanded && hasCh) {
        for (const c of d.dirs) walk(c, level + 1);
      }
    };

    const hasRootChildren = roots.some(dirVisible);
    if (rootPages.some(matches) || hasRootChildren) {
      rootPages.filter(matches).forEach((page, i) => {
        if (seen.has(page.url)) return;
        seen.add(page.url);
        rows.push({
          kind: 'page',
          page,
          dir: null,
          depth: 0,
          hasChildren: hasRootChildren && i === 0,
        });
      });
      if (rootExpanded && hasRootChildren) {
        for (const c of roots) walk(c, 1);
      }
    }

    return { rows, aggs };
  });

  function aggOf(d: DirState): Agg {
    return treeView.aggs.get(d) ?? { count: 0, issues: 0, worst: '2xx' };
  }

  function rowKey(row: Row): string {
    if (row.kind === 'folder') return 'folder:' + row.dir.key;
    return 'page:' + row.page.url;
  }

  const ROW_HEIGHT = 28;

  let listEl = $state<HTMLDivElement | null>(null);
  let scrollTop = $state(0);
  let viewportHeight = $state(0);

  const visibleNodes = $derived(treeView.rows);
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

  function selectPage(page: TreeNode) {
    selectedPage = page;
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
      <span class="text-xs font-normal text-muted-foreground">{m['tree.grouped_by_path']()}</span>
    </div>
    <div class="tree-tools">
      <div class="relative">
        <Search class="tree-search-icon size-3.5" />
        <Input
          type="text"
          class="tree-search h-7 w-44 pl-7 text-xs"
          placeholder={m['tree.search_placeholder']()}
          bind:value={siteMapFilters.search}
        />
      </div>
      <Select.Root
        type="single"
        value={siteMapFilters.status}
        onValueChange={(v) => {
          if (v) siteMapFilters.status = v as StatusFilter;
        }}
      >
        <Select.Trigger
          class="h-7 w-32 justify-between text-xs"
          aria-label={m['tree.filter_status']()}
        >
          {siteMapFilters.status === 'all'
            ? m['tree.filter_all']()
            : statusLabel(siteMapFilters.status)}
        </Select.Trigger>
        <Select.Content>
          <Select.Item value="all">{m['tree.filter_all']()}</Select.Item>
          <Select.Item value="2xx">{m['graph.filter_2xx']()}</Select.Item>
          <Select.Item value="3xx">{m['graph.filter_3xx']()}</Select.Item>
          <Select.Item value="4xx">{m['graph.filter_4xx']()}</Select.Item>
          <Select.Item value="5xx">{m['graph.filter_5xx']()}</Select.Item>
          <Select.Item value="unknown">{m['graph.filter_unknown']()}</Select.Item>
        </Select.Content>
      </Select.Root>
      <Select.Root
        type="single"
        value={issueFilter}
        onValueChange={(v) => {
          if (v) issueFilter = v as IssueFilter;
        }}
      >
        <Select.Trigger class="h-7 w-32 justify-between text-xs">
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
        onclick={expandAll}
        aria-label={m['tree.expand_all']()}
        title={m['tree.expand_all']()}
      >
        <Expand class="size-3.5" />
      </Button>
      <Button
        variant="ghost"
        size="icon"
        class="size-7"
        onclick={collapseAll}
        aria-label={m['tree.collapse_all']()}
        title={m['tree.collapse_all']()}
      >
        <Shrink class="size-3.5" />
      </Button>
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
    <div class="tree-body">
      <div class="tree-list" bind:this={listEl} onscroll={onScroll}>
        <div class="tree-spacer" style="height: {totalHeight}px">
          {#each windowRows as row, i (rowKey(row))}
            <div
              class="tree-row"
              class:leaf={!row.hasChildren}
              class:tree-row-clickable={row.kind === 'page'}
              class:tree-row-selected={row.kind === 'page' && selectedPage?.url === row.page.url}
              class:tree-row-flash={row.kind === 'page' && flashUrl === row.page.url}
              style="--tree-depth: {row.depth}; transform: translateY({(startIndex + i) *
                ROW_HEIGHT}px)"
              {...row.kind === 'page'
                ? {
                    role: 'button',
                    tabindex: 0,
                    onclick: () => selectPage(row.page),
                    onkeydown: (e: KeyboardEvent) => {
                      if (e.key === 'Enter' || e.key === ' ') {
                        e.preventDefault();
                        selectPage(row.page);
                      }
                    },
                  }
                : { tabindex: -1 }}
              title={row.kind === 'page' ? row.page.url : row.dir.key}
            >
              {#if row.kind === 'folder'}
                <Button
                  variant="ghost"
                  size="icon"
                  class="tree-toggle size-5"
                  onclick={(e) => {
                    e.stopPropagation();
                    toggleRow(row);
                  }}
                  aria-expanded={rowExpanded(row)}
                  aria-label={rowExpanded(row) ? m['tree.collapse']() : m['tree.expand']()}
                >
                  {#if rowExpanded(row)}
                    <ChevronDown class="size-3.5" />
                  {:else}
                    <ChevronRight class="size-3.5" />
                  {/if}
                </Button>
                {#if row.dir.isLang}
                  <Languages class="tree-lang size-4" />
                {:else}
                  <Folder class="tree-folder size-4" />
                {/if}
                <span class="tree-title">{row.dir.name}</span>
                <span class="tree-url" title={row.dir.key}>{row.dir.key}</span>
                <span
                  class="tree-status-dot"
                  style="background: {legendColor(aggOf(row.dir).worst)}"
                ></span>
                {#if aggOf(row.dir).issues > 0}
                  <Badge
                    variant="destructive"
                    class="gap-1 shrink-0"
                    title={`${aggOf(row.dir).issues} issues`}
                  >
                    <TriangleAlert class="size-3" />
                    {aggOf(row.dir).issues}
                  </Badge>
                {:else}
                  <span class="tree-clean"></span>
                {/if}
                {#if aggOf(row.dir).count > 0}
                  <span class="tree-agg" title={`${aggOf(row.dir).count} pages`}>
                    {aggOf(row.dir).count}
                  </span>
                {/if}
              {:else}
                {#if row.hasChildren}
                  <Button
                    variant="ghost"
                    size="icon"
                    class="tree-toggle size-5"
                    onclick={(e) => {
                      e.stopPropagation();
                      toggleRow(row);
                    }}
                    aria-expanded={rowExpanded(row)}
                    aria-label={rowExpanded(row) ? m['tree.collapse']() : m['tree.expand']()}
                  >
                    {#if rowExpanded(row)}
                      <ChevronDown class="size-3.5" />
                    {:else}
                      <ChevronRight class="size-3.5" />
                    {/if}
                  </Button>
                {:else}
                  <span class="tree-dot"></span>
                {/if}

                {#if row.page.status_code != null}
                  <Badge variant={statusVariant(row.page.status_code)} class="tree-status">
                    {row.page.status_code}
                  </Badge>
                {/if}

                <span class="tree-title">{row.page.title || row.page.url}</span>
                <span class="tree-url" title={row.page.url}>{row.page.url}</span>

                {#if row.page.issue_count > 0}
                  <Badge
                    variant="destructive"
                    class="gap-1 shrink-0"
                    title={`${row.page.issue_count} issues`}
                  >
                    <TriangleAlert class="size-3" />
                    {row.page.issue_count}
                  </Badge>
                {:else}
                  <span class="tree-clean"></span>
                {/if}
              {/if}
            </div>
          {/each}
        </div>
      </div>

      {#if selectedPage}
        <aside class="tree-panel">
          <div class="flex items-start justify-between gap-2">
            <span class="tree-panel-title">{selectedPage.title || m['tree.title']()}</span>
            <Button
              variant="ghost"
              size="icon"
              class="size-6"
              onclick={() => (selectedPage = null)}
              aria-label={m['detail.close']()}
              title={m['detail.close']()}
            >
              <X class="size-3.5" />
            </Button>
          </div>
          <p class="tree-panel-url" title={selectedPage.url}>{selectedPage.url}</p>
          <dl class="tree-panel-dl">
            <dt>{m['graph.node.status']()}</dt>
            <dd>
              <Badge variant={statusVariant(selectedPage.status_code)}>
                {selectedPage.status_code ?? '—'}
              </Badge>
            </dd>
            <dt>{m['graph.node.depth']()}</dt>
            <dd>{selectedPage.depth}</dd>
            <dt>{m['graph.node.issues']()}</dt>
            <dd class={selectedPage.issue_count > 0 ? 'text-destructive' : ''}>
              {selectedPage.issue_count}
            </dd>
          </dl>
          <div class="tree-panel-foot">
            <a href={selectedPage.url} target="_blank" rel="noreferrer">
              <Button variant="default" size="sm" class="h-7 text-xs">
                {m['graph.node.open']()}
              </Button>
            </a>
          </div>
        </aside>
      {/if}
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

  .tree-body {
    display: flex;
    align-items: stretch;
    gap: 8px;
  }

  .tree-list {
    position: relative;
    max-height: 520px;
    overflow-y: auto;
    flex: 1;
    min-width: 0;
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

  .tree-row-clickable {
    cursor: pointer;
  }

  .tree-row-selected {
    background: color-mix(in srgb, var(--accent) 14%, transparent);
  }

  .tree-row-flash {
    animation: tree-flash 1.6s ease-out;
  }

  @keyframes tree-flash {
    0% {
      background: color-mix(in srgb, var(--accent) 45%, transparent);
    }
    100% {
      background: transparent;
    }
  }

  :global(.tree-search-icon) {
    position: absolute;
    left: 8px;
    top: 50%;
    transform: translateY(-50%);
    color: var(--text-muted);
    pointer-events: none;
  }

  :global(.tree-search) {
    padding-left: 26px;
  }

  :global(.tree-toggle) {
    flex-shrink: 0;
    color: var(--text-muted);
  }

  :global(.tree-folder) {
    flex-shrink: 0;
    color: var(--info);
  }

  :global(.tree-lang) {
    flex-shrink: 0;
    color: var(--accent);
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

  .tree-status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
    box-shadow: 0 0 0 1px rgba(0, 0, 0, 0.25);
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

  .tree-agg {
    flex-shrink: 0;
    min-width: 20px;
    text-align: center;
    font-size: 0.72rem;
    font-weight: 600;
    color: var(--text-secondary);
    background: color-mix(in srgb, var(--text-muted) 12%, transparent);
    border-radius: 999px;
    padding: 1px 7px;
  }

  .tree-panel {
    width: 260px;
    flex-shrink: 0;
    max-height: 520px;
    overflow-y: auto;
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 10px;
    background: var(--bg-card);
  }

  .tree-panel-title {
    font-weight: 600;
    font-size: 0.85rem;
    line-height: 1.25;
    overflow: hidden;
    display: -webkit-box;
    line-clamp: 3;
    -webkit-line-clamp: 3;
    -webkit-box-orient: vertical;
  }

  .tree-panel-url {
    font-size: 0.72rem;
    color: var(--text-secondary);
    word-break: break-all;
  }

  .tree-panel-dl {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 6px 12px;
    align-items: center;
    font-size: 0.8rem;
    margin: 0;
  }

  .tree-panel-dl dt {
    color: var(--text-secondary);
  }

  .tree-panel-dl dd {
    margin: 0;
    text-align: right;
    font-weight: 500;
  }

  .tree-panel-foot {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
    margin-top: auto;
  }
</style>
