<script lang="ts">
  import { m } from '$lib/paraglide/messages.js';
  import { SlidersHorizontal, ChevronDown, CircleX, TriangleAlert, Info } from '@lucide/svelte';
  import * as Popover from '$lib/components/ui/popover/index.js';
  import { Button } from '$lib/components/ui/button/index.js';
  import { Badge } from '$lib/components/ui/badge/index.js';
  import { cn } from '$lib/utils.js';
  import { translateIssueName } from '$lib/i18n-issues';
  import { parseIssues } from '$lib/features/results/issue-cache';

  let {
    items,
    filters,
    onFilter,
  }: {
    items: any[];
    filters: FilterState;
    onFilter: (filters: FilterState) => void;
  } = $props();

  export type FilterState = {
    statusCodes: number[];
    severities: string[];
    depth: number | undefined;
    missingTitle: boolean;
    duplicateTitle: boolean;
    noindexOnly: boolean;
    is404: boolean;
    issueType: string;
  };

  const selectedStatuses = $derived(filters.statusCodes);
  const selectedSeverities = $derived(filters.severities);
  const maxDepth = $derived(filters.depth);
  const missingTitle = $derived(filters.missingTitle);
  const duplicateTitle = $derived(filters.duplicateTitle);
  const noindexOnly = $derived(filters.noindexOnly);
  const is404 = $derived(filters.is404);
  const issueType = $derived(filters.issueType);

  const AVAILABLE_STATUSES = $derived.by(() => {
    let statusCount: Record<number, number> = {};
    for (const page of items) {
      if (page.status_code) {
        statusCount[page.status_code] = (statusCount[page.status_code] || 0) + 1;
      }
    }
    return Object.entries(statusCount)
      .map(([code, _]) => parseInt(code))
      .sort((a, b) => a - b);
  });

  const availableSeverities = $derived.by(() => {
    let sevCount: Record<string, number> = {};
    for (const page of items) {
      for (const issue of parseIssues(page.semantic_issues_json)) {
        if (issue.severity) {
          sevCount[issue.severity] = (sevCount[issue.severity] || 0) + 1;
        }
      }
    }
    return Object.keys(sevCount).sort();
  });

  const availableIssueTypes = $derived.by(() => {
    const types = new Set<string>();
    for (const page of items) {
      for (const issue of parseIssues(page.semantic_issues_json)) {
        if (issue.issue_type) types.add(issue.issue_type);
      }
    }
    return [...types].sort();
  });

  const activeFilterCount = $derived(
    selectedStatuses.length +
      selectedSeverities.length +
      (maxDepth !== undefined ? 1 : 0) +
      (missingTitle ? 1 : 0) +
      (duplicateTitle ? 1 : 0) +
      (noindexOnly ? 1 : 0) +
      (is404 ? 1 : 0) +
      (issueType ? 1 : 0)
  );

  const sliderPct = $derived(((maxDepth ?? 10) / 10) * 100);

  function toggleStatus(code: number) {
    const next = selectedStatuses.includes(code)
      ? selectedStatuses.filter((s) => s !== code)
      : [...selectedStatuses, code];
    onFilter({ ...filters, statusCodes: next });
  }

  function toggleSeverity(severity: string) {
    const next = selectedSeverities.includes(severity)
      ? selectedSeverities.filter((s) => s !== severity)
      : [...selectedSeverities, severity];
    onFilter({ ...filters, severities: next });
  }

  function handleDepthChange(e: Event) {
    const val = parseInt((e.target as HTMLInputElement).value);
    onFilter({ ...filters, depth: isNaN(val) ? undefined : val });
  }

  function clearAll() {
    onFilter({
      statusCodes: [],
      severities: [],
      depth: undefined,
      missingTitle: false,
      duplicateTitle: false,
      noindexOnly: false,
      is404: false,
      issueType: '',
    });
  }

  const seoChips = $derived([
    {
      key: 'missingTitle',
      label: m['filter.missing_title'](),
      state: missingTitle,
      toggle: () => onFilter({ ...filters, missingTitle: !missingTitle }),
    },
    {
      key: 'duplicateTitle',
      label: m['filter.duplicate_title'](),
      state: duplicateTitle,
      toggle: () => onFilter({ ...filters, duplicateTitle: !duplicateTitle }),
    },
    {
      key: 'noindexOnly',
      label: m['filter.noindex'](),
      state: noindexOnly,
      toggle: () => onFilter({ ...filters, noindexOnly: !noindexOnly }),
    },
    {
      key: 'is404',
      label: m['filter.is_404'](),
      state: is404,
      toggle: () => onFilter({ ...filters, is404: !is404 }),
    },
  ]);
</script>

<div class="filter-bar">
  <div class="quick-chips">
    {#each seoChips as chip (chip.label)}
      <Button
        variant="ghost"
        size="sm"
        class={cn(
          'gap-1.5 rounded-full px-3',
          chip.state
            ? 'border-primary bg-primary text-primary-foreground hover:bg-primary/80'
            : 'border-transparent bg-muted/70 text-muted-foreground hover:bg-muted hover:text-muted-foreground'
        )}
        onclick={chip.toggle}
      >
        {chip.label}
      </Button>
    {/each}
  </div>
  <Popover.Root>
    <Popover.Trigger>
      {#snippet child({ props })}
        <Button variant="outline" size="sm" class="gap-1.5" {...props}>
          <SlidersHorizontal class="size-4" />
          {m['filter.filters']()}
          {#if activeFilterCount > 0}
            <Badge variant="default" class="size-5 justify-center rounded-full px-1"
              >{activeFilterCount}</Badge
            >
          {/if}
          <ChevronDown class="size-3.5 opacity-60" />
        </Button>
      {/snippet}
    </Popover.Trigger>

    <Popover.Content class="w-96 max-w-[calc(100vw-2rem)]" align="start">
      <div class="flex flex-col gap-4">
        <div class="flex flex-col gap-2">
          <div class="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
            {m['filter.status']()}
          </div>
          <div class="flex flex-wrap gap-1.5">
            {#each AVAILABLE_STATUSES as code (code)}
              <Button
                variant="ghost"
                size="sm"
                class={cn(
                  'rounded-full px-3',
                  selectedStatuses.includes(code)
                    ? 'border-primary bg-primary text-primary-foreground hover:bg-primary/80'
                    : 'border-transparent bg-muted/70 text-muted-foreground hover:bg-muted hover:text-muted-foreground'
                )}
                onclick={() => toggleStatus(code)}
              >
                {code}
              </Button>
            {/each}
            {#if AVAILABLE_STATUSES.length === 0}
              <span class="text-xs text-muted-foreground">{m['filter.no_options']()}</span>
            {/if}
          </div>
        </div>

        <div class="flex flex-col gap-2">
          <div class="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
            {m['filter.severity']()}
          </div>
          <div class="flex flex-wrap gap-1.5">
            {#each availableSeverities as severity (severity)}
              <Button
                variant="ghost"
                size="sm"
                class={cn(
                  'gap-1.5 rounded-full border px-3 capitalize',
                  selectedSeverities.includes(severity)
                    ? 'chip-sev-active sev-{severity}'
                    : 'border-transparent bg-muted/70 text-muted-foreground hover:bg-muted hover:text-muted-foreground'
                )}
                onclick={() => toggleSeverity(severity)}
              >
                {#if severity === 'error'}
                  <CircleX class="size-3.5" />
                {:else if severity === 'warning'}
                  <TriangleAlert class="size-3.5" />
                {:else}
                  <Info class="size-3.5" />
                {/if}
                {severity}
              </Button>
            {/each}
            {#if availableSeverities.length === 0}
              <span class="text-xs text-muted-foreground">{m['filter.no_options']()}</span>
            {/if}
          </div>
        </div>

        <div class="flex flex-col gap-2">
          <div class="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
            {m['filter.issue_type']()}
          </div>
          <div class="flex flex-wrap gap-1.5">
            {#each availableIssueTypes as type (type)}
              <Button
                variant="ghost"
                size="sm"
                class={cn(
                  'rounded-full px-3',
                  issueType === type
                    ? 'border-primary bg-primary text-primary-foreground hover:bg-primary/80'
                    : 'border-transparent bg-muted/70 text-muted-foreground hover:bg-muted hover:text-muted-foreground'
                )}
                onclick={() => onFilter({ ...filters, issueType: issueType === type ? '' : type })}
              >
                {translateIssueName(type)}
              </Button>
            {/each}
            {#if availableIssueTypes.length === 0}
              <span class="text-xs text-muted-foreground">{m['filter.no_options']()}</span>
            {/if}
          </div>
        </div>

        <div class="flex flex-col gap-2">
          <div class="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
            {m['filter.depth']()}
          </div>
          <div class="depth-slider">
            <input
              type="range"
              min="0"
              max="10"
              value={maxDepth ?? 10}
              oninput={handleDepthChange}
              style="background: linear-gradient(to right, var(--accent) {sliderPct}%, var(--border) {sliderPct}%)"
            />
            <span class="depth-value">{maxDepth !== undefined ? maxDepth : '—'}</span>
          </div>
        </div>

        {#if activeFilterCount > 0}
          <Button variant="destructive" size="sm" class="w-fit" onclick={clearAll}>
            {m['filter.clear_all']()} ({activeFilterCount})
          </Button>
        {/if}
      </div>
    </Popover.Content>
  </Popover.Root>
</div>

<style>
  .filter-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    margin-bottom: 8px;
  }

  .quick-chips {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    min-width: 0;
  }

  :global(.chip-sev-active.sev-error) {
    background: var(--danger-subtle);
    border-color: var(--danger);
    color: var(--danger);
  }
  :global(.chip-sev-active.sev-warning) {
    background: var(--warning-subtle);
    border-color: var(--warning);
    color: var(--warning);
  }
  :global(.chip-sev-active.sev-info) {
    background: var(--info-subtle);
    border-color: var(--info);
    color: var(--info);
  }

  .depth-slider {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .depth-slider input[type='range'] {
    flex: 1;
    height: 6px;
    -webkit-appearance: none;
    appearance: none;
    background: var(--border);
    border-radius: 3px;
    outline: none;
    cursor: pointer;
  }

  .depth-slider input[type='range']::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: var(--accent);
    border: 2px solid var(--bg-card);
    box-shadow: var(--neu-raised-sm);
    cursor: pointer;
    transition: transform var(--transition-fast);
  }

  .depth-slider input[type='range']::-webkit-slider-thumb:hover {
    transform: scale(1.15);
  }

  .depth-slider input[type='range']::-moz-range-thumb {
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: var(--accent);
    border: 2px solid var(--bg-card);
    box-shadow: var(--neu-raised-sm);
    cursor: pointer;
  }

  .depth-slider input[type='range']:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  .depth-value {
    min-width: 24px;
    text-align: center;
    font-size: 0.82rem;
    color: var(--text-secondary);
    font-weight: 500;
  }
</style>
