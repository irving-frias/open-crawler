<script lang="ts">
  import { toast } from 'svelte-sonner';
  import { CalendarClock, Plus, Pencil, Trash2, RefreshCw, Save, X } from '@lucide/svelte';
  import { m } from '$lib/paraglide/messages.js';
  import { cn } from '$lib/utils.js';
  import * as api from '$lib/api';
  import { getAppShell } from '$lib/app.svelte';
  import type { CrawlConfig, ScheduledJob } from '$lib/api/types';
  import { Badge } from '$lib/components/ui/badge/index.js';
  import { Button } from '$lib/components/ui/button/index.js';
  import { Card, CardContent } from '$lib/components/ui/card/index.js';
  import { Checkbox } from '$lib/components/ui/checkbox/index.js';
  import { Input } from '$lib/components/ui/input/index.js';
  import { Label } from '$lib/components/ui/label/index.js';
  import { Skeleton } from '$lib/components/ui/skeleton/index.js';

  let {
    projectId,
  }: {
    projectId: string;
  } = $props();

  const app = getAppShell();

  const config = $derived<CrawlConfig>({
    seed_urls: app.seedUrl.trim() ? [app.seedUrl.trim()] : [],
    max_depth: app.maxDepth,
    respect_robots: app.respectRobots,
    check_sitemap: app.checkSitemap,
    check_semantics: app.checkSemantics,
    max_crawl_time_secs: app.maxCrawlTime,
    cookies: app.cookies
      .split('\n')
      .map((c) => c.trim())
      .filter((c) => c.length > 0),
    site_auth:
      app.siteUser.trim() || app.sitePass
        ? { username: app.siteUser, password: app.sitePass }
        : null,
    proxy: app.proxyUrl.trim()
      ? {
          url: app.proxyUrl.trim(),
          username: app.proxyUser.trim() || null,
          password: app.proxyPass || null,
        }
      : null,
  });

  let jobs = $state<ScheduledJob[]>([]);
  let loading = $state(false);
  let creating = $state(false);
  let jobsSeq = 0;

  let newCron = $state('0 * * * *');
  let editingId = $state<string | null>(null);
  let editCron = $state('');

  const PRESETS = [
    { id: 'hourly', label: m['schedule.preset_hourly'](), expr: '0 * * * *' },
    { id: 'daily', label: m['schedule.preset_daily'](), expr: '0 0 * * *' },
    { id: 'weekly', label: m['schedule.preset_weekly'](), expr: '0 0 * * 1' },
  ] as const;

  function buildConfigJson(): string {
    const c = config ?? {};
    return JSON.stringify({
      seed_urls: c.seed_urls ?? [],
      max_depth: c.max_depth,
      respect_robots: c.respect_robots,
      check_sitemap: c.check_sitemap,
      check_semantics: c.check_semantics,
      max_crawl_time_secs: c.max_crawl_time_secs,
      concurrency: c.concurrency,
      delay_ms: c.delay_ms,
      same_origin_only: c.same_origin_only,
      request_timeout_ms: c.request_timeout_ms,
      include_patterns: c.include_patterns,
      exclude_patterns: c.exclude_patterns,
      cookies: c.cookies,
      site_auth: c.site_auth,
      proxy: c.proxy,
      custom_headers: c.custom_headers,
    });
  }

  async function loadJobs() {
    const seq = ++jobsSeq;
    loading = true;
    try {
      const data = await api.schedule.listScheduledJobs();
      if (seq !== jobsSeq) return;
      jobs = data;
    } catch (e) {
      if (seq === jobsSeq) toast.error(String(e));
    } finally {
      if (seq === jobsSeq) loading = false;
    }
  }

  async function createJob() {
    const expr = newCron.trim();
    if (!expr) {
      toast.error(m['schedule.cron_invalid']());
      return;
    }
    creating = true;
    try {
      await api.schedule.createScheduledJob({
        project_id: projectId,
        cron_expression: expr,
        config_json: buildConfigJson(),
      });
      newCron = '0 * * * *';
      await loadJobs();
      toast.success(m['schedule.toast_created']());
    } catch (e) {
      toast.error(String(e));
    } finally {
      creating = false;
    }
  }

  function startEdit(job: ScheduledJob) {
    editingId = job.id;
    editCron = job.cron_expression;
  }

  function cancelEdit() {
    editingId = null;
    editCron = '';
  }

  async function saveEdit(job: ScheduledJob) {
    const expr = editCron.trim();
    if (!expr) {
      toast.error(m['schedule.cron_invalid']());
      return;
    }
    try {
      await api.schedule.updateScheduledJob({ id: job.id, cron_expression: expr });
      editingId = null;
      editCron = '';
      await loadJobs();
      toast.success(m['schedule.toast_saved']());
    } catch (e) {
      toast.error(String(e));
    }
  }

  async function toggleEnabled(job: ScheduledJob) {
    const previous = { ...job };
    job.enabled = !job.enabled;
    try {
      const updated = await api.schedule.updateScheduledJob({
        id: job.id,
        enabled: job.enabled,
      });
      if (updated) {
        const idx = jobs.findIndex((j) => j.id === job.id);
        if (idx >= 0) jobs[idx] = updated;
      }
    } catch (e) {
      job.enabled = previous.enabled;
      toast.error(String(e));
    }
  }

  async function deleteJob(job: ScheduledJob) {
    try {
      await api.schedule.deleteScheduledJob(job.id);
      jobs = jobs.filter((j) => j.id !== job.id);
      toast.success(m['schedule.toast_deleted']());
    } catch (e) {
      toast.error(String(e));
    }
  }

  function formatDate(ts: string | null | undefined): string {
    if (!ts) return m['schedule.never']();
    const d = new Date(ts);
    if (isNaN(d.getTime())) return ts;
    return d.toLocaleString();
  }

  $effect(() => {
    if (projectId) loadJobs();
    else jobs = [];
  });
</script>

<div class="schedule">
  <div class="schedule-head">
    <div class="flex items-center gap-2 text-sm font-semibold">
      <CalendarClock class="size-4" />
      {m['schedule.title']()}
      {#if jobs.length > 0}
        <Badge variant="secondary" class="ml-1">{jobs.length}</Badge>
      {/if}
    </div>
    <Button
      variant="ghost"
      size="icon"
      class="size-7"
      onclick={loadJobs}
      aria-label={m['config.refresh']()}
      title={m['config.refresh']()}
      disabled={loading}
    >
      <RefreshCw class={cn('size-3.5', loading && 'animate-spin')} />
    </Button>
  </div>

  <Card>
    <CardContent>
      <div class="create-form">
        <div class="create-field">
          <Label for="cronExpr">{m['schedule.cron_label']()}</Label>
          <div class="create-row">
            <Input
              id="cronExpr"
              bind:value={newCron}
              class="h-9 font-mono text-sm"
              placeholder={m['schedule.cron_placeholder']()}
              onkeydown={(e) => e.key === 'Enter' && createJob()}
            />
            <Button class="shrink-0" size="sm" onclick={createJob} disabled={creating}>
              <Plus class="size-4" />
              {m['schedule.create']()}
            </Button>
          </div>
          <div class="presets">
            {#each PRESETS as p (p.id)}
              <Button variant="outline" size="xs" class="h-7" onclick={() => (newCron = p.expr)}>
                {p.label}
              </Button>
            {/each}
          </div>
          <p class="create-hint">{m['schedule.cron_hint']()}</p>
          <p class="create-hint">{m['schedule.uses_current_config']()}</p>
        </div>
      </div>
    </CardContent>
  </Card>

  {#if loading && jobs.length === 0}
    <div class="flex flex-col gap-2">
      <Skeleton class="h-12 w-full" />
      <Skeleton class="h-12 w-full" />
      <Skeleton class="h-12 w-4/5" />
    </div>
  {:else if jobs.length === 0}
    <div class="p-4 text-sm text-muted-foreground">{m['schedule.empty']()}</div>
  {:else}
    <ul class="job-list">
      {#each jobs as job (job.id)}
        <li class={cn('job-row', !job.enabled && 'job-disabled')}>
          <div class="job-main">
            <div class="job-cron">
              {#if editingId === job.id}
                <Input
                  bind:value={editCron}
                  class="h-8 w-full font-mono text-sm"
                  onkeydown={(e) => {
                    if (e.key === 'Enter') saveEdit(job);
                    if (e.key === 'Escape') cancelEdit();
                  }}
                />
              {:else}
                <code>{job.cron_expression}</code>
              {/if}
            </div>
            <div class="job-meta">
              <span>
                {m['schedule.next_run']()}: {formatDate(job.next_run)}
              </span>
              <span>
                {m['schedule.last_run']()}: {formatDate(job.last_run)}
              </span>
            </div>
          </div>
          <div class="job-actions">
            <label class="enabled-toggle">
              <Checkbox
                checked={job.enabled}
                onclick={() => toggleEnabled(job)}
                aria-label={m['schedule.enabled']()}
              />
              <span class="enabled-label">{m['schedule.enabled']()}</span>
            </label>
            {#if editingId === job.id}
              <Button variant="ghost" size="icon" class="size-8" onclick={() => saveEdit(job)}>
                <Save class="size-4" />
              </Button>
              <Button variant="ghost" size="icon" class="size-8" onclick={cancelEdit}>
                <X class="size-4" />
              </Button>
            {:else}
              <Button
                variant="ghost"
                size="icon"
                class="size-8"
                onclick={() => startEdit(job)}
                title={m['schedule.edit']()}
              >
                <Pencil class="size-4" />
              </Button>
            {/if}
            <Button
              variant="ghost"
              size="icon"
              class="size-8 text-destructive"
              onclick={() => deleteJob(job)}
              title={m['schedule.delete']()}
            >
              <Trash2 class="size-4" />
            </Button>
          </div>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .schedule {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .schedule-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4px 8px 0;
  }

  .create-form {
    padding-top: 8px;
  }

  .create-field {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .create-row {
    display: flex;
    gap: 8px;
  }

  .presets {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .create-hint {
    margin: 0;
    font-size: 0.78rem;
    color: var(--text-muted);
  }

  .job-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .job-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 10px 12px;
    border-radius: var(--radius-lg);
    background: var(--bg-card);
    box-shadow: var(--neu-pressed-sm);
  }

  .job-disabled {
    opacity: 0.55;
  }

  .job-main {
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-width: 0;
    flex: 1;
  }

  .job-cron code {
    font-size: 0.95rem;
    font-weight: 600;
    color: var(--text);
  }

  .job-meta {
    display: flex;
    flex-wrap: wrap;
    gap: 4px 16px;
    font-size: 0.78rem;
    color: var(--text-muted);
  }

  .job-actions {
    display: flex;
    align-items: center;
    gap: 4px;
    flex-shrink: 0;
  }

  .enabled-toggle {
    display: flex;
    align-items: center;
    gap: 6px;
    cursor: pointer;
    margin-right: 4px;
  }

  .enabled-label {
    font-size: 0.78rem;
    color: var(--text-secondary);
  }
</style>
