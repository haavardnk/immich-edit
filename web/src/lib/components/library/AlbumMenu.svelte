<script lang="ts">
  import {
    ALBUM_ACTIONS,
    albumMenu,
    type AlbumAction,
    type AlbumMenuPlace
  } from '#lib/browse/albumActions.ts';
  import { albumRole } from '#lib/browse/albumRole.ts';
  import ContextMenuItem from '#lib/components/ContextMenuItem.svelte';
  import MenuRow from '#lib/components/MenuRow.svelte';
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

  function run(action: AlbumAction): void {
    onrun?.();
    void albumActions.run(action, album);
  }
</script>

{#each groups as group, index (group.join())}
  {#if index > 0}
    <div class="my-1 h-px bg-hairline touch:hidden" role="separator"></div>
  {/if}
  {#each group as action (action)}
    {@const info = ALBUM_ACTIONS[action]}
    {#if variant === 'context'}
      <ContextMenuItem icon={info.icon} danger={info.danger} onSelect={() => run(action)}>
        {info.label}
      </ContextMenuItem>
    {:else}
      <MenuRow
        icon={info.icon}
        label={info.label}
        danger={info.danger}
        onclick={() => run(action)}
      />
    {/if}
  {/each}
{/each}
