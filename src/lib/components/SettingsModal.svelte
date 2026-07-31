<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { setLocale, getLocale, type Locale } from '$lib/paraglide/runtime.js';
  import { m } from '$lib/paraglide/messages.js';

  let {
    open = $bindable(false),
    onsave,
  }: {
    open: boolean;
    onsave?: (settings: Record<string, string>) => void;
  } = $props();

  let language = $state(getLocale());
  let theme = $state('system');
  let pageSize = $state('50');
  let maxDepth = $state('10');
  let respectRobots = $state(true);
  let checkSitemap = $state(true);
  let checkSemantics = $state(true);
  let maxCrawlTime = $state(3600);
  let saving = $state(false);

  $effect(() => {
    if (open) loadSettings();
  });

  async function loadSettings() {
    try {
      const settings = await invoke<Record<string, string>>('get_settings');
      if (settings.language) language = settings.language as Locale;
      if (settings.theme) theme = settings.theme;
      if (settings.page_size) pageSize = settings.page_size;
      if (settings.max_depth) maxDepth = settings.max_depth;
      if (settings.respect_robots) respectRobots = settings.respect_robots === 'true';
      if (settings.check_sitemap) checkSitemap = settings.check_sitemap === 'true';
      if (settings.check_semantics) checkSemantics = settings.check_semantics === 'true';
      if (settings.max_crawl_time) maxCrawlTime = parseInt(settings.max_crawl_time, 10);
    } catch (e) {
      console.warn('Failed to load settings:', e);
    }
  }

  async function save() {
    saving = true;
    try {
      const settings: Record<string, string> = {
        language,
        theme,
        page_size: pageSize,
        max_depth: maxDepth.toString(),
        respect_robots: respectRobots.toString(),
        check_sitemap: checkSitemap.toString(),
        check_semantics: checkSemantics.toString(),
        max_crawl_time: maxCrawlTime.toString(),
      };
      await invoke('save_settings', { settings });
      setLocale(language as Locale);
      applyTheme(theme);
      onsave?.(settings);
      open = false;
    } catch (e) {
      console.error('Failed to save settings:', e);
    } finally {
      saving = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') open = false;
  }

  function applyTheme(t: string) {
    document.documentElement.setAttribute('data-theme', t);
    localStorage.setItem('theme', t);
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if open}
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_noninteractive_tabindex -->
  <div class="modal-overlay" onclick={() => open = false} role="presentation">
    <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
    <div class="modal" onclick={(e) => e.stopPropagation()} role="dialog" tabindex="-1" aria-label="Settings">
      <div class="modal-header">
        <h2>{m["settings.title"]()}</h2>
        <button class="btn-close" onclick={() => open = false} aria-label="Close">✕</button>
      </div>

      <div class="modal-body">
        <div class="setting-group">
          <label for="lang">{m["language.label"]()}</label>
          <select id="lang" bind:value={language}>
            <option value="en">{m["language.en"]()}</option>
            <option value="es">{m["language.es"]()}</option>
          </select>
        </div>

        <div class="setting-group">
          <label for="theme">{m["theme.label"]()}</label>
          <select id="theme" bind:value={theme}>
            <option value="system">{m["theme.system"]()}</option>
            <option value="light">{m["theme.light"]()}</option>
            <option value="dark">{m["theme.dark"]()}</option>
          </select>
        </div>

        <div class="setting-divider"></div>

        <div class="setting-group">
          <label for="page-size">{m["settings.page_size"]()}</label>
          <select id="page-size" bind:value={pageSize}>
            <option value="25">25</option>
            <option value="50">50</option>
            <option value="100">100</option>
            <option value="200">200</option>
          </select>
        </div>

        <div class="setting-divider"></div>

        <h3>{m["settings.default_config"]()}</h3>

        <div class="setting-group">
          <label for="max-depth">{m["config.max_depth"]()}</label>
          <input id="max-depth" type="number" bind:value={maxDepth} min="1" max="50" />
        </div>

        <div class="setting-group">
          <label for="crawl-time">{m["config.time_limit"]()}</label>
          <input id="crawl-time" type="number" bind:value={maxCrawlTime} min="60" max="86400" />
        </div>

        <div class="setting-row">
          <label class="checkbox-label">
            <input type="checkbox" bind:checked={respectRobots} />
            {m["config.respect_robots"]()}
          </label>
        </div>

        <div class="setting-row">
          <label class="checkbox-label">
            <input type="checkbox" bind:checked={checkSitemap} />
            {m["config.check_sitemap"]()}
          </label>
        </div>

        <div class="setting-row">
          <label class="checkbox-label">
            <input type="checkbox" bind:checked={checkSemantics} />
            {m["config.check_semantics"]()}
          </label>
        </div>
      </div>

      <div class="modal-footer">
        <button class="btn btn-secondary" onclick={() => open = false}>{m["settings.cancel"]()}</button>
        <button class="btn btn-primary" onclick={save} disabled={saving}>
          {saving ? '...' : m["settings.save"]()}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    z-index: var(--z-modal);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--space-md);
  }

  .modal {
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: var(--radius-xl);
    width: 100%;
    max-width: 480px;
    max-height: 90vh;
    overflow-y: auto;
    box-shadow: var(--shadow-lg);
  }

  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--space-lg);
    border-bottom: 1px solid var(--border);
  }

  .modal-header h2 {
    margin: 0;
    font-size: var(--text-lg);
    color: var(--text);
  }

  .btn-close {
    width: 32px;
    height: 32px;
    background: none;
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    color: var(--text-secondary);
    font-size: 1rem;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all var(--transition-base);
  }
  .btn-close:hover {
    background: var(--bg-hover);
    color: var(--text);
  }

  .modal-body {
    padding: var(--space-lg);
    display: flex;
    flex-direction: column;
    gap: var(--space-md);
  }

  .setting-group {
    display: flex;
    flex-direction: column;
    gap: var(--space-xs);
  }

  .setting-group label {
    font-size: var(--text-sm);
    color: var(--text-secondary);
    font-weight: var(--weight-medium);
  }

  .setting-group select,
  .setting-group input {
    padding: 8px 12px;
    background: var(--bg-deep);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    color: var(--text);
    font-size: var(--text-base);
    transition: border-color var(--transition-base);
  }

  .setting-group select:focus,
  .setting-group input:focus {
    outline: none;
    border-color: var(--accent);
  }

  .setting-row {
    padding: var(--space-xs) 0;
  }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    cursor: pointer;
    font-size: var(--text-base);
    color: var(--text-secondary);
  }

  .checkbox-label input[type='checkbox'] {
    width: 18px;
    height: 18px;
    accent-color: var(--accent);
  }

  .setting-divider {
    height: 1px;
    background: var(--border);
    margin: var(--space-xs) 0;
  }

  h3 {
    font-size: var(--text-sm);
    text-transform: uppercase;
    color: var(--text-muted);
    font-weight: var(--weight-semibold);
    margin: 0;
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-sm);
    padding: var(--space-lg);
    border-top: 1px solid var(--border);
  }

  .btn {
    padding: 8px 20px;
    border: none;
    border-radius: var(--radius-md);
    font-size: var(--text-base);
    font-weight: var(--weight-semibold);
    cursor: pointer;
    transition: all var(--transition-base);
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-primary {
    background: var(--accent-gradient);
    color: white;
  }
  .btn-primary:hover:not(:disabled) {
    box-shadow: var(--shadow-md);
  }

  .btn-secondary {
    background: var(--border);
    color: var(--text);
  }
  .btn-secondary:hover:not(:disabled) {
    background: var(--bg-hover);
  }
</style>
