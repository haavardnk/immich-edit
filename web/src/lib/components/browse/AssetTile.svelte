<script lang="ts">
  import type { AssetSummary } from '$lib/types/album';
  import { thumbUrl } from '$lib/api/assets';
  import Icon from '$lib/components/Icon.svelte';
  import { mdiHeart, mdiStar } from '@mdi/js';

  let { asset, active = false }: { asset: AssetSummary; active?: boolean } = $props();

  const rating = $derived(asset.exifInfo?.rating ?? 0);
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
  {#if asset.isFavorite}
    <div class="absolute top-1 right-1 text-white drop-shadow-md pointer-events-none">
      <Icon path={mdiHeart} size={16} />
    </div>
  {/if}
  {#if rating > 0}
    <div
      class="absolute top-1 left-1 flex items-center gap-0.5 text-white drop-shadow-md pointer-events-none"
    >
      {#each [1, 2, 3, 4, 5] as n (n)}
        <Icon path={mdiStar} size={12} class={n <= rating ? 'opacity-100' : 'opacity-30'} />
      {/each}
    </div>
  {/if}
  <div
    class="absolute inset-x-0 bottom-0 px-2 py-1 text-[10px] text-white truncate bg-linear-to-t from-black/70 to-transparent opacity-0 group-hover:opacity-100 transition-opacity"
  >
    {asset.originalFileName}
  </div>
</a>
