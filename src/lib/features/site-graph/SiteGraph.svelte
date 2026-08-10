<script lang="ts">
  import { onDestroy as onDestroyLifecycle } from 'svelte';
  import { Graph as CosmosGraph, type GraphConfig } from '@cosmos.gl/graph';
  import type { SiteGraph as GraphData, SiteGraphEdge, SiteGraphNode } from '$lib/api/types';
  import { m } from '$lib/paraglide/messages.js';
  import { Badge } from '$lib/components/ui/badge/index.js';
  import { Button } from '$lib/components/ui/button/index.js';
  import { Input } from '$lib/components/ui/input/index.js';
  import { Skeleton } from '$lib/components/ui/skeleton/index.js';
  import * as Select from '$lib/components/ui/select/index.js';
  import {
    GitBranch,
    Network,
    Play,
    RefreshCw,
    ScanSearch,
    Search,
    Tag,
    TriangleAlert,
    X,
  } from 'lucide-svelte';
  import { cn } from '$lib/utils.js';
  import { getSiteGraph, getSiteGraphEdges } from '$lib/api/results';
  import {
    cssVar,
    followEdgeRgba,
    formatBytes,
    formatMs,
    hexToRgba,
    languageOf,
    legendColor,
    nofollowEdgeRgba,
    nodeFill,
    shortLabel,
    statusClass,
    statusLabel,
    statusVariant,
    type StatusClass,
  } from '$lib/features/site-map/shared.js';
  import { requestFocusInTree, siteMapNav } from '$lib/features/site-map/nav.svelte.js';
  import {
    resetSiteMapFilters,
    siteMapFilters,
    type StatusFilter,
  } from '$lib/features/site-map/filters.svelte.js';
  import { resolveColor } from '$lib/components/charts/chart-theme.js';

  let { projectId }: { projectId: string } = $props();

  // Performance limits for large graphs
  const MAX_NODES = 50000;
  const MAX_EDGES = 100000;
  const SPACE_SIZE = 4096;
  const MAX_LABELS = 2500;
  const LABEL_MIN_ZOOM = 1;

  type LayoutId = 'force' | 'breadthfirst' | 'concentric' | 'grid' | 'circle';

  interface NodeMeta {
    title: string;
    status: StatusClass;
    lang: string | null;
    degree: number;
  }

  let graph = $state<GraphData | null>(null);
  let edges = $state<SiteGraphEdge[]>([]);
  let loading = $state(false);
  let error = $state('');
  let containerEl = $state<HTMLDivElement | null>(null);

  let cosmos = $state<CosmosGraph | null>(null);
  let selectedNode = $state<SiteGraphNode | null>(null);
  let hoveredUrl = $state<string | null>(null);

  let languageFilter = $state<'all' | 'default' | string>('all');
  let showLabels = $state(true);
  let layoutId = $state<LayoutId>('force');
  let layouting = $state(false);

  let graphSeq = 0;
  let urls: string[] = [];
  let indexByUrl = new Map<string, number>();
  let meta: NodeMeta[] = [];
  let posCache: Float32Array = new Float32Array(0);
  let baseColors = new Float32Array(0);
  let baseSizes = new Float32Array(0);
  let initialLayoutDone = false;
  let lastLayout: LayoutId | null = null;
  let hadSize = false;
  let resizeObs: ResizeObserver | null = null;

  // DOM label overlay (Cosmograph does not render text labels natively).
  let labelsContainer: HTMLDivElement | null = null;
  let labelEls = new Map<number, HTMLDivElement>();
  let labelIndices: number[] = [];
  let labelsReady = false;
  let labelsFrame = 0;

  const edgeFollowColor = `rgb(${followEdgeRgba[0]}, ${followEdgeRgba[1]}, ${followEdgeRgba[2]})`;
  const highlightColor = resolveColor(cssVar('--accent', '#667eea'));

  function visibleAtFilter(i: number): boolean {
    const md = meta[i];
    if (!md) return false;
    if (siteMapFilters.status !== 'all' && md.status !== siteMapFilters.status) return false;
    if (languageFilter !== 'all') {
      const lang = md.lang ?? null;
      if (languageFilter === 'default' ? lang !== null : lang !== languageFilter) return false;
    }
    const q = siteMapFilters.search.trim().toLowerCase();
    if (q) {
      const url = urls[i].toLowerCase();
      const label = md.title.toLowerCase();
      if (!url.includes(q) && !label.includes(q)) return false;
    }
    return true;
  }

  function applyFilter() {
    const g = cosmos;
    if (!g || urls.length === 0 || !g.isReady) return;
    const current = g.getPointPositions();
    for (let i = 0; i < urls.length; i++) {
      if (visibleAtFilter(i)) {
        posCache[i * 2] = current[i * 2];
        posCache[i * 2 + 1] = current[i * 2 + 1];
      }
    }
    const filtered = posCache.slice();
    for (let i = 0; i < urls.length; i++) {
      if (!visibleAtFilter(i)) {
        filtered[i * 2] = NaN;
        filtered[i * 2 + 1] = NaN;
      }
    }
    g.setPointPositions(new Float32Array(filtered));
    g.render();
    if (layoutId === 'force') g.unpause();
    requestLabelUpdate();
  }

  function requestLabelUpdate() {
    if (labelsFrame) return;
    labelsFrame = requestAnimationFrame(() => {
      labelsFrame = 0;
      updateLabels();
    });
  }

  function updateLabels() {
    const g = cosmos;
    if (!g || !labelsReady) return;
    if (!labelsContainer) return;
    if (!showLabels) {
      labelsContainer.style.display = 'none';
      return;
    }
    labelsContainer.style.display = '';
    const map = g.getTrackedPointPositionsMap();
    const zoom = g.getZoomLevel();
    const showAll = zoom >= LABEL_MIN_ZOOM;
    for (const idx of labelIndices) {
      const div = labelEls.get(idx);
      if (!div) continue;
      if (!showAll) {
        div.style.display = 'none';
        continue;
      }
      const p = map.get(idx);
      if (!p) {
        div.style.display = 'none';
        continue;
      }
      const [sx, sy] = g.spaceToScreenPosition(p);
      div.style.display = '';
      div.style.transform = `translate(-50%, -100%) translate(${sx}px, ${sy}px)`;
    }
  }

  function ensureLabel(idx: number) {
    const g = cosmos;
    if (!g || !labelsContainer || !showLabels) return;
    if (labelEls.has(idx)) return;
    const div = document.createElement('div');
    div.className = 'graph-label';
    div.textContent = shortLabel(meta[idx]?.title ?? urls[idx] ?? '');
    labelsContainer.appendChild(div);
    labelEls.set(idx, div);
    labelIndices.push(idx);
    g.trackPointPositionsByIndices(labelIndices);
    requestLabelUpdate();
  }

  function initLabels() {
    const g = cosmos;
    const el = containerEl;
    if (!g || !el || labelsReady) return;
    labelsReady = true;
    labelsContainer = document.createElement('div');
    labelsContainer.className = 'graph-labels';
    el.appendChild(labelsContainer);
    const order = urls
      .map((_, i) => i)
      .sort((a, b) => (meta[b]?.degree ?? 0) - (meta[a]?.degree ?? 0));
    const selected = order.slice(0, MAX_LABELS);
    for (const idx of selected) {
      const div = document.createElement('div');
      div.className = 'graph-label';
      div.textContent = shortLabel(meta[idx]?.title ?? urls[idx] ?? '');
      labelsContainer.appendChild(div);
      labelEls.set(idx, div);
    }
    labelIndices = selected;
    g.trackPointPositionsByIndices(labelIndices);
    updateLabels();
  }

  function fitView() {
    const g = cosmos;
    if (!g || urls.length === 0) return;
    g.fitView();
    requestLabelUpdate();
  }

  function focusUrl(url: string) {
    const g = cosmos;
    if (!g) return;
    const idx = indexByUrl.get(url);
    if (idx === undefined) return;
    g.zoomToPointByIndex(idx, 400, 3);
    requestLabelUpdate();
  }

  function selectNode(idx: number) {
    const g = cosmos;
    const nd = graph?.nodes[idx];
    if (!g || !nd) return;
    selectedNode = nd;
    hoveredUrl = null;
    g.setConfigPartial({ focusedPointIndex: idx });
    ensureLabel(idx);
  }

  function clearSelection() {
    const g = cosmos;
    if (!g) return;
    selectedNode = null;
    g.setConfigPartial({ focusedPointIndex: undefined });
  }

  function handlePointClick(index: number) {
    selectNode(index);
  }

  function handleBackgroundClick() {
    clearSelection();
  }

  function handlePointMouseOver(index: number) {
    hoveredUrl = urls[index] ?? null;
    ensureLabel(index);
  }

  function handlePointMouseOut() {
    hoveredUrl = null;
  }

  function handleSimulationTick() {
    requestLabelUpdate();
  }

  function handleZoom() {
    requestLabelUpdate();
  }

  const cosmosConfig: GraphConfig = {
    spaceSize: SPACE_SIZE,
    rescalePositions: false,
    backgroundColor: resolveColor(cssVar('--bg-card', '#0b0f1a')),
    pointDefaultSize: 4,
    pointSizeScale: 1,
    scalePointsOnZoom: true,
    hoveredPointCursor: 'pointer',
    renderHoveredPointRing: true,
    hoveredPointRingColor: highlightColor,
    focusedPointRingColor: highlightColor,
    renderLinks: true,
    linkDefaultColor: edgeFollowColor,
    linkOpacity: 0.5,
    linkGreyoutOpacity: 0.06,
    linkDefaultWidth: 1,
    linkWidthScale: 1,
    linkBlending: false,
    curvedLinks: true,
    linkDefaultArrows: true,
    linkArrowsSizeScale: 0.7,
    linkDashLength: 4,
    linkDashGap: 3,
    enableZoom: true,
    enableDrag: true,
    enableSimulation: true,
    enableSimulationDuringZoom: false,
    fitViewOnInit: true,
    fitViewDelay: 500,
    fitViewPadding: 0.1,
    transitionDuration: 500,
    simulationFriction: 0.85,
    simulationLinkSpring: 1,
    simulationLinkDistance: 15,
    simulationRepulsion: 1,
    simulationGravity: 0.1,
    simulationDecay: 50000,
    onPointClick: handlePointClick,
    onBackgroundClick: handleBackgroundClick,
    onPointMouseOver: handlePointMouseOver,
    onPointMouseOut: handlePointMouseOut,
    onSimulationTick: handleSimulationTick,
    onZoom: handleZoom,
  };

  function layoutLabel(id: LayoutId): string {
    switch (id) {
      case 'force':
        return 'Force';
      case 'breadthfirst':
        return 'Breadthfirst';
      case 'concentric':
        return 'Concentric';
      case 'grid':
        return 'Grid';
      case 'circle':
        return 'Circle';
    }
  }

  function computePositions(id: LayoutId): Float32Array | null {
    const data = graph;
    if (!data) return null;
    const order = data.nodes.length;
    const positions = new Float32Array(order * 2);
    const spacing = 30;

    if (id === 'grid') {
      const cols = Math.ceil(Math.sqrt(order));
      for (let i = 0; i < order; i++) {
        positions[i * 2] = (i % cols) * spacing;
        positions[i * 2 + 1] = Math.floor(i / cols) * spacing;
      }
    } else if (id === 'circle') {
      const radius = (order * spacing) / (2 * Math.PI);
      for (let i = 0; i < order; i++) {
        const a = (i / order) * Math.PI * 2;
        positions[i * 2] = radius * Math.cos(a);
        positions[i * 2 + 1] = radius * Math.sin(a);
      }
    } else if (id === 'concentric') {
      const indexList = data.nodes.map((_, i) => i);
      indexList.sort((a, b) => {
        const da = data.nodes[a].in_degree + data.nodes[a].out_degree;
        const db = data.nodes[b].in_degree + data.nodes[b].out_degree;
        return db - da;
      });
      let ring = 0;
      let i = 0;
      while (i < indexList.length) {
        const count = ring === 0 ? 1 : 8 * ring;
        for (let j = 0; j < count && i < indexList.length; j++, i++) {
          const a = (j / count) * Math.PI * 2;
          const idx = indexList[i];
          positions[idx * 2] = ring * spacing * 1.6 * Math.cos(a);
          positions[idx * 2 + 1] = ring * spacing * 1.6 * Math.sin(a);
        }
        ring++;
      }
    } else if (id === 'breadthfirst') {
      const depthOf = (i: number) => data.nodes[i].depth ?? 0;
      const depths = [...new Set(Array.from({ length: order }, (_, i) => depthOf(i)))].sort(
        (a, b) => a - b
      );
      depths.forEach((d, di) => {
        const col: number[] = [];
        for (let i = 0; i < order; i++) if (depthOf(i) === d) col.push(i);
        col.forEach((idx, j) => {
          positions[idx * 2] = di * spacing * 2.4;
          positions[idx * 2 + 1] = (j - (col.length - 1) / 2) * spacing;
        });
      });
    }

    // Scale the computed layout so it fits inside the simulation space.
    let maxAbs = 1;
    for (let i = 0; i < order; i++) {
      maxAbs = Math.max(maxAbs, Math.abs(positions[i * 2]), Math.abs(positions[i * 2 + 1]));
    }
    const s = ((SPACE_SIZE / 2) * 0.9) / maxAbs;
    for (let i = 0; i < order; i++) {
      positions[i * 2] *= s;
      positions[i * 2 + 1] *= s;
    }
    return positions;
  }

  function applyLayout(id: LayoutId) {
    const g = cosmos;
    if (!g || urls.length === 0) return;
    if (layouting) return;
    layouting = true;
    try {
      if (id === 'force') {
        g.setConfigPartial({ enableSimulation: true });
        g.start();
        requestLabelUpdate();
      } else {
        const layoutPositions = computePositions(id);
        if (!layoutPositions) return;
        for (let i = 0; i < urls.length; i++) {
          if (visibleAtFilter(i)) {
            posCache[i * 2] = layoutPositions[i * 2];
            posCache[i * 2 + 1] = layoutPositions[i * 2 + 1];
          }
        }
        const filtered = posCache.slice();
        for (let i = 0; i < urls.length; i++) {
          if (!visibleAtFilter(i)) {
            filtered[i * 2] = NaN;
            filtered[i * 2 + 1] = NaN;
          }
        }
        g.setConfigPartial({ enableSimulation: false });
        g.setPointPositions(new Float32Array(filtered));
        g.pause();
        g.render();
        requestLabelUpdate();
      }
    } finally {
      layouting = false;
    }
  }

  function buildLinks() {
    const g = cosmos;
    if (!g || urls.length === 0) return;
    const valid = edges.filter(
      (e) => e.source !== e.target && indexByUrl.has(e.source) && indexByUrl.has(e.target)
    );
    const n = valid.length;
    const links = new Float32Array(n * 2);
    const linkColors = new Float32Array(n * 4);
    const linkStyles = new Float32Array(n);
    const linkArrows: boolean[] = new Array(n);
    for (let i = 0; i < n; i++) {
      const e = valid[i];
      links[i * 2] = indexByUrl.get(e.source)!;
      links[i * 2 + 1] = indexByUrl.get(e.target)!;
      const [r, gg, b] = e.is_follow ? followEdgeRgba : nofollowEdgeRgba;
      linkColors[i * 4] = r;
      linkColors[i * 4 + 1] = gg;
      linkColors[i * 4 + 2] = b;
      linkColors[i * 4 + 3] = 0.6;
      linkStyles[i] = e.is_follow ? 0 : 1;
      linkArrows[i] = true;
    }
    g.setLinks(links);
    g.setLinkColors(linkColors);
    g.setLinkStyles(linkStyles);
    g.setLinkArrows(linkArrows);
    g.render();
  }

  function destroyGraph() {
    if (cosmos) {
      try {
        cosmos.destroy();
      } catch (e) {
        console.error('Error destroying graph:', e);
      }
      cosmos = null;
    }
    resizeObs?.disconnect();
    resizeObs = null;
    const el = containerEl;
    if (el) el.innerHTML = '';
    labelsReady = false;
    labelsContainer = null;
    labelEls.clear();
    labelIndices = [];
    urls = [];
    indexByUrl = new Map();
    meta = [];
    selectedNode = null;
    hoveredUrl = null;
    initialLayoutDone = false;
    lastLayout = null;
    hadSize = false;
  }

  $effect(() => {
    const el = containerEl;
    const data = graph;

    destroyGraph();
    if (!el || !data) return;

    const nodes = data.nodes;
    const n = nodes.length;

    let maxDegree = 1;
    for (const nd of nodes) {
      maxDegree = Math.max(maxDegree, nd.in_degree + nd.out_degree);
    }
    const spread = SPACE_SIZE * 0.3;
    urls = new Array(n);
    indexByUrl = new Map();
    meta = new Array(n);
    const positions = new Float32Array(n * 2);
    const colors = new Float32Array(n * 4);
    const sizes = new Float32Array(n);
    for (let i = 0; i < n; i++) {
      const nd = nodes[i];
      const sc = statusClass(nd.status_code, nd.blocked);
      const degree = nd.in_degree + nd.out_degree;
      urls[i] = nd.url;
      indexByUrl.set(nd.url, i);
      meta[i] = {
        title: nd.title || nd.url,
        status: sc,
        lang: languageOf(nd.url),
        degree,
      };
      positions[i * 2] = (Math.random() - 0.5) * 2 * spread;
      positions[i * 2 + 1] = (Math.random() - 0.5) * 2 * spread;
      const [r, gg, b] = hexToRgba(nodeFill[sc]);
      colors[i * 4] = r;
      colors[i * 4 + 1] = gg;
      colors[i * 4 + 2] = b;
      colors[i * 4 + 3] = 1;
      sizes[i] = 4 + 14 * Math.sqrt(degree / maxDegree);
    }
    posCache = positions;
    baseColors = colors;
    baseSizes = sizes;

    const instance = new CosmosGraph(el, cosmosConfig);
    cosmos = instance;

    instance.ready.then(() => {
      if (cosmos !== instance) return;
      instance.setPointPositions(posCache);
      instance.setPointColors(baseColors);
      instance.setPointSizes(baseSizes);
      instance.render();
      buildLinks();
      if (showLabels) initLabels();
      tryFocusPending();
    });

    resizeObs = new ResizeObserver(() => {
      const g = cosmos;
      if (!g || !el) return;
      const w = el.clientWidth;
      const h = el.clientHeight;
      if (!w || !h) {
        hadSize = false;
        return;
      }
      if (!hadSize) {
        hadSize = true;
        setTimeout(() => g.fitView(), 60);
      }
      requestLabelUpdate();
    });
    resizeObs.observe(el);
  });

  $effect(() => {
    const g = cosmos;
    if (!g || urls.length === 0) return;
    buildLinks();
  });

  $effect(() => {
    const g = cosmos;
    if (!g || urls.length === 0) return;
    applyFilter();
  });

  $effect(() => {
    const g = cosmos;
    if (!g) return;
    if (showLabels) {
      initLabels();
    } else if (labelsContainer) {
      labelsContainer.style.display = 'none';
    }
  });

  // Run the selected layout once, after all edges have finished loading.
  $effect(() => {
    const g = cosmos;
    const data = graph;
    if (!g || !data) return;
    if (loading) return;
    if (!initialLayoutDone) {
      initialLayoutDone = true;
      applyLayout(layoutId);
    }
  });

  // Re-apply a layout when the selected layout changes.
  $effect(() => {
    if (!cosmos) return;
    if (lastLayout === null) {
      lastLayout = layoutId;
      return;
    }
    if (lastLayout !== layoutId) {
      lastLayout = layoutId;
      applyLayout(layoutId);
    }
  });

  onDestroyLifecycle(() => {
    destroyGraph();
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

      // Limit nodes for performance
      if (data.nodes.length > MAX_NODES) {
        console.warn(
          `Graph too large (${data.nodes.length} nodes). Limiting to ${MAX_NODES} nodes for performance.`
        );
        data.nodes = data.nodes.slice(0, MAX_NODES);
        data.edge_count = Math.min(data.edge_count, MAX_EDGES);
      }

      graph = data;
      const pageSize = 20_000;
      let offset = 0;
      let totalEdges = 0;
      while (true) {
        const page = await getSiteGraphEdges(projectId, offset, pageSize);
        if (seq !== graphSeq) return;

        // Limit edges for performance
        if (totalEdges + page.edges.length > MAX_EDGES) {
          const remaining = MAX_EDGES - totalEdges;
          if (remaining > 0) {
            edges = [...edges, ...page.edges.slice(0, remaining)];
          }
          break;
        }

        edges = [...edges, ...page.edges];
        totalEdges += page.edges.length;
        offset += page.edges.length;

        if (page.done || page.edges.length === 0 || totalEdges >= MAX_EDGES) break;
      }
    } catch (e) {
      if (seq === graphSeq) error = String(e);
    } finally {
      if (seq === graphSeq) loading = false;
    }
  }

  $effect(() => {
    loadGraph();
  });

  const pageCounts = $derived.by(() => {
    const byStatus: Record<StatusClass, number> = {
      '2xx': 0,
      '3xx': 0,
      '4xx': 0,
      '5xx': 0,
      blocked: 0,
      unknown: 0,
    };
    if (!graph) return { total: 0, edges: 0, byStatus };
    for (const n of graph.nodes) {
      byStatus[statusClass(n.status_code, n.blocked)] += 1;
    }
    return { total: graph.nodes.length, edges: graph.edge_count, byStatus };
  });

  const languages = $derived.by(() => {
    const set = new Set<string>();
    let hasDefault = false;
    if (graph) {
      for (const n of graph.nodes) {
        const lang = languageOf(n.url);
        if (lang) set.add(lang);
        else hasDefault = true;
      }
    }
    return { codes: [...set].sort(), hasDefault };
  });

  // Cross-navigation: when the site tree asks to show a page in the graph,
  // reset filters so the node is visible, select it and center the camera.
  let pendingFocusUrl: string | null = null;
  let lastNavSeq = 0;

  function tryFocusPending() {
    const url = pendingFocusUrl;
    const g = cosmos;
    if (!url || !g || urls.length === 0) return;
    const idx = indexByUrl.get(url);
    if (idx === undefined) return;
    pendingFocusUrl = null;
    resetSiteMapFilters();
    hoveredUrl = null;
    selectedNode = graph?.nodes[idx] ?? null;
    g.setConfigPartial({ focusedPointIndex: idx });
    g.zoomToPointByIndex(idx, 400, 3);
    ensureLabel(idx);
    requestLabelUpdate();
  }

  $effect(() => {
    const nav = siteMapNav;
    if (!nav.url || nav.projectId !== projectId || nav.action !== 'graph') return;
    if (nav.seq === lastNavSeq) return;
    lastNavSeq = nav.seq;
    pendingFocusUrl = nav.url;
    tryFocusPending();
  });

  $effect(() => {
    if (cosmos && graph) tryFocusPending();
  });
</script>

<div class="site-graph">
  <div class="graph-head">
    <div class="flex items-center gap-2 text-sm font-semibold">
      <GitBranch class="size-4" />
      {m['graph.title']()}
      <span class="engine-badge">Cosmos</span>
      {#if graph}
        <span class="text-xs font-normal text-muted-foreground">
          {m['graph.pages']()}: {pageCounts.total.toLocaleString()} · {m['graph.edges']()}:
          {pageCounts.edges.toLocaleString()}
        </span>
      {/if}
    </div>
    <div class="graph-tools">
      <div class="graph-search">
        <Search class="size-3.5 text-muted-foreground" />
        <Input
          type="text"
          bind:value={siteMapFilters.search}
          placeholder={m['graph.search_placeholder']()}
          class="h-7"
        />
        {#if siteMapFilters.search}
          <button
            class="graph-search-clear"
            onclick={() => (siteMapFilters.search = '')}
            aria-label="clear"
          >
            <X class="size-3" />
          </button>
        {/if}
      </div>
      <Select.Root
        type="single"
        value={siteMapFilters.status}
        onValueChange={(v) => {
          if (v) siteMapFilters.status = v as StatusFilter;
        }}
      >
        <Select.Trigger class="h-7 w-36 justify-between text-xs">
          {siteMapFilters.status === 'all' ? m['graph.filter_all']() : statusLabel(siteMapFilters.status)}
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
        value={languageFilter}
        onValueChange={(v) => {
          if (v) languageFilter = v as 'all' | 'default' | string;
        }}
      >
        <Select.Trigger
          class="h-7 w-28 justify-between text-xs"
          aria-label={m['graph.filter_language']()}
        >
          {languageFilter === 'all'
            ? m['graph.language_all']()
            : languageFilter === 'default'
              ? m['graph.language_default']()
              : languageFilter}
        </Select.Trigger>
        <Select.Content>
          <Select.Item value="all">{m['graph.language_all']()}</Select.Item>
          {#if languages.hasDefault}
            <Select.Item value="default">{m['graph.language_default']()}</Select.Item>
          {/if}
          {#each languages.codes as lang (lang)}
            <Select.Item value={lang}>{lang}</Select.Item>
          {/each}
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
        {#if hoveredUrl}
          <div class="graph-hover" title={hoveredUrl}>{hoveredUrl}</div>
        {/if}
      {/if}
    </div>

    {#if selectedNode}
      <aside class="graph-side">
        <div class="graph-side-head">
          <span class="text-xs font-semibold uppercase tracking-wide text-muted-foreground">
            {shortLabel(selectedNode.title || selectedNode.url)}
          </span>
          <button
            class="graph-side-close"
            onclick={() => clearSelection()}
            aria-label={m['graph.close']()}
          >
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
            onclick={() => requestFocusInTree(projectId, selectedNode!.url)}
          >
            <Network class="size-3.5" />
            {m['graph.view_in_tree']()}
          </Button>
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

  .engine-badge {
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
    position: relative;
    width: 100%;
    height: 100%;
  }

  .graph-canvas :global(canvas) {
    outline: none;
  }

  :global(.graph-labels) {
    position: absolute;
    inset: 0;
    overflow: hidden;
    pointer-events: none;
    z-index: 2;
  }

  :global(.graph-label) {
    position: absolute;
    top: 0;
    left: 0;
    font-size: 0.68rem;
    line-height: 1.3;
    color: var(--text);
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 1px 5px;
    white-space: nowrap;
    will-change: transform;
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

  .graph-hover {
    position: absolute;
    bottom: 8px;
    right: 10px;
    font-size: 0.7rem;
    color: var(--text-secondary);
    max-width: 60%;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
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
