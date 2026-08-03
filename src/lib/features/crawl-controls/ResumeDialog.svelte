<script lang="ts">
  import { m } from '$lib/paraglide/messages.js';
  import { Button } from '$lib/components/ui/button/index.js';
  import * as AlertDialog from '$lib/components/ui/alert-dialog/index.js';
  import type { ResumableInfo } from '$lib/app.svelte';

  let {
    open = $bindable(),
    resumableInfo,
    elapsedLabel,
    onFreshStart,
    onResume,
  }: {
    open: boolean;
    resumableInfo: ResumableInfo | null;
    elapsedLabel: string;
    onFreshStart: () => void;
    onResume: () => void;
  } = $props();
</script>

{#if resumableInfo}
  <AlertDialog.Root bind:open={open}>
    <AlertDialog.Content class="max-w-md">
      <AlertDialog.Header>
        <AlertDialog.Title>{m['resume.title']()}</AlertDialog.Title>
        <AlertDialog.Description>
          {m['resume.found']({
            pages: resumableInfo.pages_crawled,
            urls: resumableInfo.queue_remaining,
            time: elapsedLabel,
          })}
        </AlertDialog.Description>
      </AlertDialog.Header>
      <AlertDialog.Footer>
        <Button variant="outline" onclick={onFreshStart}>
          {m['resume.fresh_btn']()}
        </Button>
        <Button variant="outline" onclick={() => (open = false)}>
          {m['resume.cancel']()}
        </Button>
        <Button onclick={onResume}>
          {m['resume.resume_btn']()}
        </Button>
      </AlertDialog.Footer>
    </AlertDialog.Content>
  </AlertDialog.Root>
{/if}
