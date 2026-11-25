<script lang="ts">
  import { m } from '#lib/paraglide/messages.js';
  import { getActions, fuzzyMatch, type ShortcutAction } from '#lib/shortcuts.ts';
  import { navigation } from '#lib/navigation.svelte.ts';
  import { getResults } from '#lib/api/results.ts';
  import type { AppShell } from '#lib/app.svelte.ts';
  import type { CrawlResult } from '#lib/api/types.ts';

  let {
    app,
    open = $bindable(false),
  }: {
    app: AppShell;
    open: boolean;
  } = $props();

  const RECENTS_KEY = 'oc-palette-recents';
  const MAX_RECENTS = 5;

  let query = $state('');
  let selected = $state(0);
  let inputEl = $state<HTMLInputElement | null>(null);
  let pageResults = $state<CrawlResult[]>([]);
  let pageSeq = 0;
  let searchTimer: ReturnType<typeof setTimeout> | null = null;

  const allActions = $derived(getActions(app));

  function loadRecents(): string[] {
    try {
      const raw = localStorage.getItem(RECENTS_KEY);
      return raw ? (JSON.parse(raw) as string[]) : [];
    } catch {
      return [];
    }
  }

  function saveRecent(id: string) {
    try {
      const recents = [id, ...loadRecents().filter((r) => r !== id)].slice(0, MAX_RECENTS);
      localStorage.setItem(RECENTS_KEY, JSON.stringify(recents));
    } catch {
      // Ignore storage failures.
    }
  }

  const recentActions = $derived.by(() => {
    // Re-read on every open so a just-run action shows up top.
    void open;
    if (query) return [];
    const ids = loadRecents();
    return ids
      .map((id) => allActions.find((a) => a.id === id))
      .filter((a): a is ShortcutAction => !!a);
  });

  const matchedActions = $derived.by(() => {
    const q = query.trim();
    if (!q) return allActions;
    return allActions.filter(
      (a) => fuzzyMatch(q, `${a.label()} ${a.keywords}`) || fuzzyMatch(q, a.id)
    );
  });

  type ListItem = { kind: 'action'; action: ShortcutAction } | { kind: 'page'; page: CrawlResult };

  const items = $derived.by((): ListItem[] => {
    const list: ListItem[] = [];
    for (const a of recentActions) list.push({ kind: 'action', action: a });
    for (const a of matchedActions) {
      if (list.some((i) => i.kind === 'action' && i.action.id === a.id)) continue;
      list.push({ kind: 'action', action: a });
    }
    for (const p of pageResults) list.push({ kind: 'page', page: p });
    return list;
  });

  $effect(() => {
    if (open) {
      query = '';
      pageResults = [];
      selected = 0;
      queueMicrotask(() => inputEl?.focus());
    }
  });

  $effect(() => {
    const q = query.trim();
    if (searchTimer) clearTimeout(searchTimer);
    if (!q || !app.selectedProjectId) {
      pageResults = [];
      return;
    }
    const seq = ++pageSeq;
    searchTimer = setTimeout(() => {
      void getResults({ projectId: app.selectedProjectId, page: 1, pageSize: 5, search: q })
        .then((res) => {
          if (seq === pageSeq) pageResults = res.items;
        })
        .catch(() => {
          if (seq === pageSeq) pageResults = [];
        });
    }, 300);
  });

  $effect(() => {
    // Keep the selection inside the list.
    if (selected >= items.length) selected = Math.max(0, items.length - 1);
    if (selected < 0) selected = 0;
  });

  function close() {
    open = false;
  }

  function runItem(item: ListItem) {
    if (item.kind === 'action') {
      saveRecent(item.action.id);
      close();
      item.action.run();
    } else {
      close();
      navigation.goToDetail(item.page.id);
    }
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      e.stopPropagation();
      selected = Math.min(items.length - 1, selected + 1);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      e.stopPropagation();
      selected = Math.max(0, selected - 1);
    } else if (e.key === 'Enter') {
      e.preventDefault();
      e.stopPropagation();
      const item = items[selected];
      if (item) runItem(item);
    } else if (e.key === 'Escape') {
      e.preventDefault();
      e.stopPropagation();
      close();
    }
  }
</script>

{#if open}
  <div
    class="palette-overlay"
    role="presentation"
    onclick={(e) => {
      if (e.target === e.currentTarget) close();
    }}
  >
    <div class="palette" role="dialog" aria-label={m['palette.title']()}>
      <input
        bind:this={inputEl}
        type="text"
        value={query}
        placeholder={m['palette.placeholder']()}
        oninput={(e) => {
          query = (e.target as HTMLInputElement).value;
          selected = 0;
        }}
        onkeydown={onKeydown}
      />
      {#if items.length === 0}
        <p class="palette-empty">{m['palette.no_results']()}</p>
      {:else}
        <ul class="palette-list">
          {#each items as item, i (item.kind === 'action' ? item.action.id : item.page.id)}
            <li>
              <button
                type="button"
                class="palette-item"
                class:palette-item-selected={i === selected}
                onclick={() => runItem(item)}
                onmouseenter={() => (selected = i)}
              >
                {#if item.kind === 'action'}
                  <span class="palette-label">{item.action.label()}</span>
                  {#if item.action.hint}
                    <kbd>{item.action.hint}</kbd>
                  {/if}
                {:else}
                  <span class="palette-label palette-page">
                    {item.page.title || item.page.url}
                  </span>
                  <span class="palette-url">{item.page.url}</span>
                {/if}
              </button>
            </li>
          {/each}
        </ul>
      {/if}
    </div>
  </div>
{/if}

<style>
  .palette-overlay {
    position: fixed;
    inset: 0;
    z-index: 100;
    background: rgba(0, 0, 0, 0.45);
    display: flex;
    justify-content: center;
    align-items: flex-start;
    padding: 12vh 16px 16px;
  }

  .palette {
    width: min(560px, 100%);
    background: var(--bg-card);
    border-radius: var(--radius-lg);
    box-shadow: var(--neu-raised-md);
    overflow: hidden;
  }

  .palette input {
    width: 100%;
    border: none;
    outline: none;
    padding: 14px 16px;
    font-size: 1rem;
    background: transparent;
    color: var(--text);
    border-bottom: 1px solid var(--border);
  }

  .palette-empty {
    padding: 16px;
    margin: 0;
    color: var(--text-muted);
    font-size: 0.9rem;
  }

  .palette-list {
    list-style: none;
    margin: 0;
    padding: 6px;
    max-height: 320px;
    overflow-y: auto;
  }

  .palette-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    width: 100%;
    background: none;
    border: none;
    border-radius: 8px;
    padding: 8px 10px;
    cursor: pointer;
    font: inherit;
    color: var(--text);
    text-align: left;
  }

  .palette-item-selected {
    background: var(--bg-deep);
  }

  .palette-label {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .palette-page {
    font-weight: 500;
  }

  .palette-url {
    color: var(--text-muted);
    font-size: 0.8rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 55%;
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
