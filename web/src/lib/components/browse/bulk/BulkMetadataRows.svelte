<script lang="ts">
  import MenuRow from '#lib/components/MenuRow.svelte';
  import BulkActionRow from '#lib/components/browse/bulk/BulkActionRow.svelte';
  import { bulkActions } from '#lib/browse/bulk/bulkActions.svelte.ts';
  import type { BulkMenuAction } from '#lib/browse/bulk/bulkMenu.ts';
  import { bulkMetadata } from '#lib/browse/bulk/bulkMetadata.svelte.ts';
  import { Icon } from '@immich/ui';
  import { mdiChevronRight, mdiMapMarkerOutline } from '@mdi/js';

  let { onDone, onLocation }: { onDone: () => void; onLocation: () => void } = $props();

  function pick(action: BulkMenuAction): void {
    onDone();
    action.run();
  }
</script>

<div class="flex flex-col" role="group" aria-label="Metadata">
  <BulkActionRow action={bulkMetadata.captureTime} onPick={pick} />
  <BulkActionRow action={bulkMetadata.description} onPick={pick} />
  <MenuRow
    icon={mdiMapMarkerOutline}
    label="Location"
    aria-label="Location"
    disabled={bulkActions.metaBusy}
    onclick={onLocation}
  >
    <Icon icon={mdiChevronRight} size="16px" class="shrink-0 opacity-70" aria-hidden="true" />
  </MenuRow>
</div>
