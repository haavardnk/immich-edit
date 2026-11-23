<script lang="ts">
  import BulkActionRow from '#lib/components/browse/bulk/BulkActionRow.svelte';
  import { bulkLocation } from '#lib/browse/bulk/bulkLocation.svelte.ts';
  import type { BulkMenuAction } from '#lib/browse/bulk/bulkMenu.ts';

  let { onDone }: { onDone: () => void } = $props();

  function pick(action: BulkMenuAction): void {
    onDone();
    action.run();
  }
</script>

<div class="flex flex-col" role="group" aria-label="Location">
  <BulkActionRow action={bulkLocation.choose} onPick={pick} />
  <BulkActionRow action={bulkLocation.paste} onPick={pick} />
  <BulkActionRow action={bulkLocation.copy} onPick={pick} />
</div>
