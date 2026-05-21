<script lang="ts">
  import type { AssetSummary } from '$lib/types/album';
  import { thumbUrl } from '$lib/api/assets';

  let { asset, active = false }: { asset: AssetSummary; active?: boolean } = $props();
</script>

<a
  href={`/assets/${asset.id}`}
  class="block aspect-square overflow-hidden bg-white/5 rounded-lg group relative transition-all"
  class:ring-2={active}
  class:ring-immich-dark-primary={active}
  title={asset.originalFileName}
>
  <img
    src={thumbUrl(asset.id)}
    alt=""
    loading="lazy"
    class="object-cover w-full h-full transition-transform group-hover:scale-105"
  />
  <div
    class="absolute inset-x-0 bottom-0 px-2 py-1 text-[10px] text-white truncate bg-linear-to-t from-black/70 to-transparent opacity-0 group-hover:opacity-100 transition-opacity"
  >
    {asset.originalFileName}
  </div>
</a>
