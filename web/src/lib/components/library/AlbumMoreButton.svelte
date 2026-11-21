<script lang="ts">
  import type { AlbumMenuPlace } from '#lib/browse/albumActions.ts';
  import { albumCaption } from '#lib/browse/albumSharing.ts';
  import Popover from '#lib/components/Popover.svelte';
  import AlbumMenu from '#lib/components/library/AlbumMenu.svelte';
  import { layout } from '#lib/device/layout.svelte.ts';
  import { session } from '#lib/stores/session.svelte.ts';
  import type { AlbumSummary } from '#lib/types/album.ts';
  import { IconButton } from '@immich/ui';
  import { mdiDotsHorizontal } from '@mdi/js';

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

  let open = $state(false);
</script>

<Popover
  {open}
  onOpenChange={(next) => (open = next)}
  tooltip="Album actions"
  title={album.albumName}
  subtitle={albumCaption(album, session.user?.id ?? null)}
  contentClass="w-52 py-1"
>
  {#snippet trigger(props)}
    <IconButton
      {...props}
      size={layout.mode === 'touch' ? 'medium' : size}
      variant="ghost"
      color="secondary"
      shape="round"
      class="touch:size-11 {className}"
      icon={mdiDotsHorizontal}
      title=""
      aria-label="Album actions"
    />
  {/snippet}
  <AlbumMenu {album} {place} variant="popover" onrun={() => (open = false)} />
</Popover>
