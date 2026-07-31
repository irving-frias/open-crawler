<script lang="ts">
  import type { Snippet } from 'svelte';
  import { getLocale } from '$lib/paraglide/runtime.js';
  import '../lib/tokens.css';

  let { children }: { children: Snippet } = $props();

  $effect(() => {
    document.documentElement.lang = getLocale();

    const savedTheme = localStorage.getItem('theme') || 'system';
    document.documentElement.setAttribute('data-theme', savedTheme);

    const mq = window.matchMedia('(prefers-color-scheme: dark)');
    function onSystemChange() {
      if ((localStorage.getItem('theme') || 'system') === 'system') {
        document.documentElement.setAttribute('data-theme', 'system');
      }
    }
    mq.addEventListener('change', onSystemChange);
    return () => mq.removeEventListener('change', onSystemChange);
  });
</script>

<div class="app">
  {@render children()}
</div>

<style>
  :global(*) {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
  }

  :global(body) {
    font-family: var(--font-sans);
    font-size: var(--text-base);
    line-height: var(--leading-normal);
    background: var(--bg-deep);
    color: var(--text);
    min-height: 100vh;
    -webkit-font-smoothing: antialiased;
  }

  :global(a) {
    color: var(--accent);
    text-decoration: none;
  }
  :global(a:hover) {
    color: var(--accent-hover);
  }

  :global(button) {
    font-family: inherit;
    cursor: pointer;
  }

  :global(input), :global(textarea), :global(select) {
    font-family: inherit;
    font-size: inherit;
  }

  .app {
    max-width: 1920px;
    margin: 0 auto;
    padding: var(--space-lg);
  }
</style>
