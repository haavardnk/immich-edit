<script lang="ts">
  import type { AlbumSummary } from '$lib/types/album';
  import { thumbUrl } from '$lib/api/assets';

  let { album }: { album: AlbumSummary } = $props();
</script>

<a href={`/albums/${album.id}`} class="card bg-base-200 shadow-md hover:bg-base-300 transition-colors">
  <figure class="aspect-square overflow-hidden bg-base-300">
    {#if album.albumThumbnailAssetId}
      <img
        src={thumbUrl(album.albumThumbnailAssetId)}
        alt={album.albumName}
        loading="lazy"
        class="object-cover w-full h-full"
      />
    {:else}
      <div class="flex items-center justify-center w-full h-full opacity-40">no thumb</div>
    {/if}
  </figure>
  <div class="card-body p-3">
    <h2 class="card-title text-sm truncate">{album.albumName}</h2>
    <p class="text-xs opacity-60">{album.assetCount} assets</p>
  </div>
</a>
