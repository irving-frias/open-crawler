<script lang="ts">
  import cytoscape, {
    type Core,
    type ElementDefinition,
    type LayoutOptions,
    type NodeSingular,
    type EdgeSingular,
  } from 'cytoscape';
  import { getSiteGraph } from '$lib/api/results';
  import type { SiteGraph as GraphData, SiteGraphEdge, SiteGraphNode } from '$lib/api/types';
  import { m } from '$lib/paraglide/messages.js';
  import { Badge } from '$lib/components/ui/badge/index.js';
  import { Button } from '$lib/components/ui/button/index.js';
  import { Input } from '$lib/components/ui/input/index.js';
  import { Skeleton } from '$lib/components/ui/skeleton/index.js';
  import * as Select from '$lib/components/ui/select/index.js';
  import { GitBranch, RefreshCw, ScanSearch, Search, Tag, TriangleAlert, X } from 'lucide-svelte';
  import { cn } from '$lib/utils.js';

  let { projectId }: { projectId: string } = $props();

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

  let graph = $state<GraphData | null>(null);
  let loading = $state(false);
  let error = $state('');
  let cy = $state<Core | null>(null);
  let containerEl = $state<HTMLDivElement | null>(null);

  let selectedNode = $state<SiteGraphNode | null>(null);
  let selectedEdge = $state<SiteGraphEdge | null>(null);

  let searchQuery = $state('');
  let statusFilter = $state<StatusClass | 'all'>('all');
  let showLabels = $state(true);
  let layoutId = $state('cose');

  let graphSeq = 0;

  function shortLabel(node: SiteGraphNode): string {
    const base = node.title || node.url;
    const trimmed = base.trim();
    if (trimmed.length <= 24) return trimmed;
    return trimmed.slice(0, 23) + '…';
  }

  function buildElements(data: GraphData): ElementDefinition[] {
    const nodes: ElementDefinition[] = data.nodes.map((n) => {
      const degree = n.in_degree + n.out_degree;
      return {
        data: {
          id: n.url,
          url: n.url,
          label: shortLabel(n),
          degree,
          statusClass: statusClass(n.status_code, n.blocked),
          statusCode: n.status_code,
          issueCount: n.issue_count,
        },
      };
    });
    const edges: ElementDefinition[] = data.edges.map((e, i) => ({
      data: {
        id: `e${i}`,
        source: e.source,
        target: e.target,
        linkType: e.link_type,
        follow: e.is_follow,
      },
    }));
    return nodes.concat(edges);
  }

  function initCy() {
    if (!containerEl || cy) return;
    const nodeFill: Record<StatusClass, string> = {
      '2xx': cssVar('--success', '#51cf66'),
      '3xx': cssVar('--warning', '#ffd43b'),
      '4xx': cssVar('--danger', '#ff6b6b'),
      '5xx': cssVar('--danger', '#ff6b6b'),
      blocked: cssVar('--info', '#74c0fc'),
      unknown: cssVar('--text-muted', '#6b7079'),
    };
    const border = cssVar('--border', '#2c313c');
    const text = cssVar('--text', '#e6e8eb');
    const textMuted = cssVar('--text-muted', '#6b7079');
    const edgeColor = cssVar('--border-muted', '#3d4450');
    const accent = cssVar('--accent', '#667eea');

    const instance = cytoscape({
      container: containerEl,
      style: [
        {
          selector: 'node',
          style: {
            'background-color': (el) => nodeFill[el.data('statusClass') as StatusClass],
            'background-opacity': 0.85,
            'border-width': (el) => (el.data('issueCount') > 0 ? 3 : 1.5),
            'border-color': (el) => (el.data('issueCount') > 0 ? border : textMuted),
            'border-style': (el) => (el.data('statusClass') === 'blocked' ? 'dashed' : 'solid'),
            label: 'data(label)',
            color: text,
            'font-size': 11,
            'text-valign': 'bottom',
            'text-margin-y': 4,
            'text-wrap': 'ellipsis',
            'text-max-width': '120px',
            width: 'mapData(degree, 0, 40, 28, 64)',
            height: 'mapData(degree, 0, 40, 28, 64)',
          },
        },
        {
          selector: 'node:selected',
          style: {
            'border-color': accent,
            'border-width': 3,
          },
        },
        {
          selector: 'node.no-label',
          style: { label: '' },
        },
        {
          selector: 'node.search-hit',
          style: {
            'border-color': accent,
            'border-width': 3,
            opacity: 1,
          },
        },
        {
          selector: 'node.dimmed',
          style: { opacity: 0.12 },
        },
        {
          selector: 'edge',
          style: {
            width: 1.4,
            'line-color': edgeColor,
            'target-arrow-color': edgeColor,
            'target-arrow-shape': 'triangle',
            'arrow-scale': 0.8,
            'curve-style': 'bezier',
            'line-style': (el) => (el.data('follow') ? 'solid' : 'dashed'),
            opacity: 0.75,
          },
        },
        {
          selector: 'edge.dimmed',
          style: { opacity: 0.08 },
        },
        {
          selector: 'edge:selected',
          style: {
            'line-color': accent,
            'target-arrow-color': accent,
            width: 2,
            opacity: 1,
          },
        },
      ],
      elements: [],
      wheelSensitivity: 0.2,
      minZoom: 0.02,
      maxZoom: 4,
    });

    instance.on('tap', 'node', (evt) => {
      const nodeEl = evt.target as NodeSingular;
      const url = nodeEl.data('url') as string;
      const node = graph?.nodes.find((n) => n.url === url) ?? null;
      selectedNode = node;
      selectedEdge = null;
    });
    instance.on('tap', 'edge', (evt) => {
      const edgeEl = evt.target as EdgeSingular;
      const source = edgeEl.data('source') as string;
      const target = edgeEl.data('target') as string;
      const edge = graph?.edges.find((e) => e.source === source && e.target === target) ?? null;
      selectedEdge = edge;
      selectedNode = null;
    });
    instance.on('tap', (evt) => {
      if (evt.target === instance) {
        selectedNode = null;
        selectedEdge = null;
      }
    });

    cy = instance;
  }

  function runLayout(animate = true) {
    const c = cy;
    if (!c) return;
    const options = {
      name: layoutId,
      animate,
      fit: true,
      padding: 40,
      ...(layoutId === 'cose' && {
        idealEdgeLength: 110,
        nodeRepulsion: 12000,
        nodeOverlap: 16,
        gravity: 0.9,
        numIter: 1000,
      }),
      ...(layoutId === 'breadthfirst' && {
        roots: graph?.nodes.filter((n) => n.depth === 0).map((n) => n.url) as string[],
        spacingFactor: 1.1,
      }),
    } as LayoutOptions;
    c.layout(options).run();
  }

  function applyFilter() {
    const c = cy;
    if (!c || !graph) return;
    const q = searchQuery.trim().toLowerCase();
    c.batch(() => {
      c.nodes().forEach((nodeEl) => {
        const url = nodeEl.data('url') as string;
        const sc = nodeEl.data('statusClass') as StatusClass;
        const matchesStatus = statusFilter === 'all' || sc === statusFilter;
        const matchesSearch =
          !q ||
          url.toLowerCase().includes(q) ||
          String(nodeEl.data('label')).toLowerCase().includes(q);
        nodeEl.style('display', matchesStatus ? 'element' : 'none');
        nodeEl.toggleClass('search-hit', !!q && matchesStatus && matchesSearch);
        nodeEl.toggleClass('dimmed', !!q && matchesStatus && !matchesSearch);
      });

      c.edges().forEach((edgeEl) => {
        const src = edgeEl.source();
        const tgt = edgeEl.target();
        const visible = src.visible() && tgt.visible();
        edgeEl.style('display', visible ? 'element' : 'none');
        const bothMatched = !!q && src.hasClass('search-hit') && tgt.hasClass('search-hit');
        edgeEl.toggleClass('dimmed', !!q && visible && !bothMatched);
      });

      if (q) {
        const hits = c.nodes(':visible.search-hit');
        if (hits.length > 0) {
          c.animate({ fit: { eles: hits, padding: 60 }, duration: 300 });
        }
      }
    });
  }

  function focusNode(url: string) {
    if (!cy) return;
    const nodeEl = cy.getElementById(url);
    cy.animate({
      center: { eles: nodeEl },
      zoom: Math.max(cy.zoom(), 0.9),
      duration: 250,
    });
  }

  function focusEdge(source: string, target: string) {
    if (!cy) return;
    const edgeEl = cy.getElementById(
      cy
        .edges()
        .filter((e) => e.data('source') === source && e.data('target') === target)
        .id() as string
    );
    if (edgeEl.length) {
      cy.animate({ center: { eles: edgeEl }, duration: 250 });
    }
  }

  function fitView() {
    if (!cy) return;
    cy.animate({ fit: { eles: cy.elements(':visible'), padding: 40 }, duration: 200 });
  }

  async function loadGraph() {
    if (!projectId) return;
    const seq = ++graphSeq;
    loading = true;
    error = '';
    try {
      const data = await getSiteGraph(projectId);
      if (seq !== graphSeq) return;
      graph = data;
      if (cy) {
        cy.elements().remove();
        cy.add(buildElements(data));
        runLayout(true);
      }
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
    let issues = 0;
    for (const n of graph.nodes) {
      byStatus[statusClass(n.status_code, n.blocked)] += 1;
      issues += n.issue_count;
    }
    return { total: graph.nodes.length, edges: graph.edges.length, issues, byStatus };
  });

  $effect(() => {
    initCy();
  });

  $effect(() => {
    if (cy && graph) {
      applyFilter();
    }
  });

  $effect(() => {
    if (cy && graph) {
      cy.nodes().toggleClass('no-label', !showLabels);
    }
  });

  $effect(() => {
    loadGraph();
  });

  $effect(() => {
    if (cy && graph && layoutId) {
      runLayout(true);
    }
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
          if (v) layoutId = v as string;
        }}
      >
        <Select.Trigger class="h-7 w-32 justify-between text-xs">
          {layoutId}
        </Select.Trigger>
        <Select.Content>
          <Select.Item value="cose">cose</Select.Item>
          <Select.Item value="breadthfirst">breadthfirst</Select.Item>
          <Select.Item value="concentric">concentric</Select.Item>
          <Select.Item value="grid">grid</Select.Item>
          <Select.Item value="circle">circle</Select.Item>
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
              onclick={() => focusNode(selectedNode!.url)}
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
