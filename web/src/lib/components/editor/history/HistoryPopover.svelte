<script lang="ts">
  import Popover from '#lib/components/Popover.svelte';
  import HistoryList from './HistoryList.svelte';
  import SnapshotList from '#lib/components/editor/snapshots/SnapshotList.svelte';
  import { IconButton } from '@immich/ui';
  import { mdiHistory, mdiClose } from '@mdi/js';
  import { editor } from '#lib/stores/editor.svelte.ts';

  let open = $state(false);

  function close(): void {
    open = false;
  }
</script>

<Popover
  {open}
  align="end"
  onOpenChange={(v) => (open = v)}
  appearance="panel"
  contentClass="w-80 max-h-112 overflow-y-auto"
  sheetClass="p-0"
  title="Edit history"
  tooltip="Edit history"
>
  {#snippet trigger(popoverProps)}
    <IconButton
      size="small"
      variant="ghost"
      color="secondary"
      class="bg-transparent hover:bg-light-950/6"
      icon={mdiHistory}
      title=""
      disabled={!editor.assetId}
      aria-label="Edit history"
      {...popoverProps}
    />
  {/snippet}
  <div class="relative border-b border-dark/5 pb-1">
    <SnapshotList />
    <IconButton
      size="tiny"
      variant="ghost"
      color="secondary"
      class="absolute top-1 right-1.5 touch:hidden"
      icon={mdiClose}
      aria-label="Close"
      onclick={close}
    />
  </div>
  <div class="px-3 pt-2 pb-1 touch:hidden">
    <span class="text-[11px] uppercase tracking-wider text-dark/65">History</span>
  </div>
  <HistoryList />
</Popover>
