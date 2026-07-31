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
    domain: string;
    depth: number | undefined;
  };

  let selectedStatuses = $state<number[]>([]);
  let selectedSeverities = $state<string[]>([]);
  let selectedDomain = $state('');
  let maxDepth = $state<number | undefined>(undefined);

  let isOpen = $state(false);

  const AVAILABLE_STATUSES = [200, 301, 302, 404, 500, 502, 503];

  const domains = $derived.by(() => {
    const domainSet = new Set<string>();
    for (const page of items) {
      try {
        const url = new URL(page.url);
        domainSet.add(url.hostname);
      } catch {}
    }
    return Array.from(domainSet).sort();
  });

  const activeFilterCount = $derived(
    selectedStatuses.length +
    selectedSeverities.length +
    (selectedDomain ? 1 : 0) +
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

  function handleDomainChange(e: Event) {
    selectedDomain = (e.target as HTMLSelectElement).value;
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
    selectedDomain = '';
    maxDepth = undefined;
    emitFilter();
  }

  function emitFilter() {
    onFilter({
      statusCodes: selectedStatuses,
      severities: selectedSeverities,
      domain: selectedDomain,
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
        <label class="filter-label">{m["filter.status"]()}</label>
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
        </div>
      </div>

      <div class="filter-group">
        <label class="filter-label">{m["filter.severity"]()}</label>
        <div class="filter-chips">
          <button
            class="chip chip-error"
            class:active={selectedSeverities.includes('error')}
            onclick={() => toggleSeverity('error')}
          >
            {m["filter.error"]()}
          </button>
          <button
            class="chip chip-warning"
            class:active={selectedSeverities.includes('warning')}
            onclick={() => toggleSeverity('warning')}
          >
            {m["filter.warning"]()}
          </button>
          <button
            class="chip chip-info"
            class:active={selectedSeverities.includes('info')}
            onclick={() => toggleSeverity('info')}
          >
            {m["filter.info"]()}
          </button>
        </div>
      </div>

      {#if domains.length > 0}
        <div class="filter-group">
          <label class="filter-label">{m["filter.domain"]()}</label>
          <select class="filter-select" value={selectedDomain} onchange={handleDomainChange}>
            <option value="">{m["filter.all_domains"]()}</option>
            {#each domains as domain}
              <option value={domain}>{domain}</option>
            {/each}
          </select>
        </div>
      {/if}

      <div class="filter-group">
        <label class="filter-label">{m["filter.depth"]()}</label>
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
