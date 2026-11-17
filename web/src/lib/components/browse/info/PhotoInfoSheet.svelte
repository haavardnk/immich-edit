<script lang="ts">
  import { takenLabel } from '#lib/browse/info/photoInfo.ts';
  import { INFO_SHEET_SHARE } from '#lib/browse/loupeLayout.ts';
  import PhotoInfoBody from '#lib/components/browse/info/PhotoInfoBody.svelte';
  import BottomSheet from '#lib/components/sheet/BottomSheet.svelte';
  import type { AssetSummary } from '#lib/types/album.ts';

  let { asset, onclose }: { asset: AssetSummary; onclose: () => void } = $props();

  const subtitle = $derived(takenLabel(asset) ?? undefined);
</script>

<BottomSheet
  open
  onOpenChange={(open) => {
    if (!open) onclose();
  }}
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
