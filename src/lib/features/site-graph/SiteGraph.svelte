<script lang="ts">
  import { onDestroy } from 'svelte';
  import Sigma from 'sigma';
  import Graph from 'graphology';
  import forceAtlas2 from 'graphology-layout-forceatlas2';
  import { getSiteGraph, getSiteGraphEdges } from '$lib/api/results';
  import type { SiteGraph as GraphData, SiteGraphEdge, SiteGraphNode } from '$lib/api/types';
  import { m } from '$lib/paraglide/messages.js';
  import { Badge } from '$lib/components/ui/badge/index.js';
  import { Button } from '$lib/components/ui/button/index.js';
  import { Input } from '$lib/components/ui/input/index.js';
  import { Skeleton } from '$lib/components/ui/skeleton/index.js';
  import * as Select from '$lib/components/ui/select/index.js';
  import {
    GitBranch,
    Play,
    RefreshCw,
    ScanSearch,
    Search,
    Tag,
    TriangleAlert,
    X,
  } from 'lucide-svelte';
  import { cn } from '$lib/utils.js';

  let { projectId }: { projectId: string } = $props();

  type StatusClass = '2xx' | '3xx' | '4xx' | '5xx' | 'blocked' | 'unknown';
  type LayoutId = 'force' | 'breadthfirst' | 'concentric' | 'grid' | 'circle';

  function statusClass(code: number | null, blocked: boolean | null): StatusClass {
    if (blocked) return 'blocked';
    if (code == null) return 'unknown';
    if (code < 300) return '2xx';
    if (code < 400) return '3xx';
    if (code < 500) return '4xx';
    return '5xx';
  }

  function cssVar(name: string, fallback: string): string {
    return getComputedStyle(document.documentElement).getPropertyValue(name).trim() || fallback;
  }

  const nodeFill: Record<StatusClass, string> = {
    '2xx': cssVar('--success', '#51cf66'),
    '3xx': cssVar('--warning', '#ffd43b'),
    '4xx': cssVar('--danger', '#ff6b6b'),
    '5xx': cssVar('--danger', '#ff6b6b'),
    blocked: cssVar('--info', '#74c0fc'),
    unknown: cssVar('--text-muted', '#6b7079'),
  };
  const edgeColor = cssVar('--border-muted', '#3d4450');

  let graph = $state<GraphData | null>(null);
  let edges = $state<SiteGraphEdge[]>([]);
  let loading = $state(false);
  let error = $state('');
  let containerEl = $state<HTMLDivElement | null>(null);
  let sigma = $state<Sigma | null>(null);
  let selectedNode = $state<SiteGraphNode | null>(null);
  let selectedEdge = $state<SiteGraphEdge | null>(null);
  let hoveredNode = $state<string | null>(null);

  let searchQuery = $state('');
  let statusFilter = $state<StatusClass | 'all'>('all');
  let showLabels = $state(true);
  let layoutId = $state<LayoutId>('force');
  let layouting = $state(false);

  let graphSeq = 0;
  let lastGraphRef: GraphData | null = null;
  let addedEdgeCount = 0;

  function shortLabel(node: SiteGraphNode): string {
    const base = node.title || node.url;
    const trimmed = base.trim();
    if (trimmed.length <= 24) return trimmed;
    return trimmed.slice(0, 23) + '…';
  }

  function nodeHidden(id: string): boolean {
    const s = sigma;
    if (!s) return false;
    const g = s.getGraph();
    const sc = g.getNodeAttribute(id, 'statusClass') as StatusClass;
    if (statusFilter !== 'all' && sc !== statusFilter) return true;
    const q = searchQuery.trim().toLowerCase();
    if (q) {
      const url = g.getNodeAttribute(id, 'url') as string;
      const label = g.getNodeAttribute(id, 'label') as string;
      if (!url.toLowerCase().includes(q) && !label.toLowerCase().includes(q)) return true;
    }
    return false;
  }

  function refresh() {
    const s = sigma;
    if (!s) return;
    s.setSetting('nodeReducer', (node, data) => {
      if (nodeHidden(node)) return { ...data, hidden: true };
      return {
        ...data,
        size: hoveredNode === node ? data.size * 1.7 : data.size,
        label: showLabels ? data.label : '',
      };
    });
    s.setSetting('edgeReducer', (edge, data) => {
      const g = s.getGraph();
      const src = g.source(edge);
      const tgt = g.target(edge);
      if (nodeHidden(src) || nodeHidden(tgt)) return { ...data, hidden: true };
      return data;
    });
    s.refresh();
  }

  function fitView() {
    const s = sigma;
    if (!s || !containerEl) return;
    const bbox = s.getBBox();
    const w = containerEl.clientWidth || 1;
    const h = containerEl.clientHeight || 1;
    const cx = (bbox.x[0] + bbox.x[1]) / 2;
    const cy = (bbox.y[0] + bbox.y[1]) / 2;
    const size = Math.max(bbox.x[1] - bbox.x[0], bbox.y[1] - bbox.y[0], 1);
    const ratio = (Math.min(w, h) / size) * 0.85;
    s.getCamera().animate({ x: cx, y: cy, ratio, angle: 0 }, { duration: 250 });
  }

  function focusUrl(url: string) {
    const s = sigma;
    if (!s) return;
    const g = s.getGraph();
    if (!g.hasNode(url)) return;
    const p = g.getNodeAttributes(url) as { x: number; y: number };
    const ratio = Math.max(s.getCamera().getState().ratio, 0.6);
    s.getCamera().animate({ x: p.x, y: p.y, ratio }, { duration: 250 });
  }

  function focusEdge(source: string, target: string) {
    const s = sigma;
    if (!s) return;
    const g = s.getGraph();
    if (!g.hasNode(source) || !g.hasNode(target)) return;
    const a = g.getNodeAttributes(source) as { x: number; y: number };
    const b = g.getNodeAttributes(target) as { x: number; y: number };
    const ratio = Math.max(s.getCamera().getState().ratio, 0.6);
    s.getCamera().animate({ x: (a.x + b.x) / 2, y: (a.y + b.y) / 2, ratio }, { duration: 250 });
  }

  function applyLayout(id: LayoutId) {
    const s = sigma;
    if (!s) return;
    const g = s.getGraph();
    const order = g.order;
    if (order === 0) return;

    if (id === 'force') {
      layouting = true;
      const settings = forceAtlas2.inferSettings(g);
      forceAtlas2.assign(g, { iterations: 60, settings });
      s.refresh();
      layouting = false;
      fitView();
      return;
    }

    const spacing = 30;
    const nodes = g.mapNodes((node, attrs) => ({
      id: node,
      degree: attrs.degree as number,
      depth: attrs.depth as number,
      x: 0,
      y: 0,
    }));

    if (id === 'grid') {
      const cols = Math.ceil(Math.sqrt(nodes.length));
      nodes.forEach((node, i) => {
        node.x = (i % cols) * spacing;
        node.y = Math.floor(i / cols) * spacing;
      });
    } else if (id === 'circle') {
      const r = (nodes.length * spacing) / (2 * Math.PI);
      nodes.forEach((node, i) => {
        const a = (i / nodes.length) * Math.PI * 2;
        node.x = r * Math.cos(a);
        node.y = r * Math.sin(a);
      });
    } else if (id === 'concentric') {
      nodes.sort((a, b) => b.degree - a.degree);
      let ring = 0;
      let i = 0;
      while (i < nodes.length) {
        const count = ring === 0 ? 1 : 8 * ring;
        for (let j = 0; j < count && i < nodes.length; j++, i++) {
          const a = (j / count) * Math.PI * 2;
          nodes[i].x = ring * spacing * 1.6 * Math.cos(a);
          nodes[i].y = ring * spacing * 1.6 * Math.sin(a);
        }
        ring++;
      }
    } else if (id === 'breadthfirst') {
      const depths = [...new Set(nodes.map((n) => n.depth))].sort((a, b) => a - b);
      const columns = new Map<number, typeof nodes>();
      for (const d of depths)
        columns.set(
          d,
          nodes.filter((n) => n.depth === d)
        );
      depths.forEach((d, di) => {
        const col = columns.get(d)!;
        col.forEach((node, j) => {
          node.x = di * spacing * 2.4;
          node.y = (j - (col.length - 1) / 2) * spacing;
        });
      });
    }

    g.forEachNode((node) => {
      const pos = nodes.find((n) => n.id === node)!;
      g.setNodeAttribute(node, 'x', pos.x);
      g.setNodeAttribute(node, 'y', pos.y);
    });
    s.refresh();
    fitView();
  }

  function buildGraph(data: GraphData): Graph {
    const g = new Graph({ multi: true, type: 'directed' });
    let maxDegree = 1;
    for (const n of data.nodes) {
      maxDegree = Math.max(maxDegree, n.in_degree + n.out_degree);
    }
    const spread = Math.max(Math.sqrt(data.nodes.length), 10);
    for (const n of data.nodes) {
      const sc = statusClass(n.status_code, n.blocked);
      const degree = n.in_degree + n.out_degree;
      g.addNode(n.url, {
        label: shortLabel(n),
        url: n.url,
        statusClass: sc,
        degree,
        depth: n.depth,
        x: (Math.random() - 0.5) * spread,
        y: (Math.random() - 0.5) * spread,
        size: 3 + 12 * Math.sqrt(degree / maxDegree),
        color: nodeFill[sc],
      });
    }
    return g;
  }

  function mountSigma() {
    if (!containerEl || !graph || sigma) return;
    const s = new Sigma(buildGraph(graph), containerEl, {
      minCameraRatio: 0.02,
      maxCameraRatio: 8,
      labelRenderedSizeThreshold: 9,
    });
    sigma = s;
    s.on('clickNode', ({ node }) => {
      selectedNode = graph?.nodes.find((n) => n.url === node) ?? null;
      selectedEdge = null;
    });
    s.on('clickEdge', ({ edge }) => {
      const g = s.getGraph();
      const src = g.source(edge);
      const tgt = g.target(edge);
      selectedEdge = edges.find((e) => e.source === src && e.target === tgt) ?? null;
      selectedNode = null;
    });
    s.on('clickStage', () => {
      selectedNode = null;
      selectedEdge = null;
    });
    s.on('enterNode', ({ node }) => {
      hoveredNode = node;
    });
    s.on('leaveNode', () => {
      hoveredNode = null;
    });
  }

  $effect(() => {
    if (!containerEl || !graph) return;
    if (!sigma || graph !== lastGraphRef) {
      if (sigma) {
        sigma.kill();
        sigma = null;
      }
      mountSigma();
      lastGraphRef = graph;
      addedEdgeCount = 0;
      selectedNode = null;
      selectedEdge = null;
      hoveredNode = null;
      return;
    }
    const g = sigma.getGraph();
    let added = 0;
    for (let i = addedEdgeCount; i < edges.length; i++) {
      const e = edges[i];
      if (e.source === e.target) continue;
      if (!g.hasNode(e.source) || !g.hasNode(e.target)) continue;
      g.addEdge(e.source, e.target, { size: 1, color: edgeColor });
      added++;
    }
    addedEdgeCount = edges.length;
    if (added > 0) sigma.refresh();
  });

  $effect(() => {
    refresh();
  });

  $effect(() => {
    if (sigma && graph) applyLayout(layoutId);
  });

  $effect(() => {
    loadGraph();
  });

  onDestroy(() => {
    sigma?.kill();
    sigma = null;
  });

  async function loadGraph() {
    if (!projectId) return;
    const seq = ++graphSeq;
    loading = true;
    error = '';
    graph = null;
    edges = [];
    try {
      const data = await getSiteGraph(projectId);
      if (seq !== graphSeq) return;
      graph = data;
      const pageSize = 20_000;
      let offset = 0;
      while (true) {
        const page = await getSiteGraphEdges(projectId, offset, pageSize);
        if (seq !== graphSeq) return;
        edges = [...edges, ...page.edges];
        offset += page.edges.length;
        if (page.done || page.edges.length === 0) break;
      }
      if (seq === graphSeq && layoutId === 'force') applyLayout(layoutId);
    } catch (e) {
      if (seq === graphSeq) error = String(e);
    } finally {
      if (seq === graphSeq) loading = false;
    }
  }

  const pageCounts = $derived.by(() => {
    const byStatus: Record<StatusClass, number> = {
      '2xx': 0,
      '3xx': 0,
      '4xx': 0,
      '5xx': 0,
      blocked: 0,
      unknown: 0,
    };
    if (!graph) return { total: 0, edges: 0, issues: 0, byStatus };
    for (const n of graph.nodes) {
      byStatus[statusClass(n.status_code, n.blocked)] += 1;
    }
    return { total: graph.nodes.length, edges: graph.edge_count, issues: 0, byStatus };
  });

  function formatBytes(bytes: number | null): string {
    if (bytes == null) return '—';
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  function formatMs(ms: number | null): string {
    if (ms == null) return '—';
    return `${ms} ms`;
  }

  function statusLabel(sc: StatusClass): string {
    switch (sc) {
      case '2xx':
        return m['graph.filter_2xx']();
      case '3xx':
        return m['graph.filter_3xx']();
      case '4xx':
        return m['graph.filter_4xx']();
      case '5xx':
        return m['graph.filter_5xx']();
      case 'blocked':
        return m['graph.filter_blocked']();
      case 'unknown':
        return m['graph.filter_unknown']();
    }
  }

  function legendColor(sc: StatusClass): string {
    switch (sc) {
      case '2xx':
        return cssVar('--success', '#51cf66');
      case '3xx':
        return cssVar('--warning', '#ffd43b');
      case '4xx':
      case '5xx':
        return cssVar('--danger', '#ff6b6b');
      case 'blocked':
        return cssVar('--info', '#74c0fc');
      case 'unknown':
        return cssVar('--text-muted', '#6b7079');
    }
  }

  function statusVariant(code: number | null): 'default' | 'warning' | 'destructive' {
    if (code == null) return 'default';
    if (code >= 400) return 'destructive';
    if (code >= 300) return 'warning';
    return 'default';
  }

  function layoutLabel(id: LayoutId): string {
    switch (id) {
      case 'force':
        return 'Force';
      case 'breadthfirst':
        return 'Breadth-first';
      case 'concentric':
        return 'Concentric';
      case 'grid':
        return 'Grid';
      case 'circle':
        return 'Circle';
    }
  }
</script>

<div class="site-graph">
  <div class="graph-head">
    <div class="flex items-center gap-2 text-sm font-semibold">
      <GitBranch class="size-4" />
      {m['graph.title']()}
      <span class="sigma-badge">Sigma</span>
      {#if graph}
        <span class="text-xs font-normal text-muted-foreground">
          {m['graph.pages']()}: {pageCounts.total} · {m['graph.edges']()}: {pageCounts.edges}
        </span>
      {/if}
    </div>
    <div class="graph-tools">
      <div class="graph-search">
        <Search class="size-3.5 text-muted-foreground" />
        <Input
          type="text"
          bind:value={searchQuery}
          placeholder={m['graph.search_placeholder']()}
          class="h-7"
        />
        {#if searchQuery}
          <button class="graph-search-clear" onclick={() => (searchQuery = '')} aria-label="clear">
            <X class="size-3" />
          </button>
        {/if}
      </div>
      <Select.Root
        type="single"
        value={statusFilter}
        onValueChange={(v) => {
          if (v) statusFilter = v as StatusClass | 'all';
        }}
      >
        <Select.Trigger class="h-7 w-36 justify-between text-xs">
          {statusFilter === 'all' ? m['graph.filter_all']() : statusLabel(statusFilter)}
        </Select.Trigger>
        <Select.Content>
          <Select.Item value="all">{m['graph.filter_all']()}</Select.Item>
          <Select.Item value="2xx">{m['graph.filter_2xx']()}</Select.Item>
          <Select.Item value="3xx">{m['graph.filter_3xx']()}</Select.Item>
          <Select.Item value="4xx">{m['graph.filter_4xx']()}</Select.Item>
          <Select.Item value="5xx">{m['graph.filter_5xx']()}</Select.Item>
          <Select.Item value="blocked">{m['graph.filter_blocked']()}</Select.Item>
          <Select.Item value="unknown">{m['graph.filter_unknown']()}</Select.Item>
        </Select.Content>
      </Select.Root>
      <Select.Root
        type="single"
        value={layoutId}
        onValueChange={(v) => {
          if (v) layoutId = v as LayoutId;
        }}
      >
        <Select.Trigger class="h-7 w-32 justify-between text-xs">
          {layoutLabel(layoutId)}
        </Select.Trigger>
        <Select.Content>
          <Select.Item value="force">Force</Select.Item>
          <Select.Item value="breadthfirst">Breadth-first</Select.Item>
          <Select.Item value="concentric">Concentric</Select.Item>
          <Select.Item value="grid">Grid</Select.Item>
          <Select.Item value="circle">Circle</Select.Item>
        </Select.Content>
      </Select.Root>
      <Button
        variant="outline"
        size="sm"
        class={cn('h-7 text-xs', showLabels && 'active')}
        onclick={() => (showLabels = !showLabels)}
      >
        <Tag class="size-3.5" />
        {m['graph.show_labels']()}
      </Button>
      <Button variant="outline" size="sm" class="h-7 text-xs" onclick={fitView}>
        <ScanSearch class="size-3.5" />
        {m['graph.fit']()}
      </Button>
      <Button
        variant="outline"
        size="sm"
        class="h-7 text-xs"
        onclick={() => applyLayout(layoutId)}
        disabled={layouting}
      >
        <Play class={cn('size-3.5', layouting && 'animate-spin')} />
        {m['graph.layout']()}
      </Button>
      <Button
        variant="ghost"
        size="icon"
        class="size-7"
        onclick={() => loadGraph()}
        aria-label={m['graph.refresh']()}
        title={m['graph.refresh']()}
        disabled={loading}
      >
        <RefreshCw class={cn('size-3.5', loading && 'animate-spin')} />
      </Button>
    </div>
  </div>

  <div class="graph-legend">
    {#if graph}
      {#each Object.entries(pageCounts.byStatus) as [sc, count] (sc)}
        {#if count > 0}
          <span class="legend-item">
            <span class="legend-dot" style="background: {legendColor(sc as StatusClass)}"></span>
            {statusLabel(sc as StatusClass)}
            <span class="legend-count">{count}</span>
          </span>
        {/if}
      {/each}
    {/if}
  </div>

  {#if graph && (graph.nodes_truncated || graph.edges_truncated)}
    <div class="graph-warn" role="status">
      <TriangleAlert class="size-3.5 shrink-0" />
      {#if graph.nodes_truncated}
        <span>
          {m['graph.truncated_nodes']({
            shown: graph.nodes.length.toLocaleString(),
            total: graph.total_nodes.toLocaleString(),
          })}
        </span>
      {/if}
      {#if graph.edges_truncated}
        <span>
          {m['graph.truncated_edges']({
            shown: edges.length.toLocaleString(),
            total: graph.edge_count.toLocaleString(),
          })}
        </span>
      {/if}
    </div>
  {/if}

  <div class="graph-body">
    <div class="graph-canvas-wrap">
      {#if loading && !graph}
        <div class="flex flex-col gap-2 p-3">
          <Skeleton class="h-6 w-full" />
          <Skeleton class="h-6 w-4/5" />
          <Skeleton class="h-6 w-3/5" />
        </div>
      {:else if error}
        <div class="flex items-center gap-2 p-3 text-sm text-destructive">
          <TriangleAlert class="size-4" />
          {m['graph.error']()}: {error}
        </div>
      {:else if !graph || graph.nodes.length === 0}
        <div class="flex h-full items-center justify-center p-3 text-sm text-muted-foreground">
          {m['graph.empty']()}
        </div>
      {:else}
        <div class="graph-canvas" bind:this={containerEl}></div>
        <div class="graph-hint">{m['graph.select_hint']()}</div>
      {/if}
    </div>

    {#if selectedNode || selectedEdge}
      <aside class="graph-side">
        <div class="graph-side-head">
          <span class="text-xs font-semibold uppercase tracking-wide text-muted-foreground">
            {selectedNode ? shortLabel(selectedNode) : m['graph.edge.source']()}
          </span>
          <button
            class="graph-side-close"
            onclick={() => {
              selectedNode = null;
              selectedEdge = null;
            }}
            aria-label={m['graph.close']()}
          >
            <X class="size-3.5" />
          </button>
        </div>

        {#if selectedNode}
          <div class="graph-side-body">
            <p class="graph-url" title={selectedNode.url}>{selectedNode.url}</p>
            <dl class="graph-dl">
              <dt>{m['graph.node.status']()}</dt>
              <dd>
                <Badge variant={statusVariant(selectedNode.status_code)}>
                  {selectedNode.blocked
                    ? m['results.status.blocked']()
                    : (selectedNode.status_code ?? '—')}
                </Badge>
              </dd>
              <dt>{m['graph.node.depth']()}</dt>
              <dd>{selectedNode.depth}</dd>
              <dt>{m['graph.node.issues']()}</dt>
              <dd class={selectedNode.issue_count > 0 ? 'text-destructive' : ''}>
                {selectedNode.issue_count}
              </dd>
              <dt>{m['graph.node.seo_score']()}</dt>
              <dd>
                {selectedNode.seo_score != null ? `${selectedNode.seo_score.toFixed(1)}` : '—'}
              </dd>
              <dt>{m['graph.node.in_degree']()}</dt>
              <dd>{selectedNode.in_degree}</dd>
              <dt>{m['graph.node.out_degree']()}</dt>
              <dd>{selectedNode.out_degree}</dd>
              <dt>{m['graph.node.size']()}</dt>
              <dd>{formatBytes(selectedNode.size_bytes)}</dd>
              <dt>{m['graph.node.load_time']()}</dt>
              <dd>{formatMs(selectedNode.load_time_ms)}</dd>
            </dl>
          </div>
          <div class="graph-side-foot">
            <Button
              variant="outline"
              size="sm"
              class="h-7 text-xs"
              onclick={() => focusUrl(selectedNode!.url)}
            >
              <ScanSearch class="size-3.5" />
              {m['graph.node.focus']()}
            </Button>
            <a href={selectedNode.url} target="_blank" rel="noreferrer">
              <Button variant="default" size="sm" class="h-7 text-xs">
                {m['graph.node.open']()}
              </Button>
            </a>
          </div>
        {/if}

        {#if selectedEdge}
          <div class="graph-side-body">
            <dl class="graph-dl">
              <dt>{m['graph.edge.source']()}</dt>
              <dd class="graph-url" title={selectedEdge.source}>{selectedEdge.source}</dd>
              <dt>{m['graph.edge.target']()}</dt>
              <dd class="graph-url" title={selectedEdge.target}>{selectedEdge.target}</dd>
              <dt>{m['graph.edge.type']()}</dt>
              <dd>{selectedEdge.link_type || '—'}</dd>
              <dt>{m['graph.edge.follow']()}</dt>
              <dd>
                {selectedEdge.is_follow ? m['graph.edge.follow']() : m['graph.edge.nofollow']()}
              </dd>
            </dl>
          </div>
          <div class="graph-side-foot">
            <Button
              variant="outline"
              size="sm"
              class="h-7 text-xs"
              onclick={() => focusEdge(selectedEdge!.source, selectedEdge!.target)}
            >
              <ScanSearch class="size-3.5" />
              {m['graph.edge.focus']()}
            </Button>
          </div>
        {/if}
      </aside>
    {/if}
  </div>
</div>

<style>
  .site-graph {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .sigma-badge {
    font-size: 0.65rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    padding: 2px 6px;
    border-radius: 5px;
    background: color-mix(in srgb, var(--accent) 15%, transparent);
    border: 1px solid var(--accent);
    color: var(--accent);
  }

  .graph-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding: 4px 8px 0;
    flex-wrap: wrap;
  }

  .graph-tools {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }

  .graph-tools :global(button[class*='active']) {
    background: var(--accent-subtle);
    border-color: var(--accent);
    color: var(--accent);
  }

  .graph-search {
    position: relative;
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .graph-search :global(svg) {
    position: absolute;
    left: 8px;
    pointer-events: none;
  }

  .graph-search :global(input) {
    padding-left: 28px;
    padding-right: 24px;
    width: 220px;
  }

  .graph-search-clear {
    position: absolute;
    right: 6px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 18px;
    height: 18px;
    border-radius: 4px;
    color: var(--text-muted);
    background: transparent;
    border: none;
    cursor: pointer;
  }

  .graph-search-clear:hover {
    background: var(--bg-hover);
    color: var(--text);
  }

  .graph-legend {
    display: flex;
    align-items: center;
    gap: 14px;
    flex-wrap: wrap;
    padding: 0 8px;
    font-size: 0.72rem;
    color: var(--text-secondary);
  }

  .legend-item {
    display: inline-flex;
    align-items: center;
    gap: 5px;
  }

  .legend-dot {
    width: 9px;
    height: 9px;
    border-radius: 50%;
  }

  .legend-count {
    color: var(--text-muted);
    font-variant-numeric: tabular-nums;
  }

  .graph-warn {
    display: flex;
    align-items: center;
    gap: 6px;
    margin: 0 8px;
    padding: 6px 10px;
    border-radius: var(--radius-md);
    border: 1px solid var(--warning);
    background: color-mix(in srgb, var(--warning) 12%, transparent);
    color: var(--warning);
    font-size: 0.75rem;
    line-height: 1.4;
  }

  .graph-body {
    display: flex;
    gap: 10px;
  }

  .graph-canvas-wrap {
    position: relative;
    flex: 1;
    min-width: 0;
    height: 600px;
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    overflow: hidden;
    background: var(--bg-card);
  }

  .graph-canvas {
    width: 100%;
    height: 100%;
  }

  .graph-canvas :global(canvas) {
    outline: none;
  }

  .graph-hint {
    position: absolute;
    bottom: 8px;
    left: 10px;
    font-size: 0.7rem;
    color: var(--text-muted);
    pointer-events: none;
    background: var(--bg-card);
    padding: 2px 6px;
    border-radius: 4px;
    border: 1px solid var(--border);
  }

  .graph-side {
    width: 280px;
    flex-shrink: 0;
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    background: var(--bg-card);
    display: flex;
    flex-direction: column;
    max-height: 600px;
    overflow: hidden;
  }

  .graph-side-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding: 10px 12px;
    border-bottom: 1px solid var(--border);
  }

  .graph-side-close {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 22px;
    height: 22px;
    border-radius: 5px;
    border: none;
    background: transparent;
    color: var(--text-muted);
    cursor: pointer;
  }

  .graph-side-close:hover {
    background: var(--bg-hover);
    color: var(--text);
  }

  .graph-side-body {
    padding: 12px;
    overflow-y: auto;
    flex: 1;
  }

  .graph-side-foot {
    display: flex;
    gap: 8px;
    padding: 10px 12px;
    border-top: 1px solid var(--border);
  }

  .graph-url {
    word-break: break-all;
    font-size: 0.75rem;
    color: var(--text-secondary);
    margin-bottom: 8px;
    line-height: 1.4;
  }

  .graph-dl {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(0, 1.4fr);
    gap: 6px 10px;
    margin: 0;
    font-size: 0.78rem;
  }

  .graph-dl dt {
    color: var(--text-muted);
  }

  .graph-dl dd {
    margin: 0;
    color: var(--text);
    text-align: right;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
