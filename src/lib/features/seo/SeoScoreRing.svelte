<script lang="ts">
  import { seoGrade, seoScoreColor } from '#lib/seo-ui.ts';

  let {
    score,
    grade,
    size = 120,
  }: {
    /** Overall SEO score, 0–100. */
    score: number;
    /** Optional grade letter; computed from the score when omitted. */
    grade?: string | null;
    /** Diameter in px. */
    size?: number;
  } = $props();

  const clamped = $derived(Math.max(0, Math.min(100, Math.round(score))));
  const gradeLetter = $derived(grade ?? seoGrade(clamped));
  const color = $derived(seoScoreColor(clamped));
  const radius = $derived(size / 2 - 8);
  const circumference = $derived(2 * Math.PI * radius);
  const dash = $derived((clamped / 100) * circumference);
</script>

<div
  class="seo-score-ring"
  style="--ring-size: {size}px"
  role="img"
  aria-label="SEO score {clamped} / 100, grade {gradeLetter}"
>
  <svg width={size} height={size} viewBox="0 0 {size} {size}" aria-hidden="true">
    <circle
      class="seo-score-ring-track"
      cx={size / 2}
      cy={size / 2}
      r={radius}
      fill="none"
      stroke-width="9"
    />
    <circle
      class="seo-score-ring-value"
      cx={size / 2}
      cy={size / 2}
      r={radius}
      fill="none"
      stroke={color}
      stroke-width="9"
      stroke-linecap="round"
      stroke-dasharray="{dash} {circumference}"
      transform="rotate(-90 {size / 2} {size / 2})"
    />
  </svg>
  <div class="seo-score-ring-center">
    <span class="seo-score-ring-value-text" style="color: {color}">{clamped}</span>
    <span class="seo-score-ring-grade">{gradeLetter}</span>
  </div>
</div>

<style>
  .seo-score-ring {
    position: relative;
    width: var(--ring-size);
    height: var(--ring-size);
    flex-shrink: 0;
  }

  .seo-score-ring-track {
    stroke: var(--muted, rgba(128, 128, 128, 0.25));
  }

  .seo-score-ring-value {
    transition: stroke-dasharray 0.4s ease;
  }

  .seo-score-ring-center {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 0;
    pointer-events: none;
  }

  .seo-score-ring-value-text {
    font-size: calc(var(--ring-size) * 0.26);
    font-weight: 800;
    line-height: 1;
    font-variant-numeric: tabular-nums;
  }

  .seo-score-ring-grade {
    font-size: calc(var(--ring-size) * 0.13);
    font-weight: 700;
    color: var(--muted-foreground);
    line-height: 1.2;
  }
</style>
