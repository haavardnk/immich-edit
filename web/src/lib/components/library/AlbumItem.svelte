<script lang="ts">
  import type { AlbumSummary } from '#lib/types/album.ts';
  import { assetThumbUrl } from '#lib/api/assets.ts';
  import HighlightedText from '#lib/components/HighlightedText.svelte';
  import TooltipLink from '#lib/components/TooltipLink.svelte';
  import AlbumContextMenu from '#lib/components/library/AlbumContextMenu.svelte';

  let {
    album,
    active = false,
    query = ''
  }: { album: AlbumSummary; active?: boolean; query?: string } = $props();
</script>

<AlbumContextMenu {album} place="list">
  {#snippet trigger(props)}
    <TooltipLink
      {...props}
      tooltip={album.albumName}
      href={`/albums/${album.id}`}
      aria-current={active ? 'page' : undefined}
      class="no-callout flex items-center gap-2.5 rounded-e-full py-1.5 ps-2.5 pe-4 outline-none transition-colors focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-primary {active
        ? 'bg-primary/15 text-primary'
        : 'hover:bg-subtle hover:text-primary'}"
    >
      <div
        class="no-callout h-8 w-8 flex-none overflow-hidden rounded-md bg-gray-100 dark:bg-gray-800"
      >
        {#if album.albumThumbnailAssetId}
          <img
            src={assetThumbUrl({ id: album.albumThumbnailAssetId })}
            alt=""
            loading="lazy"
            class="w-full h-full object-cover"
          />
        {/if}
      </div>
      <div class="min-w-0 flex-1 pe-2">
        <div data-truncate class="truncate text-label leading-tight">
          <HighlightedText text={album.albumName} {query} />
        </div>
        <div class="text-micro text-muted">{album.assetCount}</div>
      </div>
    </TooltipLink>
  {/snippet}
</AlbumContextMenu>
