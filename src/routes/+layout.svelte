<script lang="ts">
  import type { Snippet } from 'svelte';
  import { getLocale } from '$lib/paraglide/runtime.js';
  import { resolveTheme } from '$lib/theme.js';
  import { Toaster } from '$lib/components/ui/sonner/index.js';
  import '../lib/tokens.css';
  import '../app.css';

  let { children }: { children: Snippet } = $props();

  $effect(() => {
    document.documentElement.lang = getLocale();

    function apply() {
      document.documentElement.setAttribute('data-theme', resolveTheme(localStorage.getItem('theme') || 'system'));
    }

    apply();
    const mq = window.matchMedia('(prefers-color-scheme: dark)');
    mq.addEventListener('change', apply);
    return () => mq.removeEventListener('change', apply);
  });
</script>

<div class="app">
  {@render children()}
</div>

<Toaster richColors position="bottom-right" toastOptions={{ class: 'bg-card border-border' }} />

<style>
  :global(body) {
    font-family: var(--font-sans);
    font-size: var(--text-base);
    line-height: var(--leading-normal);
    background: var(--bg-deep);
    color: var(--text);
    min-height: 100vh;
    min-height: 100dvh;
    overscroll-behavior: none;
    -webkit-font-smoothing: antialiased;
    -webkit-tap-highlight-color: transparent;
  }

  :global(a) {
    color: var(--accent);
    text-decoration: none;
    -webkit-tap-highlight-color: transparent;
  }
  :global(a:hover) {
    color: var(--accent-hover);
  }

  :global(button) {
    font-family: inherit;
    cursor: pointer;
    touch-action: manipulation;
  }

  :global(input), :global(textarea), :global(select) {
    font-family: inherit;
    font-size: inherit;
  }

  :global(input),
  :global(textarea) {
    font-size: 16px;
  }

  @media (min-width: 768px) {
    :global(input),
    :global(textarea) {
      font-size: inherit;
    }
  }

  .app {
    max-width: 1920px;
    margin: 0 auto;
  }
</style>
