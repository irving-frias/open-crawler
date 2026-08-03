<script lang="ts">
  let {
    items = [],
    width = 520,
    barHeight = 16,
    gap = 10,
    labelWidth = 160,
    maxLabelChars = 28,
    defaultColor = 'var(--accent)',
  }: {
    items: { label: string; value: number; color?: string }[];
    width?: number;
    barHeight?: number;
    gap?: number;
    labelWidth?: number;
    maxLabelChars?: number;
    defaultColor?: string;
  } = $props();

  let max = $derived(Math.max(1, ...items.map((i) => i.value)));
  let height = $derived(items.length > 0 ? items.length * (barHeight + gap) : 0);
  let barArea = $derived(width - labelWidth - 12);
  let rowHeight = $derived(barHeight + gap);

  function truncate(label: string): string {
    return label.length > maxLabelChars ? `${label.slice(0, maxLabelChars)}…` : label;
  }

  function formatValue(value: number): string {
    return value.toLocaleString();
  }
</script>

{#if items.length > 0}
  <svg width={width} height={height} viewBox="0 0 {width} {height}" role="img" class="barchart">
    {#each items as item, i (item.label)}
      {@const y = i * rowHeight}
      {@const barW = (item.value / max) * barArea}
      <text x={0} y={y + barHeight * 0.7} class="bar-label" text-anchor="start">
        {truncate(item.label)}
      </text>
      <rect
        x={labelWidth + 4}
        y={y}
        width={Math.max(barW, item.value > 0 ? 2 : 0)}
        height={barHeight}
        rx={4}
        fill={item.color || defaultColor}
      />
      <text x={labelWidth + 8 + barW + 4} y={y + barHeight * 0.7} class="bar-value" text-anchor="start">
        {formatValue(item.value)}
      </text>
    {/each}
  </svg>
{/if}

<style>
  svg {
    display: block;
    max-width: 100%;
    height: auto;
  }
  .bar-label {
    fill: var(--text-secondary);
    font-size: 12px;
  }
  .bar-value {
    fill: var(--text-muted);
    font-size: 11px;
  }
</style>
