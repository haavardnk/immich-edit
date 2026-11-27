<script lang="ts">
  import { takenLabel } from '#lib/browse/info/photoInfo.ts';
  import { INFO_SHEET_SHARE } from '#lib/browse/loupeLayout.ts';
  import PhotoInfoBody from '#lib/components/browse/info/PhotoInfoBody.svelte';
  import BottomSheet from '#lib/components/sheet/BottomSheet.svelte';
  import { presence } from '#lib/components/sheet/presence.svelte.ts';
  import type { AssetSummary } from '#lib/types/album.ts';
  import { untrack } from 'svelte';

  let { asset, open, onclose }: { asset: AssetSummary | null; open: boolean; onclose: () => void } =
    $props();

  const sheet = presence(() => open);
  let atRest = $state(untrack(() => open));

  function exited(): void {
    atRest = false;
    sheet.exited();
  }
  const subtitle = $derived(asset ? (takenLabel(asset) ?? undefined) : undefined);
</script>

{#if sheet.mounted && asset}
  <BottomSheet
    {open}
    onOpenChange={(next) => {
      if (!next) onclose();
    }}
    onExited={exited}
    appear={!atRest}
    title="Info"
    {subtitle}
    close="icon"
    closeLabel="Close info"
    modal={false}
    detents={[{ fraction: INFO_SHEET_SHARE }, { fraction: 1 }]}
    detentLabels={['Half height', 'Full height']}
  >
    <PhotoInfoBody {asset} />
  </BottomSheet>
{/if}
