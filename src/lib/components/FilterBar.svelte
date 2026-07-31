<script lang="ts">
  import { m } from '$lib/paraglide/messages.js';

  let {
    items,
    totalResults,
    onFilter,
  }: {
    items: any[];
    totalResults: number;
    onFilter: (filters: FilterState) => void;
  } = $props();

  export type FilterState = {
    statusCodes: number[];
    severities: string[];
    depth: number | undefined;
  };

  let selectedStatuses = $state<number[]>([]);
  let selectedSeverities = $state<string[]>([]);
  let maxDepth = $state<number | undefined>(undefined);

  let isOpen = $state(false);

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
      if (page.semantic_issues_json) {
        try {
          const issues = JSON.parse(page.semantic_issues_json);
          for (const issue of issues) {
            if (issue.severity) {
              sevCount[issue.severity] = (sevCount[issue.severity] || 0) + 1;
            }
          }
        } catch {}
      }
    }
    return Object.keys(sevCount).sort();
  });

  const activeFilterCount = $derived(
    selectedStatuses.length +
    selectedSeverities.length +
    (maxDepth !== undefined ? 1 : 0)
  );

  function toggleStatus(code: number) {
    if (selectedStatuses.includes(code)) {
      selectedStatuses = selectedStatuses.filter(s => s !== code);
    } else {
      selectedStatuses = [...selectedStatuses, code];
    }
    emitFilter();
  }

  function toggleSeverity(severity: string) {
    if (selectedSeverities.includes(severity)) {
      selectedSeverities = selectedSeverities.filter(s => s !== severity);
    } else {
      selectedSeverities = [...selectedSeverities, severity];
    }
    emitFilter();
  }

  function handleDepthChange(e: Event) {
    const val = parseInt((e.target as HTMLInputElement).value);
    maxDepth = isNaN(val) ? undefined : val;
    emitFilter();
  }

  function clearAll() {
    selectedStatuses = [];
    selectedSeverities = [];
    maxDepth = undefined;
    emitFilter();
  }

  function emitFilter() {
    onFilter({
      statusCodes: selectedStatuses,
      severities: selectedSeverities,
      depth: maxDepth,
    });
  }
</script>

<div class="filter-bar">
  <button class="filter-toggle" onclick={() => isOpen = !isOpen}>
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <polygon points="22 3 2 3 10 12.46 10 19 14 21 14 12.46 22 3"/>
    </svg>
    Filters
    {#if activeFilterCount > 0}
      <span class="filter-badge">{activeFilterCount}</span>
    {/if}
  </button>

  {#if isOpen}
    <div class="filter-panel">
      <div class="filter-group">
        <div class="filter-label">{m["filter.status"]()}</div>
        <div class="filter-chips">
          {#each AVAILABLE_STATUSES as code}
            <button
              class="chip"
              class:active={selectedStatuses.includes(code)}
              onclick={() => toggleStatus(code)}
            >
              {code}
            </button>
          {/each}
          {#if AVAILABLE_STATUSES.length === 0}
            <span class="filter-empty">{m["filter.no_options"]()}</span>
          {/if}
        </div>
      </div>

      <div class="filter-group">
        <div class="filter-label">{m["filter.severity"]()}</div>
        <div class="filter-chips">
          {#each availableSeverities as severity}
            <button
              class="chip chip-{severity}"
              class:active={selectedSeverities.includes(severity)}
              onclick={() => toggleSeverity(severity)}
            >
              {severity}
            </button>
          {/each}
          {#if availableSeverities.length === 0}
            <span class="filter-empty">{m["filter.no_options"]()}</span>
          {/if}
        </div>
      </div>

      <div class="filter-group">
        <div class="filter-label">{m["filter.depth"]()}</div>
        <div class="depth-slider">
          <input
            type="range"
            min="0"
            max="10"
            value={maxDepth ?? 10}
            oninput={handleDepthChange}
          />
          <span class="depth-value">{maxDepth !== undefined ? maxDepth : '—'}</span>
        </div>
      </div>

      {#if activeFilterCount > 0}
        <button class="btn-clear-all" onclick={clearAll}>
          {m["filter.clear_all"]()} ({activeFilterCount})
        </button>
      {/if}
    </div>
  {/if}
</div>

<style>
  .filter-bar {
    position: relative;
    margin-bottom: 8px;
  }

  .filter-toggle {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: 6px;
    color: var(--text);
    font-size: 0.82rem;
    cursor: pointer;
    transition: all 0.15s;
  }

  .filter-toggle:hover {
    background: var(--bg-hover);
    border-color: var(--border-muted);
  }

  .filter-badge {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 18px;
    height: 18px;
    padding: 0 5px;
    background: var(--accent);
    color: white;
    border-radius: 9px;
    font-size: 0.7rem;
    font-weight: 600;
  }

  .filter-panel {
    position: absolute;
    top: 100%;
    left: 0;
    margin-top: 4px;
    padding: 12px;
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: 8px;
    box-shadow: var(--shadow-md);
    z-index: 100;
    min-width: 320px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .filter-group {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .filter-label {
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .filter-chips {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }

  .chip {
    padding: 4px 10px;
    background: var(--bg-deep);
    border: 1px solid var(--border);
    border-radius: 14px;
    font-size: 0.78rem;
    color: var(--text-secondary);
    cursor: pointer;
    transition: all 0.15s;
  }

  .chip:hover {
    background: var(--bg-hover);
    border-color: var(--border-muted);
  }

  .chip.active {
    background: var(--accent-subtle);
    border-color: var(--accent);
    color: var(--accent);
    font-weight: 500;
  }

  .chip-error.active {
    background: var(--danger-subtle);
    border-color: var(--danger);
    color: var(--danger);
  }

  .chip-warning.active {
    background: var(--warning-subtle);
    border-color: var(--warning);
    color: var(--warning);
  }

  .chip-info.active {
    background: var(--info-subtle);
    border-color: var(--info);
    color: var(--info);
  }

  .filter-select {
    padding: 6px 10px;
    background: var(--bg-deep);
    border: 1px solid var(--border);
    border-radius: 6px;
    color: var(--text);
    font-size: 0.82rem;
    cursor: pointer;
  }

  .filter-select:focus {
    outline: none;
    border-color: var(--accent);
  }

  .depth-slider {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .depth-slider input[type="range"] {
    flex: 1;
    height: 4px;
    -webkit-appearance: none;
    appearance: none;
    background: var(--border);
    border-radius: 2px;
    outline: none;
  }

  .depth-slider input[type="range"]::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: var(--accent);
    cursor: pointer;
  }

  .depth-value {
    min-width: 24px;
    text-align: center;
    font-size: 0.82rem;
    color: var(--text-secondary);
    font-weight: 500;
  }

  .btn-clear-all {
    padding: 6px 12px;
    background: var(--danger-subtle);
    border: 1px solid var(--danger);
    border-radius: 6px;
    color: var(--danger);
    font-size: 0.78rem;
    cursor: pointer;
    transition: all 0.15s;
  }

  .btn-clear-all:hover {
    background: var(--danger);
    color: white;
  }
</style>
