<script lang="ts">
  import { toasts } from '$lib/stores/toasts.svelte';
  import Icon from '$lib/components/Icon.svelte';
  import { mdiAlertCircleOutline, mdiAlertOutline, mdiInformationOutline, mdiClose } from '@mdi/js';

  const iconFor = (kind: string): string => {
    if (kind === 'error') return mdiAlertCircleOutline;
    if (kind === 'warn') return mdiAlertOutline;
    return mdiInformationOutline;
  };
</script>

<div class="fixed bottom-4 right-4 z-50 flex flex-col gap-2 pointer-events-none">
  {#each toasts.items as toast (toast.id)}
    <div
      class="pointer-events-auto flex items-start gap-2.5 px-3.5 py-2.5 rounded-lg shadow-lg border max-w-sm text-xs"
      class:bg-red-950={toast.kind === 'error'}
      class:border-red-500={toast.kind === 'error'}
      class:text-red-100={toast.kind === 'error'}
      class:bg-amber-950={toast.kind === 'warn'}
      class:border-amber-500={toast.kind === 'warn'}
      class:text-amber-100={toast.kind === 'warn'}
      class:bg-immich-dark-gray={toast.kind === 'info'}
      class:border-white={toast.kind === 'info'}
      class:text-immich-dark-fg={toast.kind === 'info'}
    >
      <Icon path={iconFor(toast.kind)} size={16} class="flex-none mt-0.5" />
      <span class="flex-1 leading-relaxed">{toast.message}</span>
      <button
        class="flex-none opacity-60 hover:opacity-100 transition-opacity mt-0.5"
        onclick={() => toasts.dismiss(toast.id)}
        aria-label="dismiss"
      >
        <Icon path={mdiClose} size={14} />
      </button>
    </div>
  {/each}
</div>
