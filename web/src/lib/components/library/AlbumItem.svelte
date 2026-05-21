<script lang="ts">
  import type { AlbumSummary } from '$lib/types/album';
  import { thumbUrl } from '$lib/api/assets';

  let { album, active = false }: { album: AlbumSummary; active?: boolean } = $props();
</script>

<li>
  <a
    href={`/albums/${album.id}`}
    class="flex items-center gap-2 py-1.5 px-2 rounded"
    class:menu-active={active}
  >
    <div class="w-7 h-7 rounded bg-base-100 overflow-hidden flex-none">
      {#if album.albumThumbnailAssetId}
        <img
          src={thumbUrl(album.albumThumbnailAssetId)}
          alt=""
          loading="lazy"
          class="w-full h-full object-cover"
        />
      {/if}
    </div>
    <div class="flex-1 min-w-0">
      <div class="truncate text-[13px] leading-tight">{album.albumName}</div>
      <div class="text-[10px] opacity-50">{album.assetCount}</div>
    </div>
  </a>
</li>
