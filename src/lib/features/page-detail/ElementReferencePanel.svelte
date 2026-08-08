<script lang="ts">
  import type { ElementReference } from '$lib/data/html-elements';
  import { m } from '$lib/paraglide/messages.js';
  import { getLocale } from '$lib/paraglide/runtime.js';
  import { ExternalLink } from 'lucide-svelte';

  let {
    tag,
    reference,
  }: {
    tag: string;
    reference: ElementReference;
  } = $props();

  const isEs = $derived(getLocale().startsWith('es'));
  const supportEntries = $derived(Object.entries(reference.support));
</script>

<div class="element-ref">
  <div class="ref-head">
    <code class="ref-tag">&lt;{tag}&gt;</code>
    <span class="ref-title">{m['element.reference.title']()}</span>
  </div>

  {#if reference.rawCategories.length}
    <div class="ref-row">
      <span class="ref-label">{m['element.categories']()}</span>
      <div class="ref-chips">
        {#if isEs}
          {#each reference.categories_es as cat, i (i)}
            <span class="chip">{cat}</span>
          {/each}
        {:else}
          {#each reference.rawCategories as group, gi (gi)}
            <span class="chip">
              {#each group as seg, si (si)}
                {#if typeof seg === 'string'}
                  {seg}
                {:else if seg.href}
                  <a href={seg.href} target="_blank" rel="noreferrer" class="ref-link">
                    {seg.text}<ExternalLink class="size-3" />
                  </a>
                {:else}
                  {seg.text}
                {/if}
              {/each}
            </span>
          {/each}
        {/if}
      </div>
    </div>
  {/if}

  {#if reference.rawContexts.length}
    <div class="ref-row">
      <span class="ref-label">{m['element.contexts']()}</span>
      <div class="ref-text">
        {#if isEs}
          {#each reference.contexts_es as ctx, i (i)}
            <span class="ref-context">{ctx}</span>
          {/each}
        {:else}
          {#each reference.rawContexts as group, gi (gi)}
            <span class="ref-context">
              {#each group as seg, si (si)}
                {#if typeof seg === 'string'}
                  {seg}
                {:else if seg.href}
                  <a href={seg.href} target="_blank" rel="noreferrer" class="ref-link">{seg.text}</a
                  >
                {:else}
                  {seg.text}
                {/if}
              {/each}
            </span>
          {/each}
        {/if}
      </div>
    </div>
  {/if}

  {#if reference.rawContentModel.length}
    <div class="ref-row">
      <span class="ref-label">{m['element.content_model']()}</span>
      <div class="ref-text">
        {#if isEs}
          {#each reference.contentModel_es as cm, i (i)}
            <span class="ref-context">{cm}</span>
          {/each}
        {:else}
          {#each reference.rawContentModel as group, gi (gi)}
            <span class="ref-context">
              {#each group as seg, si (si)}
                {#if typeof seg === 'string'}
                  {seg}
                {:else if seg.href}
                  <a href={seg.href} target="_blank" rel="noreferrer" class="ref-link">{seg.text}</a
                  >
                {:else}
                  {seg.text}
                {/if}
              {/each}
            </span>
          {/each}
        {/if}
      </div>
    </div>
  {/if}

  {#if reference.params.length}
    <div class="ref-row">
      <span class="ref-label">{m['element.parameters']()}</span>
      <div class="ref-text">
        {#if isEs}
          {#each reference.params_es as p, i (i)}
            <code class="ref-param">{p}</code>
          {/each}
        {:else}
          {#each reference.params as seg, si (si)}
            {#if typeof seg === 'string'}
              <code class="ref-param">{seg}</code>
            {:else if seg.href}
              <a href={seg.href} target="_blank" rel="noreferrer" class="ref-param-link">
                <code class="ref-param">{seg.text}</code><ExternalLink class="size-3" />
              </a>
            {:else}
              <code class="ref-param">{seg.text}</code>
            {/if}
          {/each}
        {/if}
      </div>
    </div>
  {/if}

  {#if supportEntries.length}
    <div class="ref-row ref-support">
      <span class="ref-label">{m['element.browser_support']()}</span>
      <div class="support-grid">
        {#each supportEntries as [browser, info] (browser)}
          <div class="support-row" title="WebHTMLElement: {info.WebHTMLElement}">
            <span class="support-browser">{browser}</span>
            <code class="support-ver">{info.WebHTMLElement}</code>
          </div>
        {/each}
      </div>
    </div>
  {/if}
</div>

<style>
  .element-ref {
    margin-top: 10px;
    padding: 10px 12px;
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: 10px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .ref-head {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .ref-tag {
    font-family: var(--font-mono);
    font-size: 0.75rem;
    font-weight: 700;
    color: var(--info);
    background: var(--bg-issue-info);
    padding: 2px 8px;
    border-radius: 4px;
  }

  .ref-title {
    font-size: 0.72rem;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--text-muted);
    font-weight: 600;
  }

  .ref-row {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .ref-label {
    font-size: 0.68rem;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--text-muted);
    font-weight: 600;
  }

  .ref-chips {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .chip {
    font-size: 0.75rem;
    color: var(--text-secondary);
    background: var(--bg-deep);
    border-radius: 6px;
    padding: 2px 8px;
  }

  .ref-text {
    font-size: 0.78rem;
    color: var(--text);
    line-height: 1.5;
  }

  .ref-context {
    display: block;
  }

  .ref-link {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    color: var(--accent);
    text-decoration: none;
    font-weight: 500;
  }
  .ref-link:hover {
    text-decoration: underline;
  }

  .ref-param {
    font-family: var(--font-mono);
    font-size: 0.75rem;
    color: var(--purple);
    background: var(--bg-deep);
    padding: 2px 6px;
    border-radius: 4px;
    margin-right: 4px;
  }

  .ref-param-link {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    color: var(--accent);
    text-decoration: none;
  }
  .ref-param-link:hover {
    text-decoration: underline;
  }

  .support-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
    gap: 4px 12px;
  }

  .support-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    font-size: 0.75rem;
  }

  .support-browser {
    color: var(--text-secondary);
  }

  .support-ver {
    font-family: var(--font-mono);
    font-size: 0.7rem;
    color: var(--text);
    background: var(--bg-deep);
    padding: 1px 6px;
    border-radius: 4px;
    white-space: nowrap;
  }
</style>
