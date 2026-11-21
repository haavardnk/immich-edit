<script lang="ts">
  import { assetThumbUrl } from '#lib/api/assets.ts';
  import HighlightedText from '#lib/components/HighlightedText.svelte';
  import TooltipLink from '#lib/components/TooltipLink.svelte';
  import AlbumContextMenu from '#lib/components/library/AlbumContextMenu.svelte';
  import AlbumMoreButton from '#lib/components/library/AlbumMoreButton.svelte';
  import { layout } from '#lib/device/layout.svelte.ts';
  import type { AlbumSummary } from '#lib/types/album.ts';
  import { countOf } from '#lib/utils/plural.ts';
  import { Icon } from '@immich/ui';
  import { mdiAccountMultiple, mdiImageAlbum } from '@mdi/js';

  let { album, query = '' }: { album: AlbumSummary; query?: string } = $props();

  const desktop = $derived(layout.mode !== 'touch');
</script>

<div class="group relative flex min-w-0 flex-col">
  <AlbumContextMenu {album} place="list">
    {#snippet trigger(props)}
      <TooltipLink
        {...props}
        tooltip={album.albumName}
        href={`/albums/${album.id}`}
        class="flex min-w-0 flex-col gap-0.5 rounded-lg outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2"
      >
        <span
          class="no-callout relative mb-1.5 flex aspect-square items-center justify-center overflow-hidden rounded-xl bg-light-200 text-dark/45"
        >
          {#if album.albumThumbnailAssetId}
            <img
              src={assetThumbUrl({ id: album.albumThumbnailAssetId })}
              alt=""
              loading="lazy"
              class="size-full object-cover transition-transform group-hover:scale-[1.02]"
            />
          {:else}
            <Icon icon={mdiImageAlbum} size="40px" aria-hidden="true" />
          {/if}
          {#if album.shared}
            <span
              class="absolute top-2 right-2 flex size-7 items-center justify-center rounded-full bg-light-50/85 text-light-950 ring-1 ring-hairline backdrop-blur-md"
              role="img"
              aria-label="Shared"
            >
              <Icon icon={mdiAccountMultiple} size="16px" aria-hidden="true" />
            </span>
          {/if}
        </span>
        <span
          data-truncate
          class="block min-w-0 truncate text-sm font-medium text-light-950 touch:text-body {desktop
            ? 'group-focus-within:pe-8 group-hover:pe-8'
            : ''}"
        >
          <HighlightedText text={album.albumName} {query} />
        </span>
        <span class="block text-xs text-dark/65 touch:text-label">
          {countOf(album.assetCount, 'photo')}
        </span>
      </TooltipLink>
    {/snippet}
  </AlbumContextMenu>
  {#if desktop}
    <div
      class="absolute right-0 bottom-3.5 opacity-0 group-focus-within:opacity-100 group-hover:opacity-100 has-[[aria-expanded=true]]:opacity-100"
    >
      <AlbumMoreButton {album} place="list" size="tiny" />
    </div>
  {/if}
</div>
