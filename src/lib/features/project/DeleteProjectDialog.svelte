<script lang="ts">
  import { m } from '$lib/paraglide/messages.js';
  import { Button } from '$lib/components/ui/button/index.js';
  import * as AlertDialog from '$lib/components/ui/alert-dialog/index.js';

  let {
    open = $bindable(),
    pendingId,
    onDelete,
    onClose,
  }: {
    open: boolean;
    pendingId: string | null;
    onDelete: (id: string) => void;
    onClose: () => void;
  } = $props();
</script>

<AlertDialog.Root bind:open={open} onOpenChange={(o) => { if (!o) onClose(); }}>
  <AlertDialog.Content class="max-w-md">
    <AlertDialog.Header>
      <AlertDialog.Title>{m['dialog.delete_title']()}</AlertDialog.Title>
      <AlertDialog.Description>
        {m['dialog.delete_confirm']()}
      </AlertDialog.Description>
    </AlertDialog.Header>
    <AlertDialog.Footer>
      <Button variant="outline" onclick={onClose}>
        {m['settings.cancel']()}
      </Button>
      <Button variant="destructive" onclick={() => pendingId && onDelete(pendingId)}>
        {m['sidebar.delete']()}
      </Button>
    </AlertDialog.Footer>
  </AlertDialog.Content>
</AlertDialog.Root>
