<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
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
      if (settings.max_crawl_time) maxCrawlTime = settings.max_crawl_time;
      if (settings.notifications_enabled !== undefined) notificationsEnabled = settings.notifications_enabled === 'true';
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
        max_depth: maxDepth,
        respect_robots: respectRobots.toString(),
        check_sitemap: checkSitemap.toString(),
        check_semantics: checkSemantics.toString(),
        max_crawl_time: maxCrawlTime,
        notifications_enabled: notificationsEnabled.toString(),
      };
      await invoke('save_settings', { settings });
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
</script>

<Dialog.Root bind:open>
  <Dialog.Content
    class="max-h-[90dvh] overflow-y-auto sm:max-h-[min(90dvh,640px)] sm:max-w-lg"
  >
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
              {theme === 'light' ? m['theme.light']() : theme === 'dark' ? m['theme.dark']() : m['theme.system']()}
            </span>
          </Select.Trigger>
          <Select.Content>
            <Select.Item value="system" label={m['theme.system']()} />
            <Select.Item value="light" label={m['theme.light']()} />
            <Select.Item value="dark" label={m['theme.dark']()} />
          </Select.Content>
        </Select.Root>
      </div>

      <Separator />

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
          <Label for="cfg-notifications" class="cursor-pointer font-normal">{m['settings.notifications']()}</Label>
        </div>
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
          <Label for="cfg-respect-robots" class="cursor-pointer font-normal">{m['config.respect_robots']()}</Label>
        </div>
        <div class="flex items-center gap-2">
          <Checkbox bind:checked={checkSitemap} id="cfg-check-sitemap" />
          <Label for="cfg-check-sitemap" class="cursor-pointer font-normal">{m['config.check_sitemap']()}</Label>
        </div>
        <div class="flex items-center gap-2">
          <Checkbox bind:checked={checkSemantics} id="cfg-check-semantics" />
          <Label for="cfg-check-semantics" class="cursor-pointer font-normal">{m['config.check_semantics']()}</Label>
        </div>
      </div>
    </div>

    <Dialog.Footer>
      <Button variant="outline" onclick={() => (open = false)}>{m['settings.cancel']()}</Button>
      <Button onclick={save} disabled={saving}>{saving ? '...' : m['settings.save']()}</Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
