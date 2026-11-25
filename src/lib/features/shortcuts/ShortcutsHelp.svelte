<script lang="ts">
  import { m } from '#lib/paraglide/messages.js';

  let {
    open = $bindable(false),
  }: {
    open: boolean;
  } = $props();

  let filter = $state('');

  interface HelpItem {
    keys: string[];
    label: () => string;
  }

  interface HelpGroup {
    title: () => string;
    items: HelpItem[];
  }

  const groups: HelpGroup[] = [
    {
      title: () => m['shortcuts.group_palette'](),
      items: [
        { keys: ['Ctrl', 'K'], label: () => m['shortcuts.palette_open']() },
        { keys: ['↑', '↓'], label: () => m['shortcuts.palette_navigate']() },
        { keys: ['Enter'], label: () => m['shortcuts.palette_run']() },
      ],
    },
    {
      title: () => m['shortcuts.group_navigation'](),
      items: [
        { keys: ['g', 'd'], label: () => m['shortcuts.go_dashboard']() },
        { keys: ['g', 'r'], label: () => m['shortcuts.go_results']() },
        { keys: ['g', 's'], label: () => m['shortcuts.go_tree']() },
        { keys: ['g', 'c'], label: () => m['shortcuts.go_comparator']() },
        { keys: ['g', 'p'], label: () => m['shortcuts.open_settings']() },
        { keys: ['/'], label: () => m['shortcuts.focus_search']() },
      ],
    },
    {
      title: () => m['shortcuts.group_table'](),
      items: [
        { keys: ['↑', '↓'], label: () => m['shortcuts.table_move']() },
        { keys: ['Enter'], label: () => m['shortcuts.table_open']() },
        { keys: ['Space'], label: () => m['shortcuts.table_toggle']() },
        { keys: ['e'], label: () => m['shortcuts.table_export']() },
        { keys: ['f'], label: () => m['shortcuts.table_focus_filter']() },
        { keys: ['1', '2', '3', '4'], label: () => m['shortcuts.table_page_size']() },
      ],
    },
    {
      title: () => m['shortcuts.group_crawl'](),
      items: [
        { keys: ['Ctrl', 'Enter'], label: () => m['shortcuts.crawl_start']() },
        { keys: ['Ctrl', '.'], label: () => m['shortcuts.crawl_stop']() },
      ],
    },
    {
      title: () => m['shortcuts.group_dialogs'](),
      items: [
        { keys: ['Esc'], label: () => m['shortcuts.esc_close']() },
        { keys: ['Esc'], label: () => m['shortcuts.esc_clear']() },
        { keys: ['?'], label: () => m['shortcuts.help_open']() },
      ],
    },
  ];

  const visibleGroups = $derived.by(() => {
    const q = filter.trim().toLowerCase();
    if (!q) return groups;
    return groups
      .map((g) => ({
        ...g,
        items: g.items.filter(
          (item) =>
            item.label().toLowerCase().includes(q) || item.keys.join(' ').toLowerCase().includes(q)
        ),
      }))
      .filter((g) => g.items.length > 0);
  });

  function close() {
    open = false;
  }
</script>

{#if open}
  <div
    class="help-overlay"
    role="presentation"
    onclick={(e) => {
      if (e.target === e.currentTarget) close();
    }}
  >
    <div class="help" role="dialog" aria-label={m['shortcuts.title']()}>
      <div class="help-head">
        <h2>{m['shortcuts.title']()}</h2>
        <button type="button" class="help-close" onclick={close} aria-label={m['detail.close']()}>
          ✕
        </button>
      </div>
      <input
        type="text"
        value={filter}
        placeholder={m['shortcuts.search_placeholder']()}
        oninput={(e) => {
          filter = (e.target as HTMLInputElement).value;
        }}
      />
      {#if visibleGroups.length === 0}
        <p class="help-empty">{m['palette.no_results']()}</p>
      {:else}
        {#each visibleGroups as group (group.title())}
          <h3>{group.title()}</h3>
          <ul>
            {#each group.items as item (item.label())}
              <li>
                <span class="help-keys">
                  {#each item.keys as key (key)}
                    <kbd>{key}</kbd>
                  {/each}
                </span>
                <span>{item.label()}</span>
              </li>
            {/each}
          </ul>
        {/each}
      {/if}
    </div>
  </div>
{/if}

<style>
  .help-overlay {
    position: fixed;
    inset: 0;
    z-index: 100;
    background: rgba(0, 0, 0, 0.45);
    display: flex;
    justify-content: center;
    align-items: flex-start;
    padding: 10vh 16px 16px;
  }

  .help {
    width: min(480px, 100%);
    max-height: 70vh;
    overflow-y: auto;
    background: var(--bg-card);
    border-radius: var(--radius-lg);
    box-shadow: var(--neu-raised-md);
    padding: 16px;
  }

  .help-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .help-head h2 {
    margin: 0;
    font-size: 1.1rem;
  }

  .help-close {
    background: none;
    border: none;
    cursor: pointer;
    font-size: 1rem;
    color: var(--text-secondary);
    padding: 4px 8px;
  }

  .help input {
    width: 100%;
    box-sizing: border-box;
    margin: 12px 0 4px;
    padding: 8px 10px;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: transparent;
    color: var(--text);
    font: inherit;
  }

  .help h3 {
    margin: 14px 0 6px;
    font-size: 0.85rem;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--text-secondary);
  }

  .help ul {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .help li {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 5px 0;
    font-size: 0.9rem;
  }

  .help-keys {
    display: inline-flex;
    gap: 3px;
    min-width: 84px;
  }

  .help-empty {
    color: var(--text-muted);
    font-size: 0.9rem;
  }

  kbd {
    font-family: inherit;
    font-size: 0.75rem;
    color: var(--text-secondary);
    background: var(--bg-deep);
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 1px 6px;
    white-space: nowrap;
  }
</style>
