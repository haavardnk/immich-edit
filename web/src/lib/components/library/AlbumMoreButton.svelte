<script lang="ts">
  import type { AlbumMenuPlace } from '#lib/browse/albumActions.ts';
  import { albumCaption } from '#lib/browse/albumSharing.ts';
  import MoreActionsButton from '#lib/components/MoreActionsButton.svelte';
  import AlbumMenu from '#lib/components/library/AlbumMenu.svelte';
  import { session } from '#lib/stores/session.svelte.ts';
  import type { AlbumSummary } from '#lib/types/album.ts';

  let {
    album,
    place,
    size = 'small',
    class: className = ''
  }: {
    album: AlbumSummary;
    place: AlbumMenuPlace;
    size?: 'tiny' | 'small';
    class?: string;
  } = $props();
</script>

<MoreActionsButton
  label="Album actions"
  title={album.albumName}
  subtitle={albumCaption(album, session.user?.id ?? null)}
  {size}
  class={className}
>
  {#snippet children(close)}
    <AlbumMenu {album} {place} variant="popover" onrun={close} />
  {/snippet}
</MoreActionsButton>
