<script lang="ts">
  import { m } from '$lib/paraglide/messages.js';
  import { getAppShell } from '$lib/app.svelte';
  import { Settings, Plus, FolderOpen, ExternalLink, Trash2, FileUp, Loader2 } from '@lucide/svelte';
  import { Button } from '$lib/components/ui/button/index.js';
  import { Input } from '$lib/components/ui/input/index.js';
  import * as Select from '$lib/components/ui/select/index.js';
  import { Label } from '$lib/components/ui/label/index.js';
  import { toast } from 'svelte-sonner';
  import type { Project } from '$lib/api/types';

  const app = getAppShell();

  let {
    projects,
    newProjectName = $bindable(),
    onCreate,
    onOpenProject,
    onOpenSettings,
    onDeleteProject,
  }: {
    projects: Project[];
    newProjectName: string;
    onCreate: () => void;
    onOpenProject: (id: string, name: string) => void;
    onOpenSettings: () => void;
    onDeleteProject: (id: string) => void;
  } = $props();

  let conflictMode = $state('skip');
  let importing = $state(false);

  const conflictOptions = $derived([
    { value: 'skip', label: m['transfer.import.skip']() },
    { value: 'copy', label: m['transfer.import.copy']() },
    { value: 'overwrite', label: m['transfer.import.overwrite']() },
  ]);
  const conflictLabel = $derived(
    conflictOptions.find((o) => o.value === conflictMode)?.label ?? ''
  );

  async function handlePickImport() {
    if (importing) return;
    importing = true;
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
    } finally {
      importing = false;
    }
  }
</script>

<div class="launcher">
  <header class="launcher-header">
    <div class="header-left">
      <h1 class="logo">{m['app.title']()}</h1>
      <span class="launcher-title">{m['launcher.title']()}</span>
      <Button
        variant="ghost"
        size="xs"
        class="btn-settings"
        onclick={onOpenSettings}
        aria-label={m['settings.title']()}
        title={m['settings.title']()}
      >
        <Settings class="size-4" />
      </Button>
    </div>
  </header>

  <main class="launcher-main">
    <p class="launcher-hint">{m['launcher.hint']()}</p>

    <div class="project-create">
      <Input
        type="text"
        bind:value={newProjectName}
        placeholder={m['sidebar.new_project_placeholder']()}
        onkeydown={(e) => e.key === 'Enter' && onCreate()}
      />
      <Button
        size="icon"
        onclick={onCreate}
        disabled={!newProjectName.trim()}
        aria-label={m['sidebar.new_project_placeholder']()}
        title={m['sidebar.new_project_placeholder']()}
      >
        <Plus class="size-4" />
      </Button>
    </div>

    {#if projects.length === 0}
      <div class="launcher-empty">
        <FolderOpen class="launcher-empty-icon" />
        <p>{m['sidebar.no_projects']()}</p>
      </div>
    {:else}
      <div class="launcher-grid">
        {#each projects as project (project.id)}
          <div class="launcher-card">
            <div class="launcher-card-head">
              <span class="launcher-avatar"
                >{project.name.trim().charAt(0).toUpperCase() || '?'}</span
              >
              <span class="launcher-card-name" title={project.name}>{project.name}</span>
              <Button
                variant="ghost"
                size="icon"
                class="btn-project-action launcher-delete"
                onclick={() => onDeleteProject(project.id)}
                aria-label={m['sidebar.delete']()}
                title={m['sidebar.delete']()}
              >
                <Trash2 class="size-4" />
              </Button>
            </div>
            <Button
              class="launcher-open"
              onclick={() => onOpenProject(project.id, project.name)}
              aria-label={m['launcher.open']()}
            >
              <ExternalLink class="size-4" />
              {m['launcher.open']()}
            </Button>
          </div>
        {/each}
      </div>
    {/if}

    <div class="import-card">
      <div class="import-card-head">
        <FileUp class="size-4" />
        <h3 class="import-card-title">{m['transfer.tab.import']()}</h3>
      </div>
      <div class="import-card-body">
        <div class="grid gap-1.5">
          <Label for="launcher-conflict">{m['transfer.import.mode']()}</Label>
          <Select.Root type="single" bind:value={conflictMode}>
            <Select.Trigger id="launcher-conflict" class="w-full">
              <span data-slot="select-value">{conflictLabel}</span>
            </Select.Trigger>
            <Select.Content>
              {#each conflictOptions as opt (opt.value)}
                <Select.Item value={opt.value} label={opt.label} />
              {/each}
            </Select.Content>
          </Select.Root>
        </div>
        <Button
          class="import-card-btn"
          onclick={handlePickImport}
          disabled={importing || app.transferBusy}
        >
          {#if importing || app.transferBusy}
            <Loader2 class="size-4 animate-spin" />
          {:else}
            <FolderOpen class="size-4" />
          {/if}
          {m['transfer.import.pick']()}
        </Button>
        <p class="import-card-hint">{m['transfer.import.mode_hint']()}</p>
      </div>
    </div>
  </main>
</div>

<style>
  .launcher {
    display: flex;
    flex-direction: column;
    height: 100vh;
    height: 100dvh;
    overflow: hidden;
  }

  .launcher-header {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: calc(10px + env(safe-area-inset-top)) 16px 10px;
    background: var(--bg-sidebar);
    z-index: 20;
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .logo {
    font-size: 1.1rem;
    font-weight: 700;
    background: var(--accent-gradient);
    -webkit-background-clip: text;
    background-clip: text;
    -webkit-text-fill-color: transparent;
    margin: 0;
    white-space: nowrap;
  }

  .launcher-title {
    font-size: 0.95rem;
    font-weight: 600;
    color: var(--text-secondary);
  }

  .launcher-main {
    flex: 1;
    overflow-y: auto;
    padding: var(--space-lg);
    max-width: 1100px;
    width: 100%;
    margin: 0 auto;
  }

  .launcher-hint {
    color: var(--text-muted);
    font-size: 0.85rem;
    margin: 0 0 16px;
  }

  .project-create {
    display: flex;
    gap: 6px;
    width: 100%;
    max-width: 420px;
    margin-bottom: 24px;
  }

  .launcher-empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    padding: 48px 16px;
    color: var(--text-muted);
    text-align: center;
  }

  :global(.launcher-empty-icon) {
    width: 40px;
    height: 40px;
    color: var(--border-muted);
  }

  .launcher-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
    gap: 12px;
  }

  .launcher-card {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 14px 16px;
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    box-shadow: var(--neu-raised-sm);
    transition:
      box-shadow var(--transition-base),
      transform var(--transition-fast);
  }

  .launcher-card:hover {
    box-shadow: var(--neu-raised-md);
    transform: translateY(-1px);
  }

  .launcher-card-head {
    display: flex;
    align-items: center;
    gap: 10px;
    min-width: 0;
  }

  .launcher-avatar {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border-radius: 9px;
    background: var(--accent-gradient);
    color: #fff;
    font-size: 0.8rem;
    font-weight: 700;
    text-transform: uppercase;
    flex-shrink: 0;
  }

  .launcher-card-name {
    font-size: 0.95rem;
    font-weight: 600;
    color: var(--text);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    flex: 1;
    min-width: 0;
  }

  :global(.launcher-delete) {
    color: var(--text-muted);
    flex-shrink: 0;
  }

  :global(.launcher-delete:hover) {
    color: var(--destructive);
  }

  :global(.launcher-open) {
    width: 100%;
  }

  .import-card {
    margin-top: 24px;
    max-width: 420px;
    padding: 14px 16px;
    background: var(--bg-card);
    border: 1px dashed var(--border);
    border-radius: var(--radius-lg);
  }

  .import-card-head {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 10px;
  }

  .import-card-title {
    margin: 0;
    font-size: 0.85rem;
    font-weight: 600;
    color: var(--text);
  }

  .import-card-body {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  :global(.import-card-btn) {
    width: 100%;
  }

  .import-card-hint {
    margin: 0;
    font-size: 0.75rem;
    color: var(--text-muted);
  }
</style>
