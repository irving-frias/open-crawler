<script lang="ts">
  import { onDestroy } from 'svelte';
  import Sigma from 'sigma';
  import Graph from 'graphology';
  import forceAtlas2 from 'graphology-layout-forceatlas2';
  import type { SiteGraph as GraphData, SiteGraphEdge, SiteGraphNode } from '$lib/api/types';
  import { m } from '$lib/paraglide/messages.js';
  import { Badge } from '$lib/components/ui/badge/index.js';
  import { Button } from '$lib/components/ui/button/index.js';
  import { Input } from '$lib/components/ui/input/index.js';
  import * as Select from '$lib/components/ui/select/index.js';
  import {
    ArrowLeft,
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

  let {
    projectId,
    graph,
    edges,
    onBack,
  }: {
    projectId: string;
    graph: GraphData | null;
    edges: SiteGraphEdge[];
    onBack: () => void;
  } = $props();

  type StatusClass = '2xx' | '3xx' | '4xx' | '5xx' | 'blocked' | 'unknown';

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

  let containerEl = $state<HTMLDivElement | null>(null);
  let sigma = $state<Sigma | null>(null);
  let selectedNode = $state<SiteGraphNode | null>(null);
  let hoveredNode = $state<string | null>(null);
  let searchQuery = $state('');
  let statusFilter = $state<StatusClass | 'all'>('all');
  let showLabels = $state(true);
  let layouting = $state(false);

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
    const cx = (bbox.x1 + bbox.x2) / 2;
    const cy = (bbox.y1 + bbox.y2) / 2;
    const size = Math.max(bbox.x2 - bbox.x1, bbox.y2 - bbox.y1, 1);
    const ratio = (Math.min(w, h) / size) * 0.85;
    s.getCamera().animate({ x: cx, y: cy, ratio, angle: 0 }, { duration: 250 });
  }

  function runLayout() {
    const s = sigma;
    if (!s || layouting) return;
    layouting = true;
    const g = s.getGraph();
    const settings = forceAtlas2.inferSettings(g);
    forceAtlas2.assign(g, {
      iterations: 80,
      settings: { ...settings, gravity: 0.4, scalingRatio: 4 },
    });
    s.refresh();
    layouting = false;
    fitView();
  }

  function buildGraph(data: GraphData, edgeList: SiteGraphEdge[]): Graph {
    const g = new Graph({ multi: true, type: 'directed' });
    let maxDegree = 1;
    for (const n of data.nodes) {
      maxDegree = Math.max(maxDegree, n.in_degree + n.out_degree);
    }
    const spread = Math.max(Math.sqrt(data.nodes.length), 10);
    for (const n of data.nodes) {
      const degree = n.in_degree + n.out_degree;
      g.addNode(n.url, {
        label: shortLabel(n),
        url: n.url,
        statusClass: statusClass(n.status_code, n.blocked),
        x: (Math.random() - 0.5) * spread,
        y: (Math.random() - 0.5) * spread,
        size: 3 + 12 * Math.sqrt(degree / maxDegree),
        color: nodeFill[statusClass(n.status_code, n.blocked)],
      });
    }
    for (const e of edgeList) {
      if (e.source === e.target) continue;
      if (!g.hasNode(e.source) || !g.hasNode(e.target)) continue;
      g.addEdge(e.source, e.target, { size: 1, color: edgeColor });
    }
    return g;
  }

  function mountSigma() {
    if (!containerEl || !graph || sigma) return;
    const s = new Sigma(buildGraph(graph, edges), containerEl, {
      minCameraRatio: 0.02,
      maxCameraRatio: 8,
      labelRenderedSizeThreshold: 9,
      renderLabels: true,
    });
    sigma = s;
    s.on('clickNode', ({ node }) => {
      selectedNode = graph?.nodes.find((n) => n.url === node) ?? null;
    });
    s.on('clickStage', () => {
      selectedNode = null;
    });
    s.on('enterNode', ({ node }) => {
      hoveredNode = node;
    });
    s.on('leaveNode', () => {
      hoveredNode = null;
    });
    runLayout();
  }

  $effect(() => {
    if (containerEl && graph) {
      if (sigma) {
        sigma.kill();
        sigma = null;
        selectedNode = null;
        hoveredNode = null;
      }
      mountSigma();
    }
  });

  $effect(() => {
    refresh();
  });

  onDestroy(() => {
    sigma?.kill();
    sigma = null;
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
</script>

<div class="site-graph">
  <div class="graph-head">
    <div class="flex items-center gap-2 text-sm font-semibold">
      <GitBranch class="size-4" />
      {m['graph.title']()}
      <span class="sigma-badge">Sigma (WebGL)</span>
      {#if graph}
        <span class="text-xs font-normal text-muted-foreground">
          {m['graph.pages']()}: {graph.nodes.length.toLocaleString()} · {m['graph.edges']()}:
          {graph.edge_count.toLocaleString()}
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
        onclick={runLayout}
        disabled={layouting}
      >
        <Play class={cn('size-3.5', layouting && 'animate-spin')} />
        Layout
      </Button>
      <Button
        variant="ghost"
        size="sm"
        class="h-7 text-xs"
        onclick={onBack}
        title="Volver a Cytoscape"
      >
        <ArrowLeft class="size-3.5" />
        Cytoscape
      </Button>
      <Button variant="ghost" size="icon" class="size-7" onclick={() => mountSigma()} title={m['graph.refresh']()}>
        <RefreshCw class="size-3.5" />
      </Button>
    </div>
  </div>

  <div class="graph-legend">
    {#each (['2xx', '3xx', '4xx', '5xx', 'blocked', 'unknown'] as StatusClass[]) as sc (sc)}
      {#if graph?.nodes.some((n) => statusClass(n.status_code, n.blocked) === sc)}
        <span class="legend-item">
          <span class="legend-dot" style="background: {legendColor(sc)}"></span>
          {statusLabel(sc)}
          <span class="legend-count">
            {graph?.nodes.filter((n) => statusClass(n.status_code, n.blocked) === sc).length}
          </span>
        </span>
      {/if}
    {/each}
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
      {#if !graph}
        <div class="flex h-full items-center justify-center p-3 text-sm text-muted-foreground">
          {m['graph.empty']()}
        </div>
      {:else}
        <div class="graph-canvas" bind:this={containerEl}></div>
        <div class="graph-hint">{m['graph.select_hint']()}</div>
      {/if}
    </div>

    {#if selectedNode}
      <aside class="graph-side">
        <div class="graph-side-head">
          <span class="text-xs font-semibold uppercase tracking-wide text-muted-foreground">
            {shortLabel(selectedNode)}
          </span>
          <button class="graph-side-close" onclick={() => (selectedNode = null)} aria-label={m['graph.close']()}>
            <X class="size-3.5" />
          </button>
        </div>
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
            <dd>{selectedNode.seo_score != null ? `${selectedNode.seo_score.toFixed(1)}` : '—'}</dd>
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
          <a href={selectedNode.url} target="_blank" rel="noreferrer">
            <Button variant="default" size="sm" class="h-7 text-xs">
              {m['graph.node.open']()}
            </Button>
          </a>
        </div>
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
