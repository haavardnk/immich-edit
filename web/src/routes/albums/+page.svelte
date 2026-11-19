<script lang="ts">
  import { onMount } from 'svelte';
  import AlbumCard from '#lib/components/library/AlbumCard.svelte';
  import AlbumSortMenu from '#lib/components/library/AlbumSortMenu.svelte';
  import ChoiceChips from '#lib/components/ChoiceChips.svelte';
  import CreateAlbum from '#lib/components/library/CreateAlbum.svelte';
  import LibraryLoad from '#lib/components/library/LibraryLoad.svelte';
  import ListFilter from '#lib/components/library/ListFilter.svelte';
  import PageFrame from '#lib/components/shell/PageFrame.svelte';
  import { ALBUM_SCOPES, albumsInScope, type AlbumScope } from '#lib/browse/albumScope.ts';
  import { sortAlbums, type AlbumSort } from '#lib/browse/albumSort.ts';
  import { library } from '#lib/stores/library.svelte.ts';
  import { session } from '#lib/stores/session.svelte.ts';
  import { countOf } from '#lib/utils/plural.ts';

  let sort = $state<AlbumSort>('updated');
  let scope = $state<AlbumScope>('all');

  const loaded = $derived(library.isLoaded('albums'));
  const albums = $derived(
    sortAlbums(albumsInScope(library.albums, scope, session.user?.id ?? null), sort)
  );

  onMount(() => {
    library.request('albums');
  });
</script>

{#snippet actions()}
  <AlbumSortMenu value={sort} onChange={(next) => (sort = next)} />
  <CreateAlbum />
{/snippet}

{#snippet toolbar()}
  {#if loaded && library.albums.length > 0}
    <div class="flex shrink-0 px-3 py-1.5 sm:px-5 touch:px-3 touch:py-0">
      <ChoiceChips
        options={ALBUM_SCOPES}
        value={scope}
        label="Show albums"
        onSelect={(next) => (scope = next)}
      />
    </div>
  {/if}
{/snippet}

<PageFrame
  title="Albums"
  count={loaded ? countOf(albums.length, 'album') : undefined}
  {actions}
  {toolbar}
>
  <LibraryLoad view="albums" label="albums">
    {#if albums.length === 0}
      <p class="px-6 py-16 text-center text-sm text-dark/65" role="status">No albums</p>
    {:else}
      <ListFilter
        items={albums}
        nameOf={(a) => a.albumName}
        noun={{ one: 'album', many: 'albums' }}
        class="px-4 sm:px-5"
      >
        {#snippet children(visible, query)}
          {#if visible.length > 0}
            <ul
              aria-label="Albums"
              class="grid grid-cols-2 gap-x-3 gap-y-4.5 px-4 pt-1.5 pb-4 sm:grid-cols-[repeat(auto-fill,minmax(10rem,1fr))] sm:gap-x-4 sm:p-5"
            >
              {#each visible as album (album.id)}
                <li class="min-w-0"><AlbumCard {album} {query} /></li>
              {/each}
            </ul>
          {/if}
        {/snippet}
      </ListFilter>
    {/if}
  </LibraryLoad>
</PageFrame>
