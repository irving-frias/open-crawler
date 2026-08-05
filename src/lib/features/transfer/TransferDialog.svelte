<script lang="ts">
  import { toDataURL } from 'qrcode';
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
  import { Box, QrCode, Download, FolderOpen, Share2, Link2, Loader2, Bluetooth, Globe, Camera, Scan } from 'lucide-svelte';
  import { startQrScanner, webRtcAvailable } from './p2p';

  const app = getAppShell();

  let {
    open = $bindable(false),
  }: {
    open: boolean;
  } = $props();
  let isMobile = $state(false);
  let platform = $state('');
  let btAddr = $state('');

  const obexSupported = $derived(platform === 'linux' || platform === 'windows');

  const btStatusLabel = $derived(
    app.btStatus === 'exporting'
      ? m['transfer.bt.status.exporting']()
      : app.btStatus === 'sending'
        ? m['transfer.bt.status.sending']()
        : app.btStatus === 'complete'
          ? m['transfer.bt.status.complete']()
          : ''
  );

  let includeCredentials = $state(false);
  let lightweight = $state(false);
  let minutes = $state('15');
  let conflictMode = $state('skip');
  let downloadUrl = $state('');
  let qrDataUrl = $state('');
  let nowSecs = $state(Date.now());
  let receivedAtSecs = $state(0);
  let expiresTotal = $state(0);
  let p2pCode = $state('');
  let p2pQrDataUrl = $state('');
  let scanning = $state(false);
  let stopScan: (() => void) | null = null;
  let videoEl = $state<HTMLVideoElement | null>(null);

  const conflictOptions = $derived([
    { value: 'skip', label: m['transfer.import.skip']() },
    { value: 'copy', label: m['transfer.import.copy']() },
    { value: 'overwrite', label: m['transfer.import.overwrite']() },
  ]);
  const conflictLabel = $derived(conflictOptions.find((o) => o.value === conflictMode)?.label ?? '');

  const minutesOptions = [
    { value: '5', label: '5 min' },
    { value: '15', label: '15 min' },
    { value: '30', label: '30 min' },
    { value: '60', label: '60 min' },
  ];
  const minutesLabel = $derived(minutesOptions.find((o) => o.value === minutes)?.label ?? '15 min');

  const transfer = $derived(app.activeTransfer);
  const expiresIn = $derived(
    expiresTotal > 0 ? Math.max(0, Math.floor(expiresTotal - (nowSecs / 1000 - receivedAtSecs))) : 0
  );

  function formatBytes(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
  }

  function formatExpiry(secs: number): string {
    const mm = Math.floor(secs / 60);
    const ss = secs % 60;
    return `${mm}:${ss.toString().padStart(2, '0')}`;
  }

  const p2pStatusLabel = $derived(
    (app.p2pStatus || '') === 'exporting'
      ? m['transfer.p2p.status.exporting']()
      : (app.p2pStatus || '') === 'ready'
        ? m['transfer.p2p.status.ready']()
        : (app.p2pStatus || '') === 'connected'
          ? m['transfer.p2p.status.connected']()
          : (app.p2pStatus || '') === 'sending' || app.p2pStatus === 'sending-header'
            ? m['transfer.p2p.status.sending']()
            : (app.p2pStatus || '') === 'receiving'
              ? m['transfer.p2p.status.receiving']()
              : (app.p2pStatus || '') === 'connecting'
                ? m['transfer.p2p.status.connecting']()
                : (app.p2pStatus || '') === 'complete'
                  ? m['transfer.p2p.status.complete']()
                  : ''
  );

  const p2pError = $derived(
    app.p2pStatus === 'error' && app.p2pError ? app.p2pError : ''
  );

  // Browser receiver page served by the LAN server: http://<ip>:<port>/receive?peer=<id>
  const p2pBrowserUrl = $derived(
    app.activeTransfer && app.p2pPeerId
      ? (app.activeTransfer.urls[0]?.split('/dl/')[0] ?? '') +
          '/receive?peer=' +
          encodeURIComponent(app.p2pPeerId)
      : ''
  );

  function toggleScan() {
    if (scanning) {
      stopScan?.();
      stopScan = null;
      scanning = false;
      return;
    }
    if (!videoEl) return;
    scanning = true;
    stopScan = startQrScanner(
      videoEl,
      (text) => {
        p2pCode = text;
        stopScan?.();
        stopScan = null;
        scanning = false;
      },
      (message) => {
        toast.error(message);
        stopScan?.();
        stopScan = null;
        scanning = false;
      }
    );
  }

  async function handleReceiveP2P() {
    if (!p2pCode.trim()) return;
    await app.receiveP2P(p2pCode, conflictMode as 'skip' | 'copy' | 'overwrite');
    p2pCode = '';
  }

  $effect(() => {
    api.crawl.isMobile().then((v) => (isMobile = v)).catch(() => {});
    api.crawl.getPlatform().then((v) => (platform = v)).catch(() => {});
  });

  $effect(() => {
    if (!open) return;
    const t = app.activeTransfer;
    if (t) {
      receivedAtSecs = nowSecs / 1000;
      expiresTotal = t.expires_in_secs;
    }
    if (t && t.urls.length > 0) {
      toDataURL(t.urls[0], { width: 280, margin: 2, errorCorrectionLevel: 'M' })
        .then((d) => (qrDataUrl = d))
        .catch(() => (qrDataUrl = ''));
    } else {
      qrDataUrl = '';
    }
  });

  $effect(() => {
    if (!open || !app.activeTransfer) return;
    const id = setInterval(() => (nowSecs = Date.now()), 1000);
    return () => clearInterval(id);
  });

  $effect(() => {
    if (!app.p2pPeerId) {
      p2pQrDataUrl = '';
      return;
    }
    toDataURL(`ocp2p:${app.p2pPeerId}`, { width: 280, margin: 2, errorCorrectionLevel: 'M' })
      .then((d) => (p2pQrDataUrl = d))
      .catch(() => (p2pQrDataUrl = ''));
  });

  $effect(() => {
    if (!open) {
      stopScan?.();
      stopScan = null;
      scanning = false;
    }
  });

  async function handleExport(shareAfter = false) {
    await app.exportPackage(includeCredentials, lightweight, shareAfter);
  }

  async function handleStartWifi() {
    await app.exportAndStartWifi(includeCredentials, lightweight, parseInt(minutes, 10));
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

  async function handleStopWifi() {
    await app.stopTransferServer();
    qrDataUrl = '';
    receivedAtSecs = 0;
    expiresTotal = 0;
  }

  async function copyText(text: string) {
    try {
      await navigator.clipboard.writeText(text);
      toast.success(m['transfer.copied']());
    } catch {
      toast.error(m['transfer.copy_failed']());
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

    <Tabs.Root value={transfer ? 'wifi' : 'export'} class="mb-4">
      <Tabs.List class="w-full">
        <Tabs.Trigger value="export">
          <Box class="size-4" />
          {m['transfer.tab.export']()}
        </Tabs.Trigger>
        <Tabs.Trigger value="wifi">
          <QrCode class="size-4" />
          {m['transfer.tab.wifi']()}
        </Tabs.Trigger>
        <Tabs.Trigger value="import">
          <Download class="size-4" />
          {m['transfer.tab.import']()}
        </Tabs.Trigger>
        <Tabs.Trigger value="bluetooth">
          <Bluetooth class="size-4" />
          {m['transfer.tab.bluetooth']()}
        </Tabs.Trigger>
        <Tabs.Trigger value="p2p">
          <Globe class="size-4" />
          {m['transfer.tab.p2p']()}
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
            <Button variant="outline" onclick={() => handleExport(true)} disabled={app.transferBusy}>
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

      <!-- ==================== WIFI ==================== -->
      <Tabs.Content value="wifi" class="space-y-4">
        {#if transfer}
          <div class="flex flex-col items-center gap-3">
            {#if qrDataUrl}
              <img src={qrDataUrl} alt="QR" class="h-40 w-40 rounded-lg border bg-white p-1.5" />
            {:else}
              <div class="flex h-40 w-40 items-center justify-center rounded-lg border">
                <Loader2 class="size-5 animate-spin text-muted-foreground" />
              </div>
            {/if}
            <p class="text-center text-xs text-muted-foreground">{m['transfer.wifi.scan_hint']()}</p>

            <div class="max-h-24 w-full space-y-1 overflow-y-auto">
              {#each transfer.urls as url}
                <button
                  class="flex w-full items-center justify-between gap-2 rounded-lg border px-2.5 py-1.5 text-left text-sm hover:bg-muted"
                  title={url}
                  onclick={() => copyText(url)}
                >
                  <span class="truncate font-mono text-xs">{url}</span>
                  <Link2 class="size-3.5 shrink-0 text-muted-foreground" />
                </button>
              {/each}
            </div>

            <p class="w-full text-xs text-muted-foreground">{m['transfer.wifi.troubleshoot']()}</p>

            <div class="flex w-full items-center justify-between gap-3 rounded-lg border px-3 py-2">
              <span class="flex min-w-0 flex-col gap-0.5">
                <span class="truncate text-sm font-medium" title={transfer.file_name}>{transfer.file_name}</span>
                <span class="text-xs text-muted-foreground">
                  {formatBytes(transfer.file_size_bytes)} · {m['transfer.wifi.expires']()} {formatExpiry(expiresIn)}
                </span>
              </span>
              <Button variant="destructive" size="sm" class="shrink-0" onclick={handleStopWifi}>
                {m['transfer.wifi.stop']()}
              </Button>
            </div>
          </div>
        {:else}
          <div class="grid gap-1.5">
            <Label for="minutes">{m['transfer.wifi.ttl']()}</Label>
            <Select.Root type="single" bind:value={minutes}>
              <Select.Trigger id="minutes" class="w-full">
                <span data-slot="select-value">{minutesLabel}</span>
              </Select.Trigger>
              <Select.Content>
                {#each minutesOptions as opt}
                  <Select.Item value={opt.value} label={opt.label} />
                {/each}
              </Select.Content>
            </Select.Root>
          </div>
          <p class="text-xs text-muted-foreground">{m['transfer.wifi.export_first']()}</p>
          <Button class="w-full" onclick={handleStartWifi} disabled={app.transferBusy}>
            {#if app.transferBusy}
              <Loader2 class="size-4 animate-spin" />
            {/if}
            <QrCode class="size-4" />
            {m['transfer.wifi.start']()}
          </Button>
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
              {#each conflictOptions as opt}
                <Select.Item value={opt.value} label={opt.label} />
              {/each}
            </Select.Content>
          </Select.Root>
          <p class="text-xs text-muted-foreground">{m['transfer.import.mode_hint']()}</p>
        </div>

        <Button variant="outline" class="w-full" onclick={handlePickImport} disabled={app.transferBusy}>
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
            <Input id="download-url" type="text" bind:value={downloadUrl} placeholder="http://192.168.1.5:45231/dl/…" />
            <Button onclick={handleDownloadAndImport} disabled={app.transferBusy || !downloadUrl.trim()}>
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
      <!-- ==================== BLUETOOTH / NEARBY ==================== -->
      <Tabs.Content value="bluetooth" class="space-y-4">
        <p class="text-sm text-muted-foreground">{m['transfer.bt.intro']()}</p>

        <Button variant="outline" class="w-full" onclick={() => app.exportAndShare(includeCredentials, lightweight)} disabled={app.transferBusy}>
          {#if app.transferBusy}
            <Loader2 class="size-4 animate-spin" />
          {/if}
          <Share2 class="size-4" />
          {m['transfer.bt.send']()}
        </Button>

        {#if isMobile}
          <div class="flex items-center gap-2">
            <div class="h-px flex-1 bg-border"></div>
            <span class="text-xs text-muted-foreground">{m['transfer.import.or']()}</span>
            <div class="h-px flex-1 bg-border"></div>
          </div>

          <p class="text-xs text-muted-foreground">{m['transfer.bt.hint']()}</p>

          <Button
            variant="outline"
            class="w-full"
            onclick={() => app.importSharedIntent(conflictMode as 'skip' | 'copy' | 'overwrite')}
            disabled={app.shareImporting}
          >
            {#if app.shareImporting}
              <Loader2 class="size-4 animate-spin" />
            {/if}
            <Download class="size-4" />
            {m['transfer.bt.check']()}
          </Button>
        {:else}
          <p class="text-xs text-muted-foreground">{m['transfer.bt.desktop_hint']()}</p>
        {/if}

        <div class="flex items-center gap-2">
          <div class="h-px flex-1 bg-border"></div>
          <span class="text-xs text-muted-foreground">{m['transfer.import.or']()}</span>
          <div class="h-px flex-1 bg-border"></div>
        </div>

        <div class="space-y-2">
          <p class="text-sm font-medium">{m['transfer.bt.obex_title']()}</p>
          {#if obexSupported}
            <Label for="bt-addr">{m['transfer.bt.obex_hint']()}</Label>
            <Input
              id="bt-addr"
              type="text"
              bind:value={btAddr}
              placeholder={m['transfer.bt.obex_placeholder']()}
              pattern="^([0-9A-Fa-f]{2}:){5}[0-9A-Fa-f]{2}$"
              disabled={app.btBusy}
            />
            <Button
              class="w-full"
              variant="outline"
              onclick={() => app.sendViaObex(btAddr, includeCredentials, lightweight)}
              disabled={app.btBusy || !btAddr.trim()}
            >
              {#if app.btBusy}
                <Loader2 class="size-4 animate-spin" />
              {/if}
              <Bluetooth class="size-4" />
              {m['transfer.bt.obex_send']()}
            </Button>
            {#if btStatusLabel || app.btError}
              <p class="text-sm text-muted-foreground">{app.btError || btStatusLabel}</p>
            {/if}
            {#if app.btStatus === 'sending' && app.transferProgress.stage === 'bluetooth'}
              <div class="w-full space-y-1">
                <Progress value={app.transferProgress.percent} class="h-2" />
                <p class="text-xs text-muted-foreground">{Math.round(app.transferProgress.percent)}%</p>
              </div>
            {/if}
          {:else}
            <p class="text-xs text-amber-500">{m['transfer.bt.obex_unsupported']()}</p>
          {/if}
        </div>
      </Tabs.Content>

      <!-- ==================== INTERNET (P2P) ==================== -->
      <Tabs.Content value="p2p" class="space-y-4">
        {#if app.p2pPeerId}
          <div class="flex flex-col items-center gap-3">
            {#if p2pQrDataUrl}
              <img src={p2pQrDataUrl} alt="QR" class="h-56 w-56 rounded-lg border bg-white p-2" />
            {:else}
              <div class="flex h-56 w-56 items-center justify-center rounded-lg border">
                <Loader2 class="size-6 animate-spin text-muted-foreground" />
              </div>
            {/if}
            <p class="text-center text-sm text-muted-foreground">{m['transfer.p2p.scan_hint']()}</p>

            <div class="w-full space-y-2">
              <div class="flex items-center justify-between gap-2 rounded-lg border px-3 py-2 text-sm">
                <span class="truncate font-mono text-xs">ocp2p:{app.p2pPeerId}</span>
                <button
                  class="shrink-0 text-xs text-muted-foreground hover:text-foreground"
                  onclick={() => copyText(`ocp2p:${app.p2pPeerId}`)}
                >
                  {m['transfer.wifi.copy']()}
                </button>
              </div>
              {#if p2pBrowserUrl}
                <div class="flex items-center justify-between gap-2 rounded-lg border px-3 py-2 text-sm">
                  <span class="truncate font-mono text-xs">{p2pBrowserUrl}</span>
                  <button
                    class="shrink-0 text-xs text-muted-foreground hover:text-foreground"
                    onclick={() => copyText(p2pBrowserUrl)}
                  >
                    {m['transfer.wifi.copy']()}
                  </button>
                </div>
                <p class="text-xs text-muted-foreground">{m['transfer.p2p.browser_receiver']()}</p>
              {/if}
            </div>

            {#if p2pStatusLabel || p2pError}
              <p class="text-sm text-muted-foreground">{p2pError || p2pStatusLabel}</p>
            {/if}
            {#if app.p2pProgress.percent > 0}
              <div class="w-full space-y-1">
                <Progress value={app.p2pProgress.percent} class="h-2" />
                <p class="text-xs text-muted-foreground">{Math.round(app.p2pProgress.percent)}%</p>
              </div>
            {/if}

            <Button variant="destructive" class="w-full" onclick={app.stopP2P} disabled={app.p2pBusy}>
              {m['transfer.p2p.stop']()}
            </Button>
          </div>
        {:else}
          <p class="text-sm text-muted-foreground">{m['transfer.p2p.intro']()}</p>
          {#if !webRtcAvailable()}
            <p class="text-xs text-amber-500">{m['transfer.p2p.unsupported']()}</p>
          {/if}
          <Button class="w-full" onclick={() => app.exportAndStartP2P(includeCredentials, lightweight)} disabled={app.transferBusy || !webRtcAvailable()}>
            {#if app.transferBusy}
              <Loader2 class="size-4 animate-spin" />
            {/if}
            <Globe class="size-4" />
            {m['transfer.p2p.send']()}
          </Button>
          <p class="text-xs text-muted-foreground">{m['transfer.p2p.send_hint']()}</p>

          <div class="flex items-center gap-2">
            <div class="h-px flex-1 bg-border"></div>
            <span class="text-xs text-muted-foreground">{m['transfer.import.or']()}</span>
            <div class="h-px flex-1 bg-border"></div>
          </div>

          <div class="space-y-2">
            <Label for="p2p-code">{m['transfer.p2p.code']()}</Label>
            <div class="flex gap-2">
              <Input
                id="p2p-code"
                type="text"
                bind:value={p2pCode}
                placeholder={m['transfer.p2p.paste_placeholder']()}
              />
              <Button
                variant="outline"
                onclick={toggleScan}
                aria-label={m['transfer.p2p.scan']()}
              >
                {#if scanning}
                  <Loader2 class="size-4 animate-spin" />
                {:else}
                  <Camera class="size-4" />
                {/if}
              </Button>
            </div>
            {#if scanning}
              <video bind:this={videoEl} class="h-52 w-full rounded-lg border bg-black object-cover" muted playsinline></video>
              <p class="text-xs text-muted-foreground"><Scan class="mr-1 inline size-3" />{m['transfer.p2p.scan']()}…</p>
            {/if}
            <Button class="w-full" onclick={handleReceiveP2P} disabled={app.p2pBusy || !p2pCode.trim()}>
              {#if app.p2pBusy}
                <Loader2 class="size-4 animate-spin" />
              {/if}
              <Download class="size-4" />
              {m['transfer.p2p.receive']()}
            </Button>
            {#if p2pStatusLabel || p2pError}
              <p class="text-sm text-muted-foreground">{p2pError || p2pStatusLabel}</p>
            {/if}
            {#if app.p2pProgress.percent > 0}
              <div class="w-full space-y-1">
                <Progress value={app.p2pProgress.percent} class="h-2" />
                <p class="text-xs text-muted-foreground">{Math.round(app.p2pProgress.percent)}%</p>
              </div>
            {/if}
          </div>
        {/if}
      </Tabs.Content>
    </Tabs.Root>
  </Dialog.Content>
</Dialog.Root>
