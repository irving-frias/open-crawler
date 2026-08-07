<script lang="ts">
  import { m } from '$lib/paraglide/messages.js';
  import { RefreshCw, Download, ChevronDown, FileSpreadsheet, FileText, Send, Globe, Server } from 'lucide-svelte';
  import { Button } from '$lib/components/ui/button/index.js';
  import { Input } from '$lib/components/ui/input/index.js';
  import { Label } from '$lib/components/ui/label/index.js';
  import { Checkbox } from '$lib/components/ui/checkbox/index.js';
  import * as Popover from '$lib/components/ui/popover/index.js';
  import type { CrawlStatus, ResumableInfo } from '$lib/app.svelte';

  let {
    status,
    resumableInfo,
    hasResults,
    exporting,
    seedUrl = $bindable(),
    maxDepth = $bindable(),
    maxCrawlTime = $bindable(),
    proxyUrl = $bindable(),
    proxyUser = $bindable(),
    proxyPass = $bindable(),
    cookies = $bindable(),
    siteUser = $bindable(),
    sitePass = $bindable(),
    respectRobots = $bindable(),
    checkSitemap = $bindable(),
    checkSemantics = $bindable(),
    scanType = $bindable(),
    localUrls = $bindable(),
    onStart,
    onStop,
    onRefresh,
    onExport,
    onTransfer,
  }: {
    status: CrawlStatus;
    resumableInfo: ResumableInfo | null;
    hasResults: boolean;
    exporting: boolean;
    seedUrl: string;
    maxDepth: number;
    maxCrawlTime: number;
    proxyUrl: string;
    proxyUser: string;
    proxyPass: string;
    cookies: string;
    siteUser: string;
    sitePass: string;
    respectRobots: boolean;
    checkSitemap: boolean;
    checkSemantics: boolean;
    scanType: 'web' | 'local';
    localUrls: string;
    onStart: () => void;
    onStop: () => void;
    onRefresh: () => void;
    onExport: (format: 'xlsx' | 'csv') => void;
    onTransfer: () => void;
  } = $props();

  let seedError = $state(false);

  function isValidUrl(value: string): boolean {
    const v = value.trim();
    if (!v) return false;
    try {
      const url = new URL(v.includes('://') ? v : `https://${v}`);
      return url.hostname.includes('.') || url.hostname === 'localhost';
    } catch {
      return false;
    }
  }

  function normalizeSeedUrl(value: string): string {
    const v = value.trim();
    if (!v) return '';
    return v.includes('://') ? v : `https://${v}`;
  }

  function handleSeedBlur() {
    if (seedUrl.trim()) {
      seedUrl = normalizeSeedUrl(seedUrl);
    }
    seedError = seedUrl.trim() ? !isValidUrl(seedUrl) : false;
  }

  function handleStart() {
    seedError = !isValidUrl(seedUrl);
    if (seedError) return;
    onStart();
  }
</script>

<section class="config-section">
  <h2>{m['config.title']()}</h2>

  <div class="form-group">
    <Label for="seed">{m['config.seed_url']()}</Label>
    <Input
      id="seed"
      type="url"
      bind:value={seedUrl}
      placeholder={m['config.seed_url_placeholder']()}
      disabled={status === 'running'}
      aria-invalid={seedError ? 'true' : undefined}
      onblur={handleSeedBlur}
      oninput={() => {
        if (seedError) seedError = !isValidUrl(seedUrl);
      }}
    />
    {#if seedError}
      <p class="field-error">{m['config.seed_url_invalid']()}</p>
    {/if}
  </div>

  <div class="form-group">
    <Label>{m['config.scan_type']()}</Label>
    <div class="scan-type-toggle">
      <Button
        type="button"
        variant={scanType === 'web' ? 'default' : 'outline'}
        class="scan-type-btn"
        onclick={() => (scanType = 'web')}
        disabled={status === 'running'}
      >
        <Globe class="size-4" />
        {m['config.scan_web']()}
      </Button>
      <Button
        type="button"
        variant={scanType === 'local' ? 'default' : 'outline'}
        class="scan-type-btn"
        onclick={() => (scanType = 'local')}
        disabled={status === 'running'}
      >
        <Server class="size-4" />
        {m['config.scan_local']()}
      </Button>
    </div>
  </div>

  {#if scanType === 'local'}
    <div class="form-group">
      <Label for="localUrls">{m['config.local_urls']()}</Label>
      <textarea
        id="localUrls"
        class="local-urls-input"
        rows="4"
        bind:value={localUrls}
        placeholder={m['config.local_urls_placeholder']()}
        disabled={status === 'running'}
      ></textarea>
      <p class="field-hint">{m['config.local_urls_hint']()}</p>
    </div>
  {/if}

  <div class="form-row">
    <div class="form-group">
      <Label for="maxDepth">{m['config.max_depth']()}</Label>
      <Input id="maxDepth" type="number" bind:value={maxDepth} min="1" max="100" disabled={status === 'running'} />
    </div>
    <div class="form-group">
      <Label for="maxTime">{m['config.time_limit']()}</Label>
      <Input id="maxTime" type="number" bind:value={maxCrawlTime} min="0" max="86400" disabled={status === 'running'} />
      <p class="field-hint">{m['config.time_limit_hint']()}</p>
    </div>
  </div>

  <details class="advanced" open={proxyUrl.trim() ? true : undefined}>
    <summary>
      <span>{m['config.advanced_options']()}</span>
      <span class="chevron"><ChevronDown class="size-4" /></span>
    </summary>
    <div class="advanced-body">
      <div class="form-group">
        <Label for="proxyUrl">{m['config.proxy_url']()}</Label>
        <Input
          id="proxyUrl"
          type="text"
          bind:value={proxyUrl}
          placeholder={m['config.proxy_url_placeholder']()}
          disabled={status === 'running'}
        />
      </div>
      {#if proxyUrl.trim()}
        <div class="form-row">
          <div class="form-group">
            <Label for="proxyUser">{m['config.proxy_user']()}</Label>
            <Input id="proxyUser" type="text" bind:value={proxyUser} autocomplete="off" disabled={status === 'running'} />
          </div>
          <div class="form-group">
            <Label for="proxyPass">{m['config.proxy_pass']()}</Label>
            <Input id="proxyPass" type="password" bind:value={proxyPass} autocomplete="off" disabled={status === 'running'} />
          </div>
        </div>
      {/if}
      <div class="form-row">
        <div class="form-group">
          <Label for="siteUser">{m['config.site_user']()}</Label>
          <Input
            id="siteUser"
            type="text"
            bind:value={siteUser}
            placeholder={m['config.site_user_placeholder']()}
            autocomplete="off"
            disabled={status === 'running'}
          />
        </div>
        <div class="form-group">
          <Label for="sitePass">{m['config.site_pass']()}</Label>
          <Input
            id="sitePass"
            type="password"
            bind:value={sitePass}
            placeholder={m['config.site_pass_placeholder']()}
            autocomplete="off"
            disabled={status === 'running'}
          />
        </div>
      </div>
      <p class="field-hint">{m['config.site_auth_hint']()}</p>
      <div class="form-group">
        <Label for="cookies">{m['config.cookies']()}</Label>
        <textarea
          id="cookies"
          class="cookies-input"
          rows="3"
          bind:value={cookies}
          placeholder={m['config.cookies_placeholder']()}
          autocomplete="off"
          spellcheck="false"
          disabled={status === 'running'}
        ></textarea>
        <p class="field-hint">{m['config.cookies_hint']()}</p>
      </div>
    </div>
  </details>

  <div class="form-group checkboxes">
    <label class="checkbox-label">
      <Checkbox bind:checked={respectRobots} disabled={status === 'running'} />
      <span>{m['config.respect_robots']()}</span>
    </label>
    <label class="checkbox-label">
      <Checkbox bind:checked={checkSitemap} disabled={status === 'running'} />
      <span>{m['config.check_sitemap']()}</span>
    </label>
    <label class="checkbox-label">
      <Checkbox bind:checked={checkSemantics} disabled={status === 'running'} />
      <span>{m['config.check_semantics']()}</span>
    </label>
  </div>

  <div class="actions">
    {#if status === 'idle' || status === 'completed' || status === 'error'}
      <Button onclick={handleStart} disabled={!seedUrl}>
        {resumableInfo ? m['config.resume']() : hasResults ? m['config.rescan']() : m['config.start']()}
      </Button>
    {:else if status === 'running'}
      <Button variant="destructive" onclick={onStop}>{m['config.stop']()}</Button>
    {/if}
    <Button variant="outline" class="gap-1.5" onclick={onRefresh}>
      <RefreshCw class="size-4" />
      {m['config.refresh']()}
    </Button>
    {#if hasResults}
      <Button variant="outline" class="gap-1.5" onclick={onTransfer}>
        <Send class="size-4" />
        {m['transfer.open']()}
      </Button>
    {/if}
    {#if hasResults}
      <Popover.Root>
        <Popover.Trigger>
          {#snippet child({ props })}
            <Button
              variant="outline"
              class="gap-1.5"
              {...props}
              disabled={exporting}
            >
              <Download class="size-4" />
              {m['settings.export']()}
              <ChevronDown class="size-3.5" />
            </Button>
          {/snippet}
        </Popover.Trigger>
        <Popover.Content align="end" class="w-44 p-1">
          <Button
            variant="ghost"
            class="w-full justify-start gap-2"
            disabled={exporting}
            onclick={() => onExport('xlsx')}
          >
            <FileSpreadsheet class="size-4" />
            {m['export.xlsx']()}
          </Button>
          <Button
            variant="ghost"
            class="w-full justify-start gap-2"
            disabled={exporting}
            onclick={() => onExport('csv')}
          >
            <FileText class="size-4" />
            {m['export.csv']()}
          </Button>
        </Popover.Content>
      </Popover.Root>
    {/if}
  </div>

  {#if resumableInfo && status === 'idle'}
    <div class="resume-hint">
      {m['resume.hint']({ pages: resumableInfo.pages_crawled, urls: resumableInfo.queue_remaining })}
    </div>
  {/if}
</section>

<style>
  section {
    background: var(--bg-card);
    border: none;
    border-radius: var(--radius-xl);
    padding: var(--space-lg);
    box-shadow: var(--neu-raised-md);
  }

  h2 {
    font-size: 1.15rem;
    margin-bottom: 16px;
    color: var(--text);
  }

  .form-group {
    margin-bottom: 16px;
  }

  .form-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;
  }

  label {
    display: block;
    margin-bottom: 6px;
    font-size: 0.9rem;
    color: var(--text-secondary);
  }

  .checkboxes {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px 24px;
    align-items: center;
  }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
  }

  .advanced {
    border: none;
    border-radius: var(--radius-lg);
    margin-bottom: 16px;
    background: transparent;
    box-shadow: var(--neu-pressed-sm);
  }

  .advanced summary {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding: 10px 0;
    cursor: pointer;
    font-size: 0.9rem;
    color: var(--text-secondary);
    user-select: none;
    list-style: none;
  }

  .advanced summary::-webkit-details-marker {
    display: none;
  }

  .advanced[open] summary {
    color: var(--text);
    border-bottom: 1px solid var(--border);
  }

  .advanced[open] .chevron {
    transform: rotate(180deg);
  }

  .advanced-body {
    padding: 14px 0 2px;
  }

  .chevron {
    transition: transform 0.2s ease;
  }

  .field-hint {
    margin-top: 4px;
    font-size: 0.8rem;
    color: var(--text-muted);
  }

  .cookies-input {
    width: 100%;
    min-width: 0;
    border-radius: var(--radius-lg);
    border: none;
    background: var(--bg-background);
    padding: 8px 10px;
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    font-size: 0.85rem;
    line-height: 1.5;
    color: var(--text);
    resize: vertical;
    outline: none;
    box-shadow: var(--neu-pressed);
    transition: box-shadow 0.15s ease;
  }

  .cookies-input::placeholder {
    color: var(--text-muted);
  }

  .cookies-input:focus {
    box-shadow: var(--neu-focus);
  }

  .cookies-input:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .field-error {
    margin-top: 6px;
    font-size: 0.82rem;
    color: var(--destructive);
  }

  .scan-type-toggle {
    display: flex;
    gap: 8px;
  }

  .local-urls-input {
    width: 100%;
    min-width: 0;
    border-radius: var(--radius-lg);
    border: none;
    background: var(--bg-background);
    padding: 8px 10px;
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    font-size: 0.85rem;
    line-height: 1.5;
    color: var(--text);
    resize: vertical;
    outline: none;
    box-shadow: var(--neu-pressed);
    transition: box-shadow 0.15s ease;
  }

  .local-urls-input::placeholder {
    color: var(--text-muted);
  }

  .local-urls-input:focus {
    box-shadow: var(--neu-focus);
  }

  .local-urls-input:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .actions {
    display: flex;
    gap: 12px;
    margin-top: 8px;
    flex-wrap: wrap;
  }

  .resume-hint {
    margin-top: 12px;
    padding: 8px 12px;
    background: var(--bg-card);
    border: none;
    border-radius: var(--radius-md);
    box-shadow: var(--neu-raised-sm);
    font-size: 0.85rem;
    color: var(--warning);
  }

  @media (max-width: 767px) {
    .form-row {
      grid-template-columns: 1fr;
    }

    section {
      padding: var(--section-padding);
      border-radius: var(--radius-lg);
    }

    .checkboxes {
      grid-template-columns: 1fr;
      gap: var(--space-md);
    }
  }

  @media (min-width: 768px) {
    .form-row {
      grid-template-columns: 1fr 1fr;
    }
  }
</style>
