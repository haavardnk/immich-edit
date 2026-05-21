<script lang="ts">
  import type { AssetSummary } from '$lib/types/album';
  import { thumbUrl } from '$lib/api/assets';

  let { asset }: { asset: AssetSummary } = $props();
</script>

<a
  href={`/assets/${asset.id}`}
  class="block aspect-square overflow-hidden bg-base-300 rounded hover:ring-2 hover:ring-primary transition"
  title={asset.originalFileName}
>
  <img
    src={thumbUrl(asset.id)}
    alt={asset.originalFileName}
    loading="lazy"
    class="object-cover w-full h-full"
  />
</a>
