<script lang="ts">
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

  let total = $derived(segments.reduce((sum, s) => sum + s.value, 0));
  let center = $derived(size / 2);
  let radius = $derived((size - thickness) / 2);
  let circumference = $derived(2 * Math.PI * radius);

  let arcs = $derived.by(() => {
    let cumulative = 0;
    return segments
      .filter((s) => s.value > 0)
      .map((s) => {
        const frac = total > 0 ? s.value / total : 0;
        const arc = {
          ...s,
          dash: frac * circumference,
          offset: -cumulative * circumference,
        };
        cumulative += frac;
        return arc;
      });
  });
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 {size} {size}"
  role="img"
  aria-label={centerLabel || 'chart'}
  class="donut"
>
  <circle
    cx={center}
    cy={center}
    r={radius}
    fill="none"
    stroke="var(--border)"
    stroke-width={thickness}
  />
  {#each arcs as arc, i}
    <circle
      cx={center}
      cy={center}
      r={radius}
      fill="none"
      stroke={arc.color}
      stroke-width={thickness}
      stroke-dasharray={`${arc.dash} ${circumference - arc.dash}`}
      stroke-dashoffset={arc.offset}
      transform={`rotate(-90 ${center} ${center})`}
      class="donut-segment"
      style={i === 0 ? '' : 'transition: stroke-dasharray 0.4s ease'}
    >
      <title>{arc.label}: {arc.value}</title>
    </circle>
  {/each}
  {#if centerLabel || centerValue}
    <text x="50%" y="46%" text-anchor="middle" dominant-baseline="middle" class="donut-value">
      {centerValue}
    </text>
    <text x="50%" y="58%" text-anchor="middle" dominant-baseline="middle" class="donut-label">
      {centerLabel}
    </text>
  {/if}
</svg>

<style>
  svg {
    display: block;
  }
  .donut-segment {
    transition: stroke-dasharray 0.4s ease;
  }
  .donut-value {
    fill: var(--text);
    font-size: 20px;
    font-weight: 700;
  }
  .donut-label {
    fill: var(--text-muted);
    font-size: 11px;
  }
</style>
