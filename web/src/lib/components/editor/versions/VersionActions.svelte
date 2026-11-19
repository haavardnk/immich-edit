<script lang="ts">
  import MenuItem from '#lib/components/MenuItem.svelte';
  import ActionSheet from '#lib/components/sheet/ActionSheet.svelte';
  import { mdiCheck, mdiClose, mdiDeleteOutline, mdiEyeOutline, mdiPencilOutline } from '@mdi/js';
  import type { VersionAction } from './versionAction.ts';

  let {
    open,
    name,
    copy,
    onOpenChange,
    onexited,
    onaction
  }: {
    open: boolean;
    name: string;
    copy: boolean;
    onOpenChange: (open: boolean) => void;
    onexited: () => void;
    onaction: (action: VersionAction) => void;
  } = $props();

  let confirming = $state(false);
  let chosen: VersionAction | null = null;

  function choose(action: VersionAction): void {
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
  {#if copy}
    <MenuItem icon={mdiPencilOutline} label="Rename" onclick={() => choose('rename')} />
  {/if}
  <MenuItem icon={mdiEyeOutline} label="Open in loupe" onclick={() => choose('loupe')} />
  {#if copy && confirming}
    <MenuItem icon={mdiCheck} label="Confirm delete" onclick={() => choose('delete')} />
    <MenuItem icon={mdiClose} label="Cancel" data-menu-stay onclick={() => (confirming = false)} />
  {:else if copy}
    <MenuItem
      icon={mdiDeleteOutline}
      label="Delete"
      data-menu-stay
      onclick={() => (confirming = true)}
    />
  {/if}
</ActionSheet>
