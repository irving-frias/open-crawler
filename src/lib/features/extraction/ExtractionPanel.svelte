<script lang="ts">
  import { toast } from 'svelte-sonner';
  import { Pencil, Trash2, RefreshCw, Save, X, FlaskConical } from '@lucide/svelte';
  import { m } from '$lib/paraglide/messages.js';
  import * as api from '$lib/api';
  import { getAppShell } from '$lib/app.svelte';
  import type { CustomExtraction, ExtractionMode, SelectorType } from '$lib/api/types';
  import { Badge } from '$lib/components/ui/badge/index.js';
  import { Button } from '$lib/components/ui/button/index.js';
  import { Card, CardContent } from '$lib/components/ui/card/index.js';
  import { Input } from '$lib/components/ui/input/index.js';
  import { Label } from '$lib/components/ui/label/index.js';
  import * as Select from '$lib/components/ui/select/index.js';
  import { Skeleton } from '$lib/components/ui/skeleton/index.js';

  let { projectId }: { projectId: string } = $props();

  const app = getAppShell();

  let rules = $state<CustomExtraction[]>([]);
  let loading = $state(true);
  let saving = $state(false);
  let recomputing = $state(false);
  let testing = $state(false);
  let testUrl = $state(app.seedUrl.trim());
  let testResult = $state<string | null>(null);
  let testError = $state<string | null>(null);

  // Draft form (add or edit).
  let editingId = $state<string | null>(null);
  let draftName = $state('');
  let draftType = $state<SelectorType>('css');
  let draftSelector = $state('');
  let draftMode = $state<ExtractionMode>('text');
  let draftAttribute = $state('');

  function resetDraft() {
    editingId = null;
    draftName = '';
    draftType = 'css';
    draftSelector = '';
    draftMode = 'text';
    draftAttribute = '';
    testResult = null;
    testError = null;
  }

  function startEdit(rule: CustomExtraction) {
    editingId = rule.id;
    draftName = rule.name;
    draftType = rule.selector_type;
    draftSelector = rule.selector;
    draftMode = rule.mode;
    draftAttribute = rule.attribute ?? '';
    testResult = null;
    testError = null;
  }

  async function loadRules() {
    loading = true;
    try {
      rules = await api.extraction.getCustomExtractions(projectId);
    } catch (e) {
      toast.error(String(e));
    } finally {
      loading = false;
    }
  }

  async function persist(next: CustomExtraction[]) {
    saving = true;
    try {
      rules = await api.extraction.saveCustomExtractions(projectId, next);
      toast.success(m['extraction.saved']());
    } catch (e) {
      toast.error(String(e));
    } finally {
      saving = false;
    }
  }

  async function saveDraft() {
    const rule: CustomExtraction = {
      id: editingId ?? crypto.randomUUID(),
      name: draftName.trim(),
      selector_type: draftType,
      selector: draftSelector.trim(),
      mode: draftMode,
      attribute: draftMode === 'attribute' ? draftAttribute.trim() || null : null,
      position: editingId ? (rules.find((r) => r.id === editingId)?.position ?? 0) : rules.length,
    };
    const next = editingId ? rules.map((r) => (r.id === editingId ? rule : r)) : [...rules, rule];
    await persist(next);
    resetDraft();
  }

  async function deleteRule(id: string) {
    await persist(rules.filter((r) => r.id !== id));
  }

  async function testDraft() {
    if (!testUrl.trim()) {
      testError = m['extraction.test_url_required']();
      testResult = null;
      return;
    }
    testing = true;
    testResult = null;
    testError = null;
    try {
      const value = await api.extraction.testCustomExtraction(
        {
          id: editingId ?? crypto.randomUUID(),
          name: draftName.trim() || 'test',
          selector_type: draftType,
          selector: draftSelector.trim(),
          mode: draftMode,
          attribute: draftMode === 'attribute' ? draftAttribute.trim() || null : null,
          position: 0,
        },
        testUrl.trim()
      );
      testResult = value === null ? m['extraction.test_no_match']() : String(value);
    } catch (e) {
      testError = String(e);
    } finally {
      testing = false;
    }
  }

  async function recompute() {
    recomputing = true;
    try {
      const count = await api.extraction.recomputeCustomExtractions(projectId);
      toast.success(m['extraction.recomputed']({ count }));
    } catch (e) {
      toast.error(String(e));
    } finally {
      recomputing = false;
    }
  }

  function typeLabel(t: SelectorType): string {
    return t === 'css'
      ? m['extraction.type_css']()
      : t === 'xpath'
        ? m['extraction.type_xpath']()
        : m['extraction.type_regex']();
  }

  function modeLabel(mode: ExtractionMode): string {
    return mode === 'text'
      ? m['extraction.mode_text']()
      : mode === 'attribute'
        ? m['extraction.mode_attribute']()
        : m['extraction.mode_count']();
  }

  $effect(() => {
    loadRules();
  });
</script>

<section class="extraction-panel">
  <div class="panel-head">
    <div>
      <h2>{m['extraction.title']()}</h2>
      <p class="muted">{m['extraction.description']()}</p>
    </div>
    <Button variant="outline" onclick={recompute} disabled={recomputing || rules.length === 0}>
      <RefreshCw class="size-4" />
      {recomputing ? m['extraction.recomputing']() : m['extraction.recompute']()}
    </Button>
  </div>

  {#if loading}
    <Skeleton class="h-24 w-full" />
  {:else if rules.length === 0 && !editingId}
    <p class="muted">{m['extraction.empty']()}</p>
  {:else}
    <div class="rules">
      {#each rules as rule (rule.id)}
        <Card>
          <CardContent class="flex items-start justify-between gap-4 px-4 py-3">
            <div class="rule-info">
              <div class="rule-name">
                <strong>{rule.name}</strong>
                <Badge variant="secondary">{typeLabel(rule.selector_type)}</Badge>
                <Badge variant="outline">{modeLabel(rule.mode)}</Badge>
              </div>
              <code class="rule-selector">{rule.selector}</code>
              {#if rule.mode === 'attribute' && rule.attribute}
                <span class="muted">@ {rule.attribute}</span>
              {/if}
            </div>
            <div class="rule-actions">
              <Button
                variant="ghost"
                size="icon"
                onclick={() => startEdit(rule)}
                title={m['extraction.edit_rule']()}
              >
                <Pencil class="size-4" />
              </Button>
              <Button
                variant="ghost"
                size="icon"
                onclick={() => deleteRule(rule.id)}
                title={m['extraction.delete_rule']()}
              >
                <Trash2 class="size-4" />
              </Button>
            </div>
          </CardContent>
        </Card>
      {/each}
    </div>
  {/if}

  <div class="form">
    <h3>{editingId ? m['extraction.edit_rule']() : m['extraction.add_rule']()}</h3>
    <div class="grid gap-1.5">
      <Label for="ext-name">{m['extraction.name']()}</Label>
      <Input
        id="ext-name"
        bind:value={draftName}
        placeholder={m['extraction.name_placeholder']()}
      />
    </div>
    <div class="form-row">
      <div class="grid gap-1.5">
        <Label for="ext-type">{m['extraction.type']()}</Label>
        <Select.Root type="single" bind:value={draftType}>
          <Select.Trigger id="ext-type" class="w-full">
            <span data-slot="select-value">{typeLabel(draftType)}</span>
          </Select.Trigger>
          <Select.Content>
            <Select.Item value="css" label={m['extraction.type_css']()} />
            <Select.Item value="xpath" label={m['extraction.type_xpath']()} />
            <Select.Item value="regex" label={m['extraction.type_regex']()} />
          </Select.Content>
        </Select.Root>
      </div>
      <div class="grid gap-1.5">
        <Label for="ext-mode">{m['extraction.mode']()}</Label>
        <Select.Root type="single" bind:value={draftMode}>
          <Select.Trigger id="ext-mode" class="w-full">
            <span data-slot="select-value">{modeLabel(draftMode)}</span>
          </Select.Trigger>
          <Select.Content>
            <Select.Item value="text" label={m['extraction.mode_text']()} />
            <Select.Item value="attribute" label={m['extraction.mode_attribute']()} />
            <Select.Item value="count" label={m['extraction.mode_count']()} />
          </Select.Content>
        </Select.Root>
      </div>
    </div>
    <div class="grid gap-1.5">
      <Label for="ext-selector">{m['extraction.selector']()}</Label>
      <Input
        id="ext-selector"
        bind:value={draftSelector}
        placeholder={m['extraction.selector_placeholder']()}
        class="font-mono"
      />
    </div>
    {#if draftMode === 'attribute'}
      <div class="grid gap-1.5">
        <Label for="ext-attribute">{m['extraction.attribute']()}</Label>
        <Input
          id="ext-attribute"
          bind:value={draftAttribute}
          placeholder={m['extraction.attribute_placeholder']()}
          class="font-mono"
        />
      </div>
    {/if}
    <div class="grid gap-1.5">
      <Label for="ext-test-url">{m['extraction.test_url']()}</Label>
      <Input
        id="ext-test-url"
        bind:value={testUrl}
        placeholder={m['extraction.test_url_placeholder']()}
      />
    </div>
    {#if testResult !== null}
      <p class="test-result">{m['extraction.test_result']()}: <strong>{testResult}</strong></p>
    {/if}
    {#if testError !== null}
      <p class="test-error">{testError}</p>
    {/if}
    <div class="form-actions">
      <Button
        variant="outline"
        onclick={testDraft}
        disabled={testing || !draftSelector.trim() || !testUrl.trim()}
      >
        <FlaskConical class="size-4" />
        {testing ? m['extraction.testing']() : m['extraction.test']()}
      </Button>
      {#if editingId}
        <Button variant="ghost" onclick={resetDraft}>
          <X class="size-4" />
          {m['extraction.cancel']()}
        </Button>
      {/if}
      <Button onclick={saveDraft} disabled={saving || !draftName.trim() || !draftSelector.trim()}>
        <Save class="size-4" />
        {m['extraction.save_rule']()}
      </Button>
    </div>
    <p class="muted help">{m['extraction.help']()}</p>
  </div>
</section>

<style>
  .extraction-panel {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    padding: 1rem 0;
  }
  .panel-head {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 1rem;
  }
  .panel-head h2 {
    font-size: 1.125rem;
    font-weight: 600;
    margin: 0 0 0.25rem;
  }
  .muted {
    color: var(--muted-foreground);
    font-size: 0.875rem;
  }
  .rules {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  .rule-info {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    min-width: 0;
  }
  .rule-name {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-wrap: wrap;
  }
  .rule-selector {
    font-size: 0.8125rem;
    word-break: break-all;
  }
  .rule-actions {
    display: flex;
    gap: 0.25rem;
    flex-shrink: 0;
  }
  .form {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 1rem;
  }
  .form h3 {
    font-size: 0.9375rem;
    font-weight: 600;
    margin: 0;
  }
  .form-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0.75rem;
  }
  .form-actions {
    display: flex;
    gap: 0.5rem;
    flex-wrap: wrap;
  }
  .test-result {
    font-size: 0.875rem;
  }
  .test-error {
    font-size: 0.875rem;
    color: var(--destructive);
  }
  .help {
    margin: 0;
  }
</style>
