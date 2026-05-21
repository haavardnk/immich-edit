<script lang="ts">
  import type { AssetSummary } from '$lib/types/album';
  import { thumbUrl } from '$lib/api/assets';

  let { asset, active = false }: { asset: AssetSummary; active?: boolean } = $props();
</script>

<a
  href={`/assets/${asset.id}`}
  class="block aspect-square overflow-hidden bg-base-100 rounded-sm group relative"
  class:ring-2={active}
  class:ring-primary={active}
  title={asset.originalFileName}
>
  <img
    src={thumbUrl(asset.id)}
    alt=""
    loading="lazy"
    class="object-cover w-full h-full transition-opacity opacity-90 group-hover:opacity-100"
  />
  <div
    class="absolute inset-x-0 bottom-0 px-1 py-0.5 text-[10px] truncate bg-gradient-to-t from-black/70 to-transparent opacity-0 group-hover:opacity-100"
  >
    {asset.originalFileName}
  </div>
</a>
