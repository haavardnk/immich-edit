<script lang="ts">
  import ViewBody from '#lib/components/browse/view/ViewBody.svelte';
  import BottomSheet from '#lib/components/sheet/BottomSheet.svelte';
  import SheetActionButton from '#lib/components/sheet/SheetActionButton.svelte';
  import { presence } from '#lib/components/sheet/presence.svelte.ts';
  import { layout } from '#lib/device/layout.svelte.ts';
  import { browseView } from '#lib/stores/browseView.svelte.ts';

  let { sortBasis }: { sortBasis: 'capture' | 'edit' | null } = $props();

  const sheet = presence(() => browseView.viewOpen);
</script>

{#snippet done()}
  <SheetActionButton onclick={() => browseView.setViewOpen(false)}>Done</SheetActionButton>
{/snippet}

{#if sheet.mounted}
  <BottomSheet
    open={browseView.viewOpen}
    onOpenChange={(open) => browseView.setViewOpen(open)}
    onExited={sheet.exited}
    title="View"
    close="icon"
    closeLabel="Close view"
    detents={[{ fraction: 1 }]}
    footer={layout.landscape ? undefined : done}
  >
    <ViewBody {sortBasis} />
  </BottomSheet>
{/if}
