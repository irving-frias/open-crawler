<script lang="ts">
  let {
    toasts = [],
    onDismiss,
  }: {
    toasts: Array<{ id: number; message: string; type: 'success' | 'error' | 'warning' | 'info' }>;
    onDismiss: (id: number) => void;
  } = $props();

  function getIcon(type: string): string {
    switch (type) {
      case 'success': return '\u2714';
      case 'error': return '\u2716';
      case 'warning': return '\u26A0';
      default: return '\u2139';
    }
  }
</script>

<div class="toast-container" aria-live="polite">
  {#each toasts as toast (toast.id)}
    <div class="toast toast-{toast.type}" role="alert">
      <span class="toast-icon">{getIcon(toast.type)}</span>
      <span class="toast-message">{toast.message}</span>
      <button class="toast-dismiss" onclick={() => onDismiss(toast.id)} aria-label="Dismiss">&times;</button>
    </div>
  {/each}
</div>

<style>
  .toast-container {
    position: fixed;
    top: var(--space-lg, 1rem);
    right: var(--space-lg, 1rem);
    z-index: 9999;
    display: flex;
    flex-direction: column;
    gap: var(--space-sm, 0.5rem);
    max-width: 400px;
  }

  .toast {
    display: flex;
    align-items: center;
    gap: var(--space-sm, 0.5rem);
    padding: var(--space-sm, 0.5rem) var(--space-md, 1rem);
    border-radius: var(--radius-md, 6px);
    box-shadow: 0 4px 12px rgba(0,0,0,0.15);
    animation: slide-in 0.3s ease-out;
    font-size: 0.875rem;
    line-height: 1.4;
  }

  .toast-success {
    background: var(--success-subtle);
    color: var(--success);
    border: 1px solid var(--success);
  }
  .toast-error {
    background: var(--danger-subtle);
    color: var(--danger);
    border: 1px solid var(--danger);
  }
  .toast-warning {
    background: var(--warning-subtle);
    color: var(--warning);
    border: 1px solid var(--warning);
  }
  .toast-info {
    background: var(--info-subtle);
    color: var(--info);
    border: 1px solid var(--info);
  }

  .toast-icon {
    font-size: 1rem;
    flex-shrink: 0;
  }

  .toast-message {
    flex: 1;
    word-break: break-word;
  }

  .toast-dismiss {
    background: none;
    border: none;
    font-size: 1.2rem;
    cursor: pointer;
    opacity: 0.6;
    padding: 0;
    line-height: 1;
    color: inherit;
    flex-shrink: 0;
  }
  .toast-dismiss:hover {
    opacity: 1;
  }

  @keyframes slide-in {
    from { transform: translateX(100%); opacity: 0; }
    to { transform: translateX(0); opacity: 1; }
  }
</style>
