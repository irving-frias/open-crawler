<script lang="ts">
  import { Chart } from './setup.js';
  import { getChartTheme, resolveColor, watchTheme } from './chart-theme.js';

  let {
    items = [],
    width = 520,
    barHeight = 16,
    gap = 10,
    maxLabelChars = 28,
    defaultColor = '',
  }: {
    items: { label: string; value: number; color?: string }[];
    width?: number;
    barHeight?: number;
    gap?: number;
    maxLabelChars?: number;
    defaultColor?: string;
  } = $props();

  let canvas = $state<HTMLCanvasElement | null>(null);
  let themeTick = $state(0);
  let chart: Chart<'bar'> | null = null;

  const height = $derived(items.length > 0 ? items.length * (barHeight + gap) : 0);

  function truncate(label: string): string {
    return label.length > maxLabelChars ? `${label.slice(0, maxLabelChars)}…` : label;
  }

  $effect(() => {
    const unwatch = watchTheme(() => themeTick++);
    return unwatch;
  });

  $effect(() => {
    items;
    themeTick;
    if (!canvas || items.length === 0) return;
    const theme = getChartTheme();
    const fallback = resolveColor(defaultColor) || theme.accent;

    chart?.destroy();
    chart = new Chart(canvas, {
      type: 'bar',
      data: {
        labels: items.map((i) => truncate(i.label)),
        datasets: [
          {
            data: items.map((i) => i.value),
            backgroundColor: items.map((i) => resolveColor(i.color) || fallback),
            borderRadius: 4,
            barThickness: barHeight,
            maxBarThickness: barHeight,
            barPercentage: barHeight / (barHeight + gap),
            categoryPercentage: 1,
          },
        ],
      },
      options: {
        indexAxis: 'y',
        responsive: true,
        maintainAspectRatio: false,
        plugins: {
          legend: { display: false },
          tooltip: {
            backgroundColor: theme.tooltipBg,
            borderColor: theme.tooltipBorder,
            borderWidth: 1,
            titleColor: theme.text,
            bodyColor: theme.textSecondary,
            boxWidth: 8,
            boxHeight: 8,
            padding: 10,
            cornerRadius: 8,
            callbacks: {
              title: (tooltipItems) => {
                const idx = tooltipItems[0]?.dataIndex;
                return idx != null && items[idx] ? items[idx].label : '';
              },
              label: (ctx) => ` ${Number(ctx.raw).toLocaleString()}`,
            },
          },
        },
        scales: {
          x: {
            beginAtZero: true,
            grid: { color: theme.grid },
            border: { display: false },
            ticks: {
              color: theme.textMuted,
              padding: 6,
              callback: (value) => Number(value).toLocaleString(),
            },
          },
          y: {
            grid: { display: false },
            border: { display: false },
            ticks: {
              color: theme.textSecondary,
              font: { size: 12 },
              padding: 8,
              autoSkip: false,
            },
          },
        },
      },
    });

    return () => {
      chart?.destroy();
      chart = null;
    };
  });
</script>

{#if items.length > 0}
  <div class="bar-wrap" style={`max-width: ${width}px; height: ${height}px`}>
    <canvas bind:this={canvas}>Bar chart</canvas>
  </div>
{/if}

<style>
  .bar-wrap {
    position: relative;
    width: 100%;
  }

  canvas {
    position: absolute;
    inset: 0;
    width: 100% !important;
    height: 100% !important;
  }
</style>
