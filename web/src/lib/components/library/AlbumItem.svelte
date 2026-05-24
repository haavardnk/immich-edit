<script lang="ts">
  import type { AlbumSummary } from '$lib/types/album';
  import { assetThumbUrl } from '$lib/api/assets';

  let { album, active = false }: { album: AlbumSummary; active?: boolean } = $props();
</script>

<a
  href={`/albums/${album.id}`}
  class="flex items-center gap-2.5 py-1.5 px-2.5 rounded-lg transition-colors {active ? 'bg-immich-dark-primary/15 text-immich-dark-primary' : 'hover:bg-white/5'}"
>
  <div class="w-8 h-8 rounded-lg bg-white/5 overflow-hidden flex-none">
    {#if album.albumThumbnailAssetId}
      <img
        src={assetThumbUrl(album.albumThumbnailAssetId)}
        alt=""
        loading="lazy"
        class="w-full h-full object-cover"
      />
    {/if}
  </div>
  <div class="flex-1 min-w-0 pr-2">
    <div class="truncate text-[13px] leading-tight">{album.albumName}</div>
    <div class="text-[10px] text-immich-dark-fg/30">{album.assetCount}</div>
  </div>
</a>
