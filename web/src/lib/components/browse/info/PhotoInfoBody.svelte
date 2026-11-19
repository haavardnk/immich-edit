<script lang="ts">
  import InfoAlbums from '#lib/components/browse/info/InfoAlbums.svelte';
  import InfoCamera from '#lib/components/browse/info/InfoCamera.svelte';
  import InfoDescription from '#lib/components/browse/info/InfoDescription.svelte';
  import InfoFile from '#lib/components/browse/info/InfoFile.svelte';
  import InfoPlace from '#lib/components/browse/info/InfoPlace.svelte';
  import InfoTags from '#lib/components/browse/info/InfoTags.svelte';
  import type { AssetSummary } from '#lib/types/album.ts';

  let { asset }: { asset: AssetSummary } = $props();
</script>

<div class="flex flex-col divide-y divide-hairline">
  <InfoDescription {asset} />
  <InfoCamera {asset} />
  <InfoPlace {asset} />
  <InfoAlbums {asset} />
  <InfoTags {asset} />
  <InfoFile {asset} />
</div>
