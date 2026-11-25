<script lang="ts">
  import { m } from '#lib/paraglide/messages.js';
  import { Badge } from '#lib/components/ui/badge/index.js';
  import { severityMeta } from '#lib/severity-ui.ts';

  let {
    severity,
    count,
    label,
  }: {
    /** One of error | warning | info | ok (unknown values fall back to info). */
    severity: string;
    /** Optional count shown after the label. */
    count?: number;
    /** Optional override for the label text. */
    label?: string;
  } = $props();

  const meta = $derived(severityMeta(severity));
</script>

<Badge variant={meta.variant}>
  <span aria-hidden="true" class="severity-badge-icon">{meta.icon}</span>
  {label ?? m[meta.labelKey]()}
  {#if count !== undefined}
    <span class="severity-badge-count">{count}</span>
  {/if}
</Badge>

<style>
  .severity-badge-icon {
    font-weight: 700;
    font-size: 0.7rem;
    line-height: 1;
  }

  .severity-badge-count {
    font-variant-numeric: tabular-nums;
    font-weight: 700;
  }
</style>
