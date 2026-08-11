<script lang="ts">
  import { m } from '$lib/paraglide/messages.js';
  import { getAppShell } from '$lib/app.svelte';
  import * as api from '$lib/api';
  import * as Dialog from '$lib/components/ui/dialog/index.js';
  import * as Tabs from '$lib/components/ui/tabs/index.js';
  import * as Select from '$lib/components/ui/select/index.js';
  import { Button } from '$lib/components/ui/button/index.js';
  import { Input } from '$lib/components/ui/input/index.js';
  import { Label } from '$lib/components/ui/label/index.js';
  import { Checkbox } from '$lib/components/ui/checkbox/index.js';
  import { Progress } from '$lib/components/ui/progress/index.js';
  import { toast } from 'svelte-sonner';
  import { Box, Download, FolderOpen, Share2, Loader2 } from '@lucide/svelte';

  const app = getAppShell();

  let {
    open = $bindable(false),
  }: {
    open: boolean;
  } = $props();
  let isMobile = $state(false);

  let includeCredentials = $state(false);
  let lightweight = $state(false);
  let conflictMode = $state('skip');
  let downloadUrl = $state('');
  let activeTab = $state<'export' | 'import'>('export');

  const conflictOptions = $derived([
    { value: 'skip', label: m['transfer.import.skip']() },
    { value: 'copy', label: m['transfer.import.copy']() },
    { value: 'overwrite', label: m['transfer.import.overwrite']() },
  ]);
  const conflictLabel = $derived(
    conflictOptions.find((o) => o.value === conflictMode)?.label ?? ''
  );

  function formatBytes(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
  }

  $effect(() => {
    api.crawl
      .isMobile()
      .then((v) => (isMobile = v))
      .catch(() => {});
  });

  async function handleExport(shareAfter = false) {
    await app.exportPackage(includeCredentials, lightweight, shareAfter);
  }

  async function handlePickImport() {
    try {
      const { open } = await import('@tauri-apps/plugin-dialog');
      const picked = await open({
        multiple: false,
        filters: [{ name: 'Open Crawler Package', extensions: ['ocproj'] }],
      });
      if (!picked || typeof picked !== 'string') return;
      await app.importPackage(picked, conflictMode as 'skip' | 'copy' | 'overwrite');
    } catch (e) {
      toast.error(String(e));
    }
  }

  async function handleDownloadAndImport() {
    const url = downloadUrl.trim();
    if (!url) return;
    try {
      const { appDataDir, join } = await import('@tauri-apps/api/path');
      const dir = await appDataDir();
      const name = url.split('/').pop()?.split('?')[0] || 'open-crawler.ocproj';
      const dest = await join(dir, 'transfers', name);
      await app.downloadTransfer(url, dest);
      await app.importPackage(dest, conflictMode as 'skip' | 'copy' | 'overwrite');
      downloadUrl = '';
    } catch (e) {
      toast.error(String(e));
    }
  }
</script>

<Dialog.Root bind:open>
  <Dialog.Content class="max-h-[90dvh] overflow-y-auto sm:max-h-[min(90dvh,680px)] sm:max-w-lg">
    <Dialog.Header>
      <Dialog.Title>{m['transfer.title']()}</Dialog.Title>
      <Dialog.Description>{m['transfer.subtitle']()}</Dialog.Description>
    </Dialog.Header>

    <div class="mb-4 flex flex-col gap-3 rounded-lg border p-3">
      <div class="flex items-center justify-between">
        <div>
          <p class="text-sm font-medium">{app.getSelectedProject()?.name ?? ''}</p>
          <p class="text-xs text-muted-foreground">{m['transfer.export.selected_project']()}</p>
        </div>
        <Checkbox
          checked={includeCredentials}
          onCheckedChange={(c) => (includeCredentials = c === true)}
          aria-label={m['transfer.export.include_credentials']()}
        />
      </div>
      <div class="flex items-center justify-between">
        <div>
          <p class="text-sm font-medium">{m['transfer.export.lightweight']()}</p>
          <p class="text-xs text-muted-foreground">{m['transfer.export.lightweight_hint']()}</p>
        </div>
        <Checkbox
          checked={lightweight}
          onCheckedChange={(c) => (lightweight = c === true)}
          aria-label={m['transfer.export.lightweight']()}
        />
      </div>
      <p class="text-xs text-muted-foreground">{m['transfer.export.credentials_hint']()}</p>
    </div>

    <Tabs.Root bind:value={activeTab} class="mb-4">
      <Tabs.List class="w-full">
        <Tabs.Trigger value="export">
          <Box class="size-4" />
          {m['transfer.tab.export']()}
        </Tabs.Trigger>
        <Tabs.Trigger value="import">
          <Download class="size-4" />
          {m['transfer.tab.import']()}
        </Tabs.Trigger>
      </Tabs.List>

      <!-- ==================== EXPORT ==================== -->
      <Tabs.Content value="export" class="space-y-4">
        <div class="flex flex-col gap-2">
          <Button onclick={() => handleExport(false)} disabled={app.transferBusy}>
            {#if app.transferBusy}
              <Loader2 class="size-4 animate-spin" />
            {/if}
            <Box class="size-4" />
            {m['transfer.export.btn']()}
          </Button>
          {#if isMobile}
            <Button
              variant="outline"
              onclick={() => handleExport(true)}
              disabled={app.transferBusy}
            >
              <Share2 class="size-4" />
              {m['transfer.export.share_btn']()}
            </Button>
          {/if}
        </div>

        {#if app.lastPackage}
          <div class="rounded-lg border p-3 text-sm">
            <p class="font-medium break-all">{app.lastPackage.file_name}</p>
            <p class="text-xs text-muted-foreground">{formatBytes(app.lastPackage.size_bytes)}</p>
          </div>
        {/if}
      </Tabs.Content>

      <!-- ==================== IMPORT ==================== -->
      <Tabs.Content value="import" class="space-y-4">
        <div class="grid gap-1.5">
          <Label for="conflict">{m['transfer.import.mode']()}</Label>
          <Select.Root type="single" bind:value={conflictMode}>
            <Select.Trigger id="conflict" class="w-full">
              <span data-slot="select-value">{conflictLabel}</span>
            </Select.Trigger>
            <Select.Content>
              {#each conflictOptions as opt (opt.value)}
                <Select.Item value={opt.value} label={opt.label} />
              {/each}
            </Select.Content>
          </Select.Root>
          <p class="text-xs text-muted-foreground">{m['transfer.import.mode_hint']()}</p>
        </div>

        <Button
          variant="outline"
          class="w-full"
          onclick={handlePickImport}
          disabled={app.transferBusy}
        >
          <FolderOpen class="size-4" />
          {m['transfer.import.pick']()}
        </Button>

        <div class="flex items-center gap-2">
          <div class="h-px flex-1 bg-border"></div>
          <span class="text-xs text-muted-foreground">{m['transfer.import.or']()}</span>
          <div class="h-px flex-1 bg-border"></div>
        </div>

        <div class="space-y-2">
          <Label for="download-url">{m['transfer.import.url']()}</Label>
          <div class="flex gap-2">
            <Input
              id="download-url"
              type="text"
              bind:value={downloadUrl}
              placeholder="http://192.168.1.5:45231/dl/…"
            />
            <Button
              onclick={handleDownloadAndImport}
              disabled={app.transferBusy || !downloadUrl.trim()}
            >
              {m['transfer.import.download']()}
            </Button>
          </div>
          {#if app.transferProgress.stage === 'download' && app.transferBusy}
            <div class="space-y-1">
              <Progress value={app.transferProgress.percent} class="h-2" />
              <p class="text-xs text-muted-foreground">
                {Math.round(app.transferProgress.percent)}%
              </p>
            </div>
          {/if}
        </div>
      </Tabs.Content>
    </Tabs.Root>
  </Dialog.Content>
</Dialog.Root>
