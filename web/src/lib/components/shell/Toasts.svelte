<script lang="ts">
  import { toasts, type ToastAction, type ToastKind } from '#lib/stores/toasts.svelte.ts';
  import { jobs } from '#lib/stores/jobs.svelte.ts';
  import JobToast from '#lib/components/jobs/JobToast.svelte';
  import { Toast, type Color, type ToastButton } from '@immich/ui';

  const colors: Record<ToastKind, Color> = {
    error: 'danger',
    warn: 'warning',
    success: 'success',
    info: 'info'
  };

  function actionButton(action: ToastAction): (close: () => void) => ToastButton {
    return (close) => ({
      label: action.label,
      variant: 'ghost',
      onclick: () => {
        close();
        action.run();
      }
    });
  }
</script>

<div
  class="pointer-events-none fixed right-[calc(1rem+var(--safe-right))] bottom-[calc(1rem+var(--chrome-bottom,var(--safe-bottom)))] z-50 flex flex-col items-end gap-2 *:motion-safe:transition-[opacity,translate] *:motion-safe:duration-base *:motion-safe:ease-standard *:starting:translate-y-2 *:starting:opacity-0"
>
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
        description={toast.description}
        button={toast.action ? actionButton(toast.action) : undefined}
        onClose={() => toasts.dismiss(toast.id)}
      />
    </div>
  {/each}
</div>
