<script lang="ts">
  import { m } from '$lib/paraglide/messages.js';
  import { Settings, Plus } from 'lucide-svelte';
  import { Button } from '$lib/components/ui/button/index.js';
  import { Input } from '$lib/components/ui/input/index.js';
  import type { ProjectsStore } from '$lib/app.svelte';

  let {
    projects,
    selectedProjectId,
    newProjectName = $bindable(),
    onSelect,
    onCreate,
    onOpenSettings,
    onOpenLauncher,
    compact = false,
    showSettings = true,
  }: {
    projects: ProjectsStore;
    selectedProjectId: string;
    newProjectName: string;
    onSelect: (id: string) => void;
    onCreate: () => void;
    onOpenSettings: () => void;
    onOpenLauncher?: () => void;
    compact?: boolean;
    showSettings?: boolean;
  } = $props();
</script>

<header class="app-header" class:compact>
  <div class="header-left">
    <button
      type="button"
      class="logo-btn"
      onclick={onOpenLauncher}
      title={m['app.all_projects']()}
      aria-label={m['app.all_projects']()}
    >
      <h1 class="logo">{m['app.title']()}</h1>
    </button>
    {#if compact}
      <span class="header-project-name"
        >{projects.optimistic.find((p) => p.id === selectedProjectId)?.name ?? ''}</span
      >
    {/if}
    {#if showSettings}
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
    {/if}
  </div>

  {#if !compact}
    <div class="header-center">
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
    </div>

    <nav class="project-list-header">
      {#each projects.optimistic as project (project.id)}
        <div
          class="project-chip"
          class:selected={project.id === selectedProjectId}
          role="button"
          tabindex="0"
          onclick={() => onSelect(project.id)}
          onkeydown={(e) => {
            if (e.key === 'Enter' || e.key === ' ') onSelect(project.id);
          }}
        >
          <span class="project-avatar">{project.name.trim().charAt(0).toUpperCase() || '?'}</span>
          <span class="project-name">{project.name}</span>
        </div>
      {/each}
      {#if projects.optimistic.length === 0}
        <div class="empty-projects">{m['sidebar.no_projects']()}</div>
      {/if}
    </nav>
  {/if}
</header>

<style>
  .app-header {
    display: flex;
    flex-wrap: wrap;
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

  .header-center {
    display: flex;
    align-items: center;
    flex: 1;
    min-width: 180px;
  }

  .project-create {
    display: flex;
    gap: 6px;
    width: 100%;
    max-width: 420px;
  }

  .project-list-header {
    display: flex;
    gap: 8px;
    align-items: center;
    overflow-x: auto;
    overflow-y: hidden;
    scroll-behavior: smooth;
    scroll-snap-type: x proximity;
    padding: 6px 2px;
    -webkit-overflow-scrolling: touch;
    scrollbar-width: thin;
    scrollbar-color: var(--border-muted) transparent;
  }

  .project-list-header::-webkit-scrollbar {
    height: 6px;
  }

  .project-list-header::-webkit-scrollbar-track {
    background: transparent;
    border-radius: 3px;
  }

  .project-list-header::-webkit-scrollbar-thumb {
    background: var(--border-muted);
    border-radius: 3px;
  }

  .project-list-header::-webkit-scrollbar-thumb:hover {
    background: var(--text-muted);
  }

  .project-chip {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 10px 16px;
    min-height: 44px;
    border-radius: var(--radius-xl);
    cursor: pointer;
    transition:
      background var(--transition-base),
      border-color var(--transition-base),
      transform var(--transition-fast),
      box-shadow var(--transition-base);
    border-left: 4px solid transparent;
    background: var(--bg-card);
    border: none;
    box-shadow: var(--neu-raised-sm);
    scroll-snap-align: start;
    flex-shrink: 0;
    font-size: 0.9rem;
  }

  .project-chip:hover {
    background: var(--bg-hover);
    box-shadow: var(--neu-raised-md);
    transform: translateY(-1px);
  }

  .project-chip:active {
    box-shadow: var(--neu-pressed-sm);
    transform: translateY(0);
  }

  .project-chip:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 1px;
  }

  .project-chip.selected {
    background: var(--bg-hover);
    border-left: 4px solid var(--accent);
    box-shadow: var(--neu-pressed-sm);
  }

  .project-name {
    font-size: 0.9rem;
    color: var(--text);
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 180px;
  }

  .project-avatar {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 26px;
    height: 26px;
    border-radius: 8px;
    background: var(--accent-gradient);
    color: #fff;
    font-size: 0.72rem;
    font-weight: 700;
    text-transform: uppercase;
    flex-shrink: 0;
  }

  .empty-projects {
    padding: 10px 12px;
    text-align: center;
    color: var(--text-muted);
    font-size: 0.85rem;
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

  .logo-btn {
    background: none;
    border: none;
    padding: 0;
    margin: 0;
    display: inline-flex;
    align-items: center;
    border-radius: var(--radius-sm);
  }

  .logo-btn:hover .logo {
    filter: brightness(1.1);
  }

  .header-project-name {
    font-size: 0.95rem;
    font-weight: 600;
    color: var(--text);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 240px;
  }

  .app-header.compact {
    gap: 12px;
    padding: calc(10px + env(safe-area-inset-top)) 16px 10px;
  }

  /* ==========================================
     RESPONSIVE — Mobile First
     ========================================== */

  .app-header {
    flex-wrap: wrap;
    gap: 10px;
    padding: calc(10px + env(safe-area-inset-top)) 12px 10px;
  }

  .header-left {
    width: 100%;
    justify-content: space-between;
  }

  .header-center {
    width: 100%;
    flex: 1 1 100%;
  }

  .project-list-header {
    width: 100%;
    overflow-x: auto;
    padding: 4px 2px;
  }

  .project-chip {
    padding: 8px 14px;
    min-height: 42px;
    font-size: 0.85rem;
  }

  .project-name {
    max-width: 140px;
    font-size: 0.85rem;
  }

  @media (min-width: 768px) {
    .app-header {
      flex-wrap: nowrap;
      gap: 16px;
      padding: calc(12px + env(safe-area-inset-top)) 20px 12px;
    }

    .header-left {
      width: auto;
    }

    .header-center {
      flex: 0 1 420px;
    }

    .project-list-header {
      width: auto;
      overflow-x: auto;
      padding: 4px 2px;
    }

    .project-chip {
      padding: 10px 16px;
      min-height: 44px;
      font-size: 0.9rem;
    }

    .project-name {
      max-width: 180px;
      font-size: 0.9rem;
    }
  }

  @media (min-width: 1024px) {
    .app-header {
      padding: calc(14px + env(safe-area-inset-top)) 24px 14px;
      gap: 20px;
    }

    .header-center {
      flex: 0 1 520px;
    }

    .project-create {
      max-width: 520px;
    }
  }

  @media (min-width: 1440px) {
    .app-header {
      padding: calc(16px + env(safe-area-inset-top)) 32px 16px;
    }
  }
</style>
