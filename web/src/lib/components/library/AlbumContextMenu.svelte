<script lang="ts">
  import type { AlbumMenuPlace } from '#lib/browse/albumActions.ts';
  import { albumCaption } from '#lib/browse/albumSharing.ts';
  import ContextMenu from '#lib/components/ContextMenu.svelte';
  import AlbumMenu from '#lib/components/library/AlbumMenu.svelte';
  import { session } from '#lib/stores/session.svelte.ts';
  import type { AlbumSummary } from '#lib/types/album.ts';
  import type { Snippet } from 'svelte';

  let {
    album,
    place,
    trigger
  }: {
    album: AlbumSummary;
    place: AlbumMenuPlace;
    trigger: Snippet<[Record<string, unknown>]>;
  } = $props();
</script>

<ContextMenu
  title={album.albumName}
  subtitle={albumCaption(album, session.user?.id ?? null)}
  {trigger}
>
  <AlbumMenu {album} {place} variant="context" />
</ContextMenu>
