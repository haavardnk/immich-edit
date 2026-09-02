<script lang="ts">
  import { IconButton } from '@immich/ui';
  import { mdiCheck, mdiClose, mdiDelete } from '@mdi/js';

  let {
    pending = $bindable(false),
    label,
    title = label,
    confirmLabel,
    size = 'tiny',
    round = false,
    disabled = false,
    deleteClass,
    confirmClass,
    onconfirm
  }: {
    pending?: boolean;
    label: string;
    title?: string;
    confirmLabel: string;
    size?: 'tiny' | 'small';
    round?: boolean;
    disabled?: boolean;
    deleteClass?: string;
    confirmClass?: string;
    onconfirm: () => void | Promise<void>;
  } = $props();

  let busy = $state(false);

  function stop(event: MouseEvent): void {
    event.preventDefault();
    event.stopPropagation();
  }

  async function confirm(event: MouseEvent): Promise<void> {
    stop(event);
    if (busy) return;
    busy = true;
    try {
      await onconfirm();
      pending = false;
    } finally {
      busy = false;
    }
  }

  function cancel(event: MouseEvent): void {
    stop(event);
    pending = false;
  }
</script>

{#if pending}
  <IconButton
    type="button"
    {size}
    shape={round ? 'round' : undefined}
    variant="ghost"
    color="primary"
    class={confirmClass}
    icon={mdiCheck}
    title="Confirm delete"
    aria-label={confirmLabel}
    disabled={busy}
    onclick={(event: MouseEvent) => void confirm(event)}
  />
  <IconButton
    type="button"
    {size}
    shape={round ? 'round' : undefined}
    variant="ghost"
    color="secondary"
    class={deleteClass}
    icon={mdiClose}
    title="Cancel"
    aria-label="Cancel delete"
    disabled={busy}
    onclick={cancel}
  />
{:else}
  <IconButton
    type="button"
    {size}
    shape={round ? 'round' : undefined}
    variant="ghost"
    color="secondary"
    class={deleteClass}
    icon={mdiDelete}
    {title}
    aria-label={label}
    {disabled}
    onclick={(event: MouseEvent) => {
      stop(event);
      pending = true;
    }}
  />
{/if}
