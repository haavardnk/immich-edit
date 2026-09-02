<script lang="ts">
  import { toasts, type ToastKind } from '$lib/stores/toasts.svelte';
  import { Toast, type Color } from '@immich/ui';

  const colors: Record<ToastKind, Color> = {
    error: 'danger',
    warn: 'warning',
    success: 'success',
    info: 'info'
  };
</script>

<div class="fixed bottom-4 right-4 z-50 flex flex-col items-end gap-2 pointer-events-none">
  {#each toasts.items as toast (toast.id)}
    <div
      role={toast.kind === 'error' ? 'alert' : 'status'}
      aria-live={toast.kind === 'error' ? 'assertive' : 'polite'}
      aria-atomic="true"
    >
      <Toast
        class="pointer-events-auto"
        size="small"
        color={colors[toast.kind]}
        icon={false}
        title={toast.message}
        onClose={() => toasts.dismiss(toast.id)}
      />
    </div>
  {/each}
</div>
