<script lang="ts">
  import { ALBUM_ACTIONS, albumMenu, type AlbumMenuPlace } from '#lib/browse/albumActions.ts';
  import { albumRole } from '#lib/browse/albumRole.ts';
  import ActionMenu from '#lib/components/ActionMenu.svelte';
  import { albumActions } from '#lib/stores/albumActions.svelte.ts';
  import { session } from '#lib/stores/session.svelte.ts';
  import type { AlbumSummary } from '#lib/types/album.ts';

  let {
    album,
    place,
    variant,
    onrun
  }: {
    album: AlbumSummary;
    place: AlbumMenuPlace;
    variant: 'context' | 'popover';
    onrun?: () => void;
  } = $props();

  const groups = $derived(albumMenu(albumRole(album, session.user?.id ?? null), place));
</script>

<ActionMenu
  {groups}
  actions={ALBUM_ACTIONS}
  {variant}
  onrun={(action) => {
    onrun?.();
    void albumActions.run(action, album);
  }}
/>
