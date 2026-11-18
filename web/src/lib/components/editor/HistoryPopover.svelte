<script lang="ts">
  import Popover from '#lib/components/Popover.svelte';
  import HistoryList from '#lib/components/editor/HistoryList.svelte';
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
  <div class="flex items-center justify-between border-b border-dark/5 px-3 py-2 touch:hidden">
    <span class="text-[11px] uppercase tracking-wider text-dark/65">History</span>
    <IconButton
      size="small"
      variant="ghost"
      color="secondary"
      icon={mdiClose}
      aria-label="Close"
      onclick={close}
    />
  </div>
  <HistoryList onRestored={close} />
</Popover>
