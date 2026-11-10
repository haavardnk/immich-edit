<script lang="ts">
  import MenuItem from '#lib/components/MenuItem.svelte';
  import ActionSheet from '#lib/components/sheet/ActionSheet.svelte';
  import {
    mdiCameraRetakeOutline,
    mdiCheck,
    mdiClose,
    mdiDeleteOutline,
    mdiPencilOutline
  } from '@mdi/js';
  import type { SnapshotAction } from './snapshotAction.ts';

  let {
    open,
    name,
    onOpenChange,
    onexited,
    onaction
  }: {
    open: boolean;
    name: string;
    onOpenChange: (open: boolean) => void;
    onexited: () => void;
    onaction: (action: SnapshotAction) => void;
  } = $props();

  let confirming = $state(false);
  let chosen: SnapshotAction | null = null;

  function choose(action: SnapshotAction): void {
    chosen = action;
  }

  function exited(): void {
    confirming = false;
    const action = chosen;
    chosen = null;
    onexited();
    if (action) onaction(action);
  }
</script>

<ActionSheet {open} {onOpenChange} onExited={exited} title={name} caption>
  <MenuItem icon={mdiPencilOutline} label="Rename" onclick={() => choose('rename')} />
  <MenuItem
    icon={mdiCameraRetakeOutline}
    label="Update with current"
    onclick={() => choose('update')}
  />
  {#if confirming}
    <MenuItem icon={mdiCheck} label="Confirm delete" onclick={() => choose('delete')} />
    <MenuItem icon={mdiClose} label="Cancel" data-menu-stay onclick={() => (confirming = false)} />
  {:else}
    <MenuItem
      icon={mdiDeleteOutline}
      label="Delete"
      data-menu-stay
      onclick={() => (confirming = true)}
    />
  {/if}
</ActionSheet>
