<script lang="ts">
  import { takenLabel } from '#lib/browse/info/photoInfo.ts';
  import { INFO_SHEET_SHARE } from '#lib/browse/loupeLayout.ts';
  import LoupeInfoBody from '#lib/components/browse/info/LoupeInfoBody.svelte';
  import BottomSheet from '#lib/components/sheet/BottomSheet.svelte';
  import { browseView } from '#lib/stores/browseView.svelte.ts';
  import type { AssetSummary } from '#lib/types/album.ts';

  let { asset }: { asset: AssetSummary } = $props();

  const subtitle = $derived(takenLabel(asset) ?? undefined);
</script>

<BottomSheet
  open
  onOpenChange={(open) => browseView.setLoupeInfoOpen(open)}
  title="Info"
  {subtitle}
  close="icon"
  closeLabel="Close info"
  modal={false}
  detents={[{ fraction: INFO_SHEET_SHARE }, { fraction: 1 }]}
  detentLabels={['Half height', 'Full height']}
>
  <LoupeInfoBody {asset} />
</BottomSheet>
