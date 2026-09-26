<script lang="ts">
  import { toasts, type ToastKind } from '$lib/stores/toasts.svelte';
  import { jobs } from '$lib/stores/jobs.svelte';
  import JobToast from '$lib/components/jobs/JobToast.svelte';
  import { Toast, type Color } from '@immich/ui';

  const colors: Record<ToastKind, Color> = {
    error: 'danger',
    warn: 'warning',
    success: 'success',
    info: 'info'
  };
</script>

<div class="fixed bottom-4 right-4 z-50 flex flex-col items-end gap-2 pointer-events-none">
  {#each jobs.watched as id (id)}
    {@const job = jobs.jobs.find((j) => j.id === id)}
    {#if job}
      <JobToast {job} />
    {/if}
  {/each}
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
