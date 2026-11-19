<script lang="ts">
  import EditableLabel from '#lib/components/EditableLabel.svelte';
  import { Button, IconButton } from '@immich/ui';
  import { mdiCheck, mdiClose } from '@mdi/js';
  import { untrack } from 'svelte';

  let {
    initial,
    ariaLabel,
    onsave,
    oncancel
  }: {
    initial: string;
    ariaLabel: string;
    onsave: (name: string) => Promise<void>;
    oncancel: () => void;
  } = $props();

  let value = $state(untrack(() => initial));
  let busy = $state(false);

  async function save(): Promise<void> {
    if (busy || !value.trim()) return;
    busy = true;
    try {
      await onsave(value);
    } finally {
      busy = false;
    }
  }
</script>

<div class="flex min-w-0 flex-1 items-center gap-0.5 touch:gap-2">
  <div class="min-w-0 flex-1">
    <EditableLabel compact {ariaLabel} bind:value oncommit={save} {oncancel} />
  </div>
  <IconButton
    size="tiny"
    variant="ghost"
    color="secondary"
    class="touch:hidden"
    icon={mdiCheck}
    title="Save"
    aria-label="Save name"
    disabled={busy}
    onclick={() => void save()}
  />
  <IconButton
    size="tiny"
    variant="ghost"
    color="secondary"
    class="touch:hidden"
    icon={mdiClose}
    title="Cancel"
    aria-label="Cancel"
    onclick={oncancel}
  />
  <Button
    size="small"
    variant="ghost"
    color="primary"
    class="hidden min-h-11 font-semibold touch:inline-flex"
    disabled={busy}
    onclick={() => void save()}
  >
    Save
  </Button>
</div>
