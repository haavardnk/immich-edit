<script lang="ts">
  import { onMount } from 'svelte';
  import AlbumCard from '#lib/components/library/AlbumCard.svelte';
  import AlbumSortMenu from '#lib/components/library/AlbumSortMenu.svelte';
  import LibraryLoad from '#lib/components/library/LibraryLoad.svelte';
  import PageFrame from '#lib/components/shell/PageFrame.svelte';
  import { sortAlbums, type AlbumSort } from '#lib/browse/albumSort.ts';
  import { library } from '#lib/stores/library.svelte.ts';
  import { countOf } from '#lib/utils/plural.ts';

  let sort = $state<AlbumSort>('updated');

  const loaded = $derived(library.isLoaded('albums'));
  const albums = $derived(sortAlbums(library.albums, sort));

  onMount(() => {
    library.request('albums');
  });
</script>

{#snippet actions()}
  <AlbumSortMenu value={sort} onChange={(next) => (sort = next)} />
{/snippet}

<PageFrame title="Albums" count={loaded ? countOf(albums.length, 'album') : undefined} {actions}>
  <LibraryLoad view="albums" label="albums">
    {#if albums.length === 0}
      <p class="px-6 py-16 text-center text-sm text-dark/65" role="status">No albums</p>
    {:else}
      <ul
        aria-label="Albums"
        class="grid grid-cols-2 gap-x-3 gap-y-4.5 px-4 pt-1.5 pb-4 sm:grid-cols-[repeat(auto-fill,minmax(10rem,1fr))] sm:gap-x-4 sm:p-5"
      >
        {#each albums as album (album.id)}
          <li class="min-w-0"><AlbumCard {album} /></li>
        {/each}
      </ul>
    {/if}
  </LibraryLoad>
</PageFrame>
