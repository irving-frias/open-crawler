<script lang="ts">
  import { getSettings, saveSettings } from '$lib/api/settings';
  import type { SettingsMap } from '$lib/api/types';
  import { setLocale, getLocale, type Locale } from '$lib/paraglide/runtime.js';
  import { m } from '$lib/paraglide/messages.js';
  import { applyTheme as applyAppTheme } from '$lib/theme.js';
  import * as Dialog from '$lib/components/ui/dialog/index.js';
  import * as Select from '$lib/components/ui/select/index.js';
  import { Label } from '$lib/components/ui/label/index.js';
  import { Input } from '$lib/components/ui/input/index.js';
  import { Checkbox } from '$lib/components/ui/checkbox/index.js';
  import { Button } from '$lib/components/ui/button/index.js';
  import { Separator } from '$lib/components/ui/separator/index.js';
  import { checkForUpdates, downloadAndInstall, relaunchApp } from '$lib/updater';
  import type { UpdaterStatus } from '$lib/updater';

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
  let maxCrawlTime = $state('3600');
  let notificationsEnabled = $state(true);
  let pagespeedKeySet = $state(false);
  let pagespeedApiKey = $state('');
  let pagespeedRemoveKey = $state(false);
  let aiEnabled = $state(false);
  let aiKeySet = $state(false);
  let aiApiKey = $state('');
  let aiRemoveKey = $state(false);
  let aiBaseUrl = $state('https://api.openai.com/v1');
  let aiModel = $state('gpt-4o-mini');
  let saving = $state(false);
  let updaterStatus: UpdaterStatus = $state('idle');
  let updaterVersion: string | null = $state(null);
  let updaterError: string | null = $state(null);
  let updaterProgress = $state(0);

  $effect(() => {
    if (open) loadSettings();
  });

  async function loadSettings() {
    try {
      const settings = await getSettings();
      if (settings.language) language = settings.language as Locale;
      if (settings.theme) theme = settings.theme;
      if (settings.page_size) pageSize = settings.page_size;
      if (settings.max_depth) maxDepth = settings.max_depth;
      if (settings.respect_robots) respectRobots = settings.respect_robots === 'true';
      if (settings.check_sitemap) checkSitemap = settings.check_sitemap === 'true';
      if (settings.check_semantics) checkSemantics = settings.check_semantics === 'true';
      if (settings.max_crawl_time) maxCrawlTime = settings.max_crawl_time;
      if (settings.notifications_enabled !== undefined)
        notificationsEnabled = settings.notifications_enabled === 'true';
      pagespeedKeySet = settings.pagespeed_api_key_set === 'true';
      pagespeedRemoveKey = false;
      if (settings.ai_enabled) aiEnabled = settings.ai_enabled === 'true';
      aiKeySet = settings.ai_api_key_set === 'true';
      aiRemoveKey = false;
      if (settings.ai_base_url) aiBaseUrl = settings.ai_base_url;
      if (settings.ai_model) aiModel = settings.ai_model;
    } catch (e) {
      console.warn('Failed to load settings:', e);
    }
  }

  async function save() {
    saving = true;
    try {
      const settings: SettingsMap = {
        language,
        theme,
        page_size: pageSize,
        max_depth: maxDepth,
        respect_robots: respectRobots.toString(),
        check_sitemap: checkSitemap.toString(),
        check_semantics: checkSemantics.toString(),
        max_crawl_time: maxCrawlTime,
        notifications_enabled: notificationsEnabled.toString(),
        ai_enabled: aiEnabled.toString(),
        ai_base_url: aiBaseUrl,
        ai_model: aiModel,
      };
      if (pagespeedApiKey) settings.pagespeed_api_key = pagespeedApiKey;
      else if (pagespeedRemoveKey) settings.pagespeed_api_key = '';
      if (aiApiKey) settings.ai_api_key = aiApiKey;
      else if (aiRemoveKey) settings.ai_api_key = '';
      await saveSettings(settings);
      setLocale(language as Locale);
      applyAppTheme(theme);
      onsave?.(settings);
      open = false;
    } catch (e) {
      console.error('Failed to save settings:', e);
    } finally {
      saving = false;
    }
  }

  const languageLabel = $derived(language === 'en' ? m['language.en']() : m['language.es']());

  async function checkUpdates() {
    updaterStatus = 'checking';
    updaterError = null;
    updaterVersion = null;
    updaterProgress = 0;
    const result = await checkForUpdates();
    if (result.error) {
      updaterError = result.error;
      updaterStatus = 'idle';
      return;
    }
    if (result.version) {
      updaterVersion = result.version;
      updaterStatus = 'available';
    } else {
      updaterStatus = 'idle';
      updaterError = null;
      updaterVersion = null;
    }
  }

  async function installUpdate() {
    updaterStatus = 'installing';
    updaterError = null;
    updaterProgress = 0;
    try {
      await downloadAndInstall((percent) => (updaterProgress = percent));
    } catch (e) {
      updaterError = e instanceof Error ? e.message : String(e);
      updaterStatus = updaterVersion ? 'available' : 'idle';
      return;
    }
    updaterStatus = 'restart';
  }
</script>

<Dialog.Root bind:open>
  <Dialog.Content class="max-h-[90dvh] overflow-y-auto sm:max-h-[min(90dvh,640px)] sm:max-w-lg">
    <Dialog.Header>
      <Dialog.Title>{m['settings.title']()}</Dialog.Title>
    </Dialog.Header>

    <div class="flex flex-col gap-4">
      <div class="grid gap-1.5">
        <Label for="lang">{m['language.label']()}</Label>
        <Select.Root type="single" bind:value={language}>
          <Select.Trigger id="lang" class="w-full">
            <span data-slot="select-value">{languageLabel}</span>
          </Select.Trigger>
          <Select.Content>
            <Select.Item value="en" label={m['language.en']()} />
            <Select.Item value="es" label={m['language.es']()} />
          </Select.Content>
        </Select.Root>
      </div>

      <div class="grid gap-1.5">
        <Label for="theme">{m['theme.label']()}</Label>
        <Select.Root type="single" bind:value={theme}>
          <Select.Trigger id="theme" class="w-full">
            <span data-slot="select-value">
              {theme === 'light'
                ? m['theme.light']()
                : theme === 'dark'
                  ? m['theme.dark']()
                  : m['theme.system']()}
            </span>
          </Select.Trigger>
          <Select.Content>
            <Select.Item value="system" label={m['theme.system']()} />
            <Select.Item value="light" label={m['theme.light']()} />
            <Select.Item value="dark" label={m['theme.dark']()} />
          </Select.Content>
        </Select.Root>
      </div>

      <div class="grid gap-1.5">
        <Label for="page-size">{m['settings.page_size']()}</Label>
        <Select.Root type="single" bind:value={pageSize}>
          <Select.Trigger id="page-size" class="w-full">
            <span data-slot="select-value">{pageSize}</span>
          </Select.Trigger>
          <Select.Content>
            <Select.Item value="25" label="25" />
            <Select.Item value="50" label="50" />
            <Select.Item value="100" label="100" />
            <Select.Item value="200" label="200" />
          </Select.Content>
        </Select.Root>
      </div>

      <Separator />

      <div class="grid gap-2 pt-1">
        <div class="flex items-center gap-2">
          <Checkbox bind:checked={notificationsEnabled} id="cfg-notifications" />
          <Label for="cfg-notifications" class="cursor-pointer font-normal"
            >{m['settings.notifications']()}</Label
          >
        </div>
      </div>

      <Separator />

      <div class="grid gap-1.5">
        <Label for="pagespeed-key">{m['settings.pagespeed_api_key']()}</Label>
        <Input
          id="pagespeed-key"
          type="password"
          bind:value={pagespeedApiKey}
          autocomplete="off"
          placeholder={pagespeedKeySet
            ? m['settings.pagespeed_api_key_saved']()
            : m['settings.pagespeed_api_key_placeholder']()}
        />
        <p class="text-xs text-muted-foreground">{m['settings.pagespeed_api_key_hint']()}</p>
        {#if pagespeedKeySet && !pagespeedRemoveKey}
          <Button
            variant="ghost"
            class="h-6 w-fit px-2 text-xs text-destructive"
            onclick={() => {
              pagespeedRemoveKey = true;
              pagespeedApiKey = '';
            }}
          >
            {m['settings.pagespeed_api_key_clear']()}
          </Button>
        {/if}
      </div>

      <Separator />

      <h3 class="text-sm font-semibold uppercase tracking-wider text-muted-foreground">
        {m['settings.ai_assistant']()}
      </h3>
      <div class="grid gap-2 pt-1">
        <div class="flex items-center gap-2">
          <Checkbox bind:checked={aiEnabled} id="cfg-ai-enabled" />
          <Label for="cfg-ai-enabled" class="cursor-pointer font-normal"
            >{m['settings.ai_enabled']()}</Label
          >
        </div>
        <p class="text-xs text-muted-foreground">{m['settings.ai_enabled_hint']()}</p>
      </div>
      <div class="grid gap-1.5">
        <Label for="ai-api-key">{m['settings.ai_api_key']()}</Label>
        <Input
          id="ai-api-key"
          type="password"
          bind:value={aiApiKey}
          autocomplete="off"
          placeholder={aiKeySet
            ? m['settings.ai_api_key_saved']()
            : m['settings.ai_api_key_placeholder']()}
        />
        <p class="text-xs text-muted-foreground">{m['settings.ai_api_key_hint']()}</p>
        {#if aiKeySet && !aiRemoveKey}
          <Button
            variant="ghost"
            class="h-6 w-fit px-2 text-xs text-destructive"
            onclick={() => {
              aiRemoveKey = true;
              aiApiKey = '';
            }}
          >
            {m['settings.ai_api_key_clear']()}
          </Button>
        {/if}
      </div>
      <div class="grid gap-1.5">
        <Label for="ai-base-url">{m['settings.ai_base_url']()}</Label>
        <Input
          id="ai-base-url"
          type="text"
          bind:value={aiBaseUrl}
          placeholder={m['settings.ai_base_url_placeholder']()}
        />
      </div>
      <div class="grid gap-1.5">
        <Label for="ai-model">{m['settings.ai_model']()}</Label>
        <Input
          id="ai-model"
          type="text"
          bind:value={aiModel}
          placeholder={m['settings.ai_model_placeholder']()}
        />
      </div>

      <Separator />

      <h3 class="text-sm font-semibold uppercase tracking-wider text-muted-foreground">
        {m['settings.default_config']()}
      </h3>
      <div class="grid gap-1.5">
        <Label for="max-depth">{m['config.max_depth']()}</Label>
        <Input id="max-depth" type="number" bind:value={maxDepth} min="1" max="50" />
      </div>

      <div class="grid gap-1.5">
        <Label for="crawl-time">{m['config.time_limit']()}</Label>
        <Input id="crawl-time" type="number" bind:value={maxCrawlTime} min="60" max="86400" />
      </div>

      <div class="grid gap-2 pt-1">
        <div class="flex items-center gap-2">
          <Checkbox bind:checked={respectRobots} id="cfg-respect-robots" />
          <Label for="cfg-respect-robots" class="cursor-pointer font-normal"
            >{m['config.respect_robots']()}</Label
          >
        </div>
        <div class="flex items-center gap-2">
          <Checkbox bind:checked={checkSitemap} id="cfg-check-sitemap" />
          <Label for="cfg-check-sitemap" class="cursor-pointer font-normal"
            >{m['config.check_sitemap']()}</Label
          >
        </div>
        <div class="flex items-center gap-2">
          <Checkbox bind:checked={checkSemantics} id="cfg-check-semantics" />
          <Label for="cfg-check-semantics" class="cursor-pointer font-normal"
            >{m['config.check_semantics']()}</Label
          >
        </div>
      </div>
    </div>

    <Separator />

    <h3 class="text-sm font-semibold uppercase tracking-wider text-muted-foreground">
      {m['settings.update']()}
    </h3>
    <div class="flex flex-col gap-2">
      {#if updaterStatus === 'available'}
        <p class="text-sm text-muted-foreground">
          {m['updater.available']({ version: updaterVersion ?? '' })}
        </p>
        <Button onclick={installUpdate}>{m['updater.download']()}</Button>
      {:else if updaterStatus === 'installing'}
        <div class="flex items-center gap-2">
          <span
            class="h-4 w-4 animate-spin rounded-full border-2 border-primary border-t-transparent"
          ></span>
          <span class="text-sm text-muted-foreground"
            >{m['updater.installing']()} {updaterProgress}%</span
          >
        </div>
      {:else if updaterStatus === 'restart'}
        <p class="text-sm text-muted-foreground">{m['updater.install']()}</p>
        <Button onclick={relaunchApp}>{m['updater.install']()}</Button>
      {:else if updaterStatus === 'checking'}
        <div class="flex items-center gap-2">
          <span
            class="h-4 w-4 animate-spin rounded-full border-2 border-primary border-t-transparent"
          ></span>
          <span class="text-sm text-muted-foreground">{m['updater.checking']()}</span>
        </div>
      {:else}
        <div class="flex items-center gap-2">
          <Button variant="outline" onclick={checkUpdates}>{m['updater.check']()}</Button>
          {#if updaterError}
            <span class="text-sm text-destructive"
              >{m['updater.error']({ error: updaterError })}</span
            >
          {/if}
        </div>
      {/if}
    </div>

    <Dialog.Footer>
      <Button variant="outline" onclick={() => (open = false)}>{m['settings.cancel']()}</Button>
      <Button onclick={save} disabled={saving}>{saving ? '...' : m['settings.save']()}</Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
