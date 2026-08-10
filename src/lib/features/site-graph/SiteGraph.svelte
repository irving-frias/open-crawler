<script lang="ts">
  import { onDestroy as onDestroyLifecycle, untrack } from 'svelte';
  import { Chart, PointElement } from 'chart.js';
  import type { ActiveElement, ChartEvent } from 'chart.js';
  import { forceSimulation, forceLink, forceManyBody, forceCenter, forceCollide } from 'd3-force';
  import DatalabelsPlugin from 'chartjs-plugin-datalabels';
  import zoomPlugin from 'chartjs-plugin-zoom';
  import type { SiteGraph as GraphData, SiteGraphEdge, SiteGraphNode } from '$lib/api/types';
  import { m } from '$lib/paraglide/messages.js';
  import { Badge } from '$lib/components/ui/badge/index.js';
  import { Button } from '$lib/components/ui/button/index.js';
  import { Input } from '$lib/components/ui/input/index.js';
  import { Skeleton } from '$lib/components/ui/skeleton/index.js';
  import * as Select from '$lib/components/ui/select/index.js';
  import {
    GitBranch,
    ListCollapse,
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
  import { resolveColor, watchTheme } from '$lib/components/charts/chart-theme.js';

  let { projectId }: { projectId: string } = $props();

  // Performance limits for large graphs
  const MAX_NODES = 50000;
  const MAX_EDGES = 100000;
  const SPACE_SIZE = 4096;
  const MAX_LABELS = 2500;
  // Only render node labels once the graph is zoomed in far enough.
  const LABEL_MIN_ZOOM = 1.4;
  // Above this node count the default layout skips the force simulation
  // (physics on tens of thousands of nodes is the main source of jank); a fast
  // deterministic layout is used instead and Force stays one click away.
  const FORCE_SIM_MAX_NODES = 3000;
  // Chart.js draws every edge as its own canvas path (plus per-edge color, dash
  // and arrow head), so tens of thousands of links freeze the tab. Above this
  // count we render only the highest-degree links with a flat style.
  const MAX_RENDERED_EDGES = 12000;

  type LayoutId = 'force' | 'breadthfirst' | 'concentric' | 'grid' | 'circle';

  interface NodeMeta {
    title: string;
    status: StatusClass;
    lang: string | null;
    degree: number;
    depth: number;
  }

  interface VisibleData {
    labels: string[];
    data: { x: number; y: number }[];
    edges: { source: number; target: number }[];
    edgeColors: string[];
    edgeDashes: number[][];
    radii: number[];
    colors: string[];
    borders: string[];
    bw: number[];
    visible: number[];
  }

  let graph = $state<GraphData | null>(null);
  let edges = $state<SiteGraphEdge[]>([]);
  let loading = $state(false);
  let error = $state('');
  let canvasEl = $state<HTMLCanvasElement | null>(null);

  let selectedNode = $state<SiteGraphNode | null>(null);
  let hoveredUrl = $state<string | null>(null);

  let languageFilter = $state<'all' | 'default' | string>('all');
  let showLabels = $state(true);
  let layoutId = $state<LayoutId>('force');
  let layouting = $state(false);
  let searchInput = $state('');
  let themeTick = $state(0);
  // Bumped whenever the chart data must be refreshed (filters, layout, ...).
  let dataTick = $state(0);

  let chart: Chart | null = null;
  let graphSeq = 0;
  let urls: string[] = [];
  let indexByUrl = new Map<string, number>();
  let meta: NodeMeta[] = [];
  let posCache = new Float32Array(0);
  let baseSizes = new Float32Array(0);
  let baseColors: string[] = [];
  let edgePairs: Array<[number, number]> = [];
  let edgeColors: string[] = [];
  let edgeDashes: number[][] = [];
  let lastVisibleIdx: number[] = [];
  let labelTopSet = new Set<number>();
  // True while the graph has more links than MAX_RENDERED_EDGES: edges render
  // with a single flat style and no arrow heads to keep the tab responsive.
  let largeEdgeMode = false;
  // Level-by-level expansion, active in every layout: only the root level is
  // shown initially; clicking a node with children expands it, revealing the
  // next level. `expandedNodes` holds the indices of expanded nodes.
  let expandedNodes = $state<Set<number>>(new Set());
  // Parent -> children index built from the loaded edges (deduplicated).
  let childMap = new Map<number, Array<{ c: number; follow: boolean }>>();
  // Deepest "root" depth of the crawl (usually 0). Nodes at this depth form the
  // initial level.
  let levelRootDepth = 0;
  let pendingFocusUrl: string | null = null;
  let lastNavSeq = 0;
  let debounceTimer: ReturnType<typeof setTimeout> | null = null;

  const highlightColor = resolveColor(cssVar('--accent', '#667eea'));
  const followEdge = `rgba(${followEdgeRgba[0]}, ${followEdgeRgba[1]}, ${followEdgeRgba[2]}, 0.55)`;
  const nofollowEdge = `rgba(${nofollowEdgeRgba[0]}, ${nofollowEdgeRgba[1]}, ${nofollowEdgeRgba[2]}, 0.38)`;

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

  function getVisibleData(): VisibleData {
    const n = urls.length;
    const levelInfo = levelTree();
    const visible: number[] = [];
    for (let i = 0; i < n; i++) {
      if (!visibleAtFilter(i)) continue;
      if (!levelInfo.visible.has(i)) continue;
      visible.push(i);
    }
    const map = new Map<number, number>();
    visible.forEach((orig, vi) => map.set(orig, vi));

    const labels = new Array<string>(visible.length);
    const data = new Array<{ x: number; y: number }>(visible.length);
    const radii = new Array<number>(visible.length);
    const colors = new Array<string>(visible.length);
    const borders = new Array<string>(visible.length);
    const bw = new Array<number>(visible.length);
    const selectedIdx = selectedNode ? indexByUrl.get(selectedNode.url) : undefined;
    for (let vi = 0; vi < visible.length; vi++) {
      const orig = visible[vi];
      const selected = orig === selectedIdx;
      labels[vi] = urls[orig];
      data[vi] = { x: posCache[orig * 2], y: posCache[orig * 2 + 1] };
      radii[vi] = baseSizes[orig] + (selected ? 2 : 0);
      colors[vi] = baseColors[orig];
      const expandable = (childMap.get(orig)?.length ?? 0) > 0;
      borders[vi] = selected || expandable ? highlightColor : 'transparent';
      bw[vi] = selected || expandedNodes.has(orig) ? 3 : 1.5;
    }

    const edgesOut: { source: number; target: number }[] = [];
    const edgeColorsOut: string[] = [];
    const edgeDashesOut: number[][] = [];
    for (const p of expandedNodes) {
      const kids = childMap.get(p);
      if (!kids) continue;
      for (const { c, follow } of kids) {
        const s = map.get(p);
        const t = map.get(c);
        if (s !== undefined && t !== undefined && s !== t) {
          edgesOut.push({ source: s, target: t });
          edgeColorsOut.push(follow ? followEdge : nofollowEdge);
          edgeDashesOut.push(follow ? [] : [4, 3]);
        }
      }
    }

    return {
      labels,
      data,
      edges: edgesOut,
      edgeColors: edgeColorsOut,
      edgeDashes: edgeDashesOut,
      radii,
      colors,
      borders,
      bw,
      visible,
    };
  }

  function applyData(c: Chart) {
    const vd = getVisibleData();
    lastVisibleIdx = vd.visible;
    const ds = c.data.datasets[0];
    c.data.labels = vd.labels;
    (ds as any).data = vd.data;
    (ds as any).__edges = vd.edges;
    (ds as any).__edgeColors = vd.edgeColors;
    (ds as any).__edgeDashes = vd.edgeDashes;
    (ds as any).pointRadius = vd.radii;
    (ds as any).pointBackgroundColor = vd.colors;
    (ds as any).pointBorderColor = vd.borders;
    (ds as any).pointBorderWidth = vd.bw;
    (ds as any).pointHoverRadius = vd.radii.map((r) => r + 2);
    c.update();
  }

  function computeBounds(): { minX: number; maxX: number; minY: number; maxY: number } {
    let minX = Infinity;
    let maxX = -Infinity;
    let minY = Infinity;
    let maxY = -Infinity;
    for (let i = 0; i < urls.length; i++) {
      const x = posCache[i * 2];
      const y = posCache[i * 2 + 1];
      if (!Number.isFinite(x) || !Number.isFinite(y)) continue;
      if (x < minX) minX = x;
      if (x > maxX) maxX = x;
      if (y < minY) minY = y;
      if (y > maxY) maxY = y;
    }
    if (!Number.isFinite(minX)) return { minX: -1, maxX: 1, minY: -1, maxY: 1 };
    const padX = Math.max(1, (maxX - minX) * 0.05);
    const padY = Math.max(1, (maxY - minY) * 0.05);
    return { minX: minX - padX, maxX: maxX + padX, minY: minY - padY, maxY: maxY + padY };
  }

  function fitView() {
    const c = chart;
    if (!c || urls.length === 0) return;
    if (typeof (c as any).resetZoom === 'function') (c as any).resetZoom();
  }

  function focusOnNode(origIdx: number, factor: number) {
    const c = chart;
    if (!c) return;
    const vd = getVisibleData();
    const vi = vd.visible.indexOf(origIdx);
    if (vi === -1) return;
    const p = vd.data[vi];
    if (typeof (c as any).zoom !== 'function') return;
    (c as any).zoom({ x: factor, y: factor, focalPoint: { x: p.x, y: p.y } });
  }

  function focusUrl(url: string) {
    const idx = indexByUrl.get(url);
    if (idx === undefined) return;
    focusOnNode(idx, 3);
  }

  function selectNode(idx: number) {
    const nd = graph?.nodes[idx];
    if (!nd) return;
    selectedNode = nd;
    hoveredUrl = null;
  }

  function clearSelection() {
    selectedNode = null;
  }

  // Computes which nodes are currently visible in the levels layout and their
  // tree level (0 for roots). Visibility flows from the root level through the
  // expanded nodes only, so collapsing a parent also hides its subtree.
  function levelTree(): { visible: Set<number>; treeLevel: Int32Array } {
    const n = urls.length;
    const visible = new Set<number>();
    const treeLevel = new Int32Array(n).fill(-1);
    const queue: number[] = [];
    for (let i = 0; i < n; i++) {
      if (meta[i] && meta[i].depth === levelRootDepth) {
        visible.add(i);
        treeLevel[i] = 0;
        queue.push(i);
      }
    }
    for (let qi = 0; qi < queue.length; qi++) {
      const p = queue[qi];
      if (!expandedNodes.has(p)) continue;
      const kids = childMap.get(p);
      if (!kids) continue;
      for (const { c } of kids) {
        if (treeLevel[c] !== -1) continue;
        treeLevel[c] = treeLevel[p] + 1;
        visible.add(c);
        queue.push(c);
      }
    }
    return { visible, treeLevel };
  }

  function toggleExpand(idx: number) {
    if (expandedNodes.has(idx)) expandedNodes.delete(idx);
    else expandedNodes.add(idx);
    applyLayout(layoutId);
  }

  function collapseLevels() {
    if (expandedNodes.size === 0) return;
    expandedNodes.clear();
    applyLayout(layoutId);
  }

  function handleChartClick(_ev: ChartEvent, elements: ActiveElement[], _c: Chart) {
    const pt = elements.find((e) => e.element instanceof PointElement);
    const idx = pt?.index;
    if (idx === undefined) return;
    const orig = lastVisibleIdx[idx];
    if (orig === undefined) return;
    if ((childMap.get(orig)?.length ?? 0) > 0) {
      toggleExpand(orig);
      return;
    }
    selectNode(orig);
  }

  function handleChartHover(_ev: ChartEvent, elements: ActiveElement[], _c: Chart) {
    const pt = elements.find((e) => e.element instanceof PointElement);
    const orig = pt?.index !== undefined ? lastVisibleIdx[pt.index] : undefined;
    hoveredUrl = orig !== undefined ? (urls[orig] ?? null) : null;
    _c.canvas.style.cursor = orig !== undefined ? 'pointer' : 'default';
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

  function computePositions(id: LayoutId): Float32Array | null {
    const data = graph;
    if (!data) return null;
    const order = data.nodes.length;
    const positions = new Float32Array(order * 2);
    const spacing = 30;
    // Layouts operate on the currently visible subset (root level + expanded
    // subtrees); hidden nodes stay NaN so the bounds ignore them.
    const { visible } = levelTree();
    const vis: number[] = [];
    for (let i = 0; i < order; i++) if (visible.has(i)) vis.push(i);
    const vcount = vis.length;

    if (id === 'grid') {
      const cols = Math.ceil(Math.sqrt(vcount));
      for (let vi = 0; vi < vcount; vi++) {
        const idx = vis[vi];
        positions[idx * 2] = (vi % cols) * spacing;
        positions[idx * 2 + 1] = Math.floor(vi / cols) * spacing;
      }
    } else if (id === 'circle') {
      const radius = (Math.max(1, vcount) * spacing) / (2 * Math.PI);
      for (let vi = 0; vi < vcount; vi++) {
        const a = (vi / Math.max(1, vcount)) * Math.PI * 2;
        const idx = vis[vi];
        positions[idx * 2] = radius * Math.cos(a);
        positions[idx * 2 + 1] = radius * Math.sin(a);
      }
    } else if (id === 'concentric') {
      const ordered = vis
        .slice()
        .sort((a, b) => (meta[b]?.degree ?? 0) - (meta[a]?.degree ?? 0));
      let ring = 0;
      let vi = 0;
      while (vi < ordered.length) {
        const count = ring === 0 ? 1 : 8 * ring;
        for (let j = 0; j < count && vi < ordered.length; j++, vi++) {
          const a = (j / count) * Math.PI * 2;
          const idx = ordered[vi];
          positions[idx * 2] = ring * spacing * 1.6 * Math.cos(a);
          positions[idx * 2 + 1] = ring * spacing * 1.6 * Math.sin(a);
        }
        ring++;
      }
    } else if (id === 'breadthfirst') {
      const depthOf = (i: number) => meta[i]?.depth ?? 0;
      const depths = [...new Set(vis.map(depthOf))].sort((a, b) => a - b);
      depths.forEach((d, di) => {
        const col: number[] = [];
        for (let vi = 0; vi < vcount; vi++) if (depthOf(vis[vi]) === d) col.push(vis[vi]);
        col.forEach((idx, j) => {
          positions[idx * 2] = di * spacing * 2.4;
          positions[idx * 2 + 1] = (j - (col.length - 1) / 2) * spacing;
        });
      });
    } else if (id === 'force') {
      // d3-force simulation over the visible subset. Links mirror the rendered
      // edges (expanded subtrees). Nodes are seeded on a golden-angle spiral so
      // repeated runs converge to a stable layout instead of exploding.
      const simNodes = vis.map((orig) => ({ orig }));
      const indexOf = new Map<number, number>();
      vis.forEach((orig, vi) => indexOf.set(orig, vi));
      const links: Array<{ source: number; target: number }> = [];
      for (const p of expandedNodes) {
        const kids = childMap.get(p);
        if (!kids) continue;
        const s = indexOf.get(p);
        if (s === undefined) continue;
        for (const { c } of kids) {
          const t = indexOf.get(c);
          if (t === undefined) continue;
          links.push({ source: s, target: t });
        }
      }
      const golden = Math.PI * (3 - Math.sqrt(5));
      for (let vi = 0; vi < vcount; vi++) {
        const a = vi * golden;
        const r = Math.sqrt(vi + 1) * 12;
        simNodes[vi].x = Math.cos(a) * r;
        simNodes[vi].y = Math.sin(a) * r;
      }
      const sim = forceSimulation(simNodes as any)
        .force('charge', forceManyBody().strength(-40))
        .force('link', forceLink(links).distance(26).strength(0.5))
        .force('center', forceCenter(0, 0))
        .force('collide', forceCollide().radius(8));
      sim.stop();
      const iterations = vcount > 800 ? 60 : 200;
      for (let it = 0; it < iterations; it++) sim.tick();
      for (let vi = 0; vi < vcount; vi++) {
        const idx = vis[vi];
        positions[idx * 2] = simNodes[vi].x;
        positions[idx * 2 + 1] = simNodes[vi].y;
      }
    }

    for (let i = 0; i < order; i++) {
      if (!visible.has(i)) {
        positions[i * 2] = NaN;
        positions[i * 2 + 1] = NaN;
      }
    }

    // Scale the computed layout so it fits inside the plot area.
    let maxAbs = 1;
    for (let i = 0; i < order; i++) {
      const px = positions[i * 2];
      const py = positions[i * 2 + 1];
      if (Number.isFinite(px)) maxAbs = Math.max(maxAbs, Math.abs(px));
      if (Number.isFinite(py)) maxAbs = Math.max(maxAbs, Math.abs(py));
    }
    const s = ((SPACE_SIZE / 2) * 0.9) / maxAbs;
    for (let i = 0; i < order; i++) {
      positions[i * 2] *= s;
      positions[i * 2 + 1] *= s;
    }
    return positions;
  }

  function applyLayout(id: LayoutId) {
    const c = chart;
    if (!c || urls.length === 0 || !graph) return;
    if (layouting) return;
    layouting = true;
    try {
      const computed = computePositions(id);
      if (!computed) return;
      for (let i = 0; i < urls.length; i++) {
        posCache[i * 2] = computed[i * 2];
        posCache[i * 2 + 1] = computed[i * 2 + 1];
      }
      const bounds = computeBounds();
      const scales = (c.options as any)?.scales;
      if (scales) {
        scales.x.min = bounds.minX;
        scales.x.max = bounds.maxX;
        scales.y.min = bounds.minY;
        scales.y.max = bounds.maxY;
      }
      const zoomOpts = (c.options as any)?.plugins?.zoom;
      if (zoomOpts) {
        zoomOpts.limits = {
          x: { min: bounds.minX, max: bounds.maxX },
          y: { min: bounds.minY, max: bounds.maxY },
        };
      }
      dataTick++;
    } finally {
      layouting = false;
    }
  }

  // Native Chart.js has no edge primitive: links are drawn as canvas lines in
  // front of the background but behind the node points.
  const edgePlugin = {
    id: 'graphEdges',
    beforeDatasetsDraw(c: Chart) {
      const ds = c.data.datasets[0] as any;
      const edges: Array<{ source: number; target: number }> = ds?.__edges ?? [];
      if (!edges.length) return;
      const colors: string[] = ds?.__edgeColors ?? [];
      const dashes: number[][] = ds?.__edgeDashes ?? [];
      const metaData = (c.getDatasetMeta(0) as any).data ?? [];
      const ctx = c.ctx;
      ctx.save();
      ctx.lineWidth = 1;
      for (let k = 0; k < edges.length; k++) {
        const e = edges[k];
        const p1 = metaData[e.source];
        const p2 = metaData[e.target];
        if (!p1 || !p2 || !Number.isFinite(p1.x) || !Number.isFinite(p2.x)) continue;
        ctx.strokeStyle = colors[k] ?? 'rgba(148, 163, 184, 0.55)';
        ctx.setLineDash(dashes[k] ?? []);
        ctx.beginPath();
        ctx.moveTo(p1.x, p1.y);
        ctx.lineTo(p2.x, p2.y);
        ctx.stroke();
      }
      ctx.setLineDash([]);
      ctx.restore();
    },
  };

  function buildChart(canvas: HTMLCanvasElement) {
    const computed = computePositions(layoutId);
    if (computed) {
      for (let i = 0; i < urls.length; i++) {
        posCache[i * 2] = computed[i * 2];
        posCache[i * 2 + 1] = computed[i * 2 + 1];
      }
    }
    const vd = getVisibleData();
    const bounds = computeBounds();

    const dataset: Record<string, unknown> = {
      data: vd.data,
      __edges: vd.edges,
      __edgeColors: vd.edgeColors,
      __edgeDashes: vd.edgeDashes,
      pointStyle: 'circle',
      pointRadius: vd.radii,
      pointBackgroundColor: vd.colors,
      pointBorderColor: vd.borders,
      pointBorderWidth: vd.bw,
      pointHoverRadius: vd.radii.map((r) => r + 2),
      pointHitRadius: 14,
    };

    const options: Record<string, unknown> = {
      responsive: true,
      maintainAspectRatio: false,
      animation: false,
      onClick: handleChartClick,
      onHover: handleChartHover,
      layout: { padding: 6 },
      scales: {
        x: { type: 'linear', display: false, min: bounds.minX, max: bounds.maxX },
        y: { type: 'linear', display: false, min: bounds.minY, max: bounds.maxY },
      },
      plugins: {
        legend: { display: false },
        tooltip: { enabled: false },
        zoom: {
          pan: { enabled: true, mode: 'xy', threshold: 5 },
          zoom: { wheel: { enabled: true, speed: 0.08 }, pinch: { enabled: true }, mode: 'xy' },
          limits: {
            x: { min: bounds.minX, max: bounds.maxX },
            y: { min: bounds.minY, max: bounds.maxY },
          },
        },
        datalabels: {
          display: (ctx: any) => {
            if (!showLabels) return false;
            const orig = lastVisibleIdx[ctx.dataIndex];
            if (orig == null) return false;
            // Only the (small) expanded subset is ever drawn, so labels always fit.
            return true;
          },
          anchor: 'end',
          align: 'top',
          offset: 2,
          formatter: (ctx: any) => {
            const orig = lastVisibleIdx[ctx.dataIndex];
            return orig != null ? shortLabel(meta[orig]?.title ?? urls[orig] ?? '') : '';
          },
          color: resolveColor(cssVar('--text', '#e1e3e6')),
          font: { size: 9, weight: 600 },
        },
      },
    };

    const plugins = [zoomPlugin, DatalabelsPlugin, edgePlugin];
    chart = new Chart(canvas, {
      type: 'scatter',
      data: { labels: vd.labels, datasets: [dataset] },
      options,
      plugins,
    } as any);
  }

  $effect(() => {
    const data = graph;
    if (!data) return;
    const nodes = data.nodes;
    const n = nodes.length;

    let maxDegree = 1;
    for (const nd of nodes) {
      maxDegree = Math.max(maxDegree, nd.in_degree + nd.out_degree);
    }
    let minDepth = Infinity;
    for (const nd of nodes) {
      if (nd.depth < minDepth) minDepth = nd.depth;
    }
    levelRootDepth = Number.isFinite(minDepth) ? minDepth : 0;
    const spread = SPACE_SIZE * 0.3;
    urls = new Array(n);
    indexByUrl = new Map();
    meta = new Array(n);
    const positions = new Float32Array(n * 2);
    const sizes = new Float32Array(n);
    const colors = new Array<string>(n);
    // Golden-angle spiral: deterministic and instantly structured (nodes come
    // pre-sorted by degree, so the hub lands near the center).
    const goldenAngle = Math.PI * (3 - Math.sqrt(5));
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
        depth: nd.depth,
      };
      const radius = spread * 1.6 * Math.sqrt(i / Math.max(1, n));
      const angle = i * goldenAngle;
      positions[i * 2] = radius * Math.cos(angle);
      positions[i * 2 + 1] = radius * Math.sin(angle);
      sizes[i] = 4 + 14 * Math.sqrt(degree / maxDegree);
      colors[i] = nodeFill[sc];
    }
    posCache = positions;
    baseSizes = sizes;
    baseColors = colors;
    labelTopSet = new Set(
      urls
        .map((_, i) => i)
        .sort((a, b) => (meta[b]?.degree ?? 0) - (meta[a]?.degree ?? 0))
        .slice(0, MAX_LABELS)
    );
    selectedNode = null;
    hoveredUrl = null;
    lastVisibleIdx = [];
    expandedNodes.clear();
  });

  $effect(() => {
    void edges;
    // Parent -> children map (deduplicated) used by the levels layout.
    childMap = new Map();
    const seenChild = new Map<number, Set<number>>();
    for (const e of edges) {
      const s = indexByUrl.get(e.source);
      const t = indexByUrl.get(e.target);
      if (s === undefined || t === undefined || s === t) continue;
      let seen = seenChild.get(s);
      if (!seen) {
        seen = new Set();
        seenChild.set(s, seen);
      }
      if (seen.has(t)) continue;
      seen.add(t);
      let arr = childMap.get(s);
      if (!arr) {
        arr = [];
        childMap.set(s, arr);
      }
      arr.push({ c: t, follow: e.is_follow });
    }
  });

  $effect(() => {
    const canvas = canvasEl;
    const data = graph;
    const ready = !loading;
    void themeTick;
    void layoutId;
    if (!canvas || !data || !ready) {
      chart?.destroy();
      chart = null;
      return;
    }

    // Large graphs default to a deterministic layout instead of the force
    // simulation, so the tab stays responsive on big sites.
    if (layoutId === 'force' && data.nodes.length > FORCE_SIM_MAX_NODES) {
      layoutId = 'concentric';
      return;
    }

    buildChart(canvas);
    untrack(() => {
      dataTick++;
    });

    return () => {
      chart?.destroy();
      chart = null;
    };
  });

  $effect(() => {
    void dataTick;
    const c = chart;
    if (!c) return;
    applyData(c);
  });

  $effect(() => {
    void siteMapFilters.status;
    void siteMapFilters.search;
    void languageFilter;
    void selectedNode;
    void showLabels;
    untrack(() => {
      dataTick++;
    });
  });

  $effect(() => {
    const q = searchInput;
    if (debounceTimer) clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => {
      if (siteMapFilters.search !== q) siteMapFilters.search = q;
    }, 150);
    return () => {
      if (debounceTimer) clearTimeout(debounceTimer);
    };
  });

  $effect(() => {
    void siteMapFilters.search;
    searchInput = siteMapFilters.search;
  });

  $effect(() => {
    const unwatch = watchTheme(() => themeTick++);
    return unwatch;
  });

  // Cross-navigation: when the site tree asks to show a page in the graph,
  // reset filters so the node is visible, select it and zoom the camera.
  function tryFocusPending() {
    const url = pendingFocusUrl;
    if (!url) return;
    const idx = indexByUrl.get(url);
    if (idx === undefined) return;
    pendingFocusUrl = null;
    resetSiteMapFilters();
    hoveredUrl = null;
    selectNode(idx);
    requestAnimationFrame(() => focusOnNode(idx, 3));
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
    if (chart && graph) tryFocusPending();
  });

  onDestroyLifecycle(() => {
    chart?.destroy();
    chart = null;
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

      // Edge pages are served from the in-memory cache after the first request,
      // so fetching them in parallel beats a serial loop.
      const pageSize = 20_000;
      const first = await getSiteGraphEdges(projectId, 0, pageSize);
      if (seq !== graphSeq) return;

      let all: SiteGraphEdge[] = first.edges;
      const remaining = Math.max(0, first.total - first.edges.length);
      const pageCount = Math.min(
        Math.ceil(remaining / pageSize),
        Math.ceil(MAX_EDGES / pageSize) - 1
      );
      if (pageCount > 0) {
        const pages = await Promise.all(
          Array.from({ length: pageCount }, (_, i) =>
            getSiteGraphEdges(projectId, (i + 1) * pageSize, pageSize)
          )
        );
        if (seq !== graphSeq) return;
        const chunks: SiteGraphEdge[][] = [];
        for (const p of pages) {
          if (p.edges.length === 0) break;
          chunks.push(p.edges);
        }
        all = all.concat(...chunks);
      }

      edges = all.length > MAX_EDGES ? all.slice(0, MAX_EDGES) : all;
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
</script>

<div class="site-graph">
  <div class="graph-head">
    <div class="flex items-center gap-2 text-sm font-semibold">
      <GitBranch class="size-4" />
      {m['graph.title']()}
      <span class="engine-badge">Chart.js</span>
      {#if graph}
        <span class="text-xs font-normal text-muted-foreground">
          {m['graph.pages']()}: {graph.total_nodes.toLocaleString()} · {m['graph.edges']()}:
          {graph.edge_count.toLocaleString()}
        </span>
      {/if}
    </div>
    <div class="graph-tools">
      <div class="graph-search">
        <Search class="size-3.5 text-muted-foreground" />
        <Input
          type="text"
          bind:value={searchInput}
          placeholder={m['graph.search_placeholder']()}
          class="h-7"
        />
        {#if searchInput}
          <button
            class="graph-search-clear"
            onclick={() => (searchInput = '')}
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
        onclick={collapseLevels}
        disabled={expandedNodes.size === 0}
        title="Collapse all levels"
      >
        <ListCollapse class="size-3.5" />
        Collapse all
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
        <div class="graph-canvas">
          <canvas bind:this={canvasEl}></canvas>
        </div>
        <div class="graph-hint">
          {m['graph.select_hint']()}
          <span class="graph-hint-secondary">· Click a node with children to expand it</span>
        </div>
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
    position: absolute;
    inset: 0;
    width: 100% !important;
    height: 100% !important;
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
