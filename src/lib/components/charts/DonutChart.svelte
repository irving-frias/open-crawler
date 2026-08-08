<script lang="ts">
  import { Chart, type Plugin } from './setup.js';
  import { getChartTheme, resolveColor, fontFamily, watchTheme } from './chart-theme.js';

  let {
    segments = [],
    size = 170,
    thickness = 20,
    centerLabel = '',
    centerValue = '',
  }: {
    segments: { label: string; value: number; color: string }[];
    size?: number;
    thickness?: number;
    centerLabel?: string;
    centerValue?: string;
  } = $props();

  let canvas = $state<HTMLCanvasElement | null>(null);
  let themeTick = $state(0);
  let chart: Chart<'doughnut'> | null = null;

  $effect(() => {
    const unwatch = watchTheme(() => themeTick++);
    return unwatch;
  });

  $effect(() => {
    void themeTick;
    if (!canvas) return;
    const theme = getChartTheme();
    const cutout = ((size - 2 * thickness) / size) * 100;

    const centerText: Plugin<'doughnut'> = {
      id: 'centerText',
      afterDraw(chart) {
        if (!centerValue && !centerLabel) return;
        const { ctx, chartArea } = chart;
        ctx.save();
        ctx.textAlign = 'center';
        ctx.textBaseline = 'middle';
        ctx.fillStyle = theme.text;
        if (centerValue) {
          ctx.font = `700 ${Math.round(size / 8.5)}px ${fontFamily()}`;
          ctx.fillText(
            centerValue,
            (chartArea.left + chartArea.right) / 2,
            chartArea.top + chartArea.height * 0.38
          );
        }
        if (centerLabel) {
          ctx.fillStyle = theme.textMuted;
          ctx.font = `400 ${Math.round(size / 16)}px ${fontFamily()}`;
          ctx.fillText(
            centerLabel,
            (chartArea.left + chartArea.right) / 2,
            chartArea.top + chartArea.height * 0.62
          );
        }
        ctx.restore();
      },
    };

    chart?.destroy();
    chart = new Chart(canvas, {
      type: 'doughnut',
      data: {
        labels: segments.map((s) => s.label),
        datasets: [
          {
            data: segments.map((s) => s.value),
            backgroundColor: segments.map((s) => resolveColor(s.color)),
            borderWidth: 0,
            hoverOffset: 4,
          },
        ],
      },
      options: {
        responsive: true,
        maintainAspectRatio: false,
        cutout: `${cutout.toFixed(1)}%`,
        animation: { animateRotate: true, duration: 600 },
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
            displayColors: true,
            padding: 10,
            cornerRadius: 8,
            callbacks: {
              label: (ctx) => ` ${ctx.label}: ${Number(ctx.raw).toLocaleString()}`,
            },
          },
        },
      },
      plugins: [centerText],
    });

    return () => {
      chart?.destroy();
      chart = null;
    };
  });
</script>

<div class="donut-wrap" style={`width: ${size}px; height: ${size}px`}>
  <canvas bind:this={canvas}>{centerLabel || 'chart'}</canvas>
</div>

<style>
  .donut-wrap {
    position: relative;
    flex-shrink: 0;
  }

  canvas {
    position: absolute;
    inset: 0;
    width: 100% !important;
    height: 100% !important;
  }
</style>
