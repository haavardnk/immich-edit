<script lang="ts">
  import { assetThumbUrl } from '#lib/api/assets.ts';
  import HighlightedText from '#lib/components/HighlightedText.svelte';
  import TooltipLink from '#lib/components/TooltipLink.svelte';
  import type { AlbumSummary } from '#lib/types/album.ts';
  import { countOf } from '#lib/utils/plural.ts';
  import { Icon } from '@immich/ui';
  import { mdiImageAlbum } from '@mdi/js';

  let { album, query = '' }: { album: AlbumSummary; query?: string } = $props();
</script>

<TooltipLink
  tooltip={album.albumName}
  href={`/albums/${album.id}`}
  class="group flex min-w-0 flex-col gap-0.5 rounded-lg outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2"
>
  <span
    class="no-callout mb-1.5 flex aspect-square items-center justify-center overflow-hidden rounded-xl bg-light-200 text-dark/40"
  >
    {#if album.albumThumbnailAssetId}
      <img
        src={assetThumbUrl(album.albumThumbnailAssetId)}
        alt=""
        loading="lazy"
        class="size-full object-cover transition-transform group-hover:scale-[1.02]"
      />
    {:else}
      <Icon icon={mdiImageAlbum} size="40px" aria-hidden="true" />
    {/if}
  </span>
  <span
    data-truncate
    class="block min-w-0 truncate text-sm font-medium text-light-950 touch:text-[14.5px]"
    ><HighlightedText text={album.albumName} {query} /></span
  >
  <span class="block text-xs text-dark/65 touch:text-[12.5px]">
    {countOf(album.assetCount, 'photo')}
  </span>
</TooltipLink>
