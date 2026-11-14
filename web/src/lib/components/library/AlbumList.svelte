<script lang="ts">
  import { page } from '$app/state';
  import { library } from '#lib/stores/library.svelte.ts';
  import ListFilter from '#lib/components/library/ListFilter.svelte';
  import AlbumItem from './AlbumItem.svelte';

  const activeId = $derived(
    page.url.pathname.startsWith('/albums/') ? (page.params.id ?? null) : null
  );
</script>

{#if library.albums.length === 0}
  <div class="px-5 py-3 text-xs text-muted" role="status">No albums</div>
{:else}
  <ListFilter
    items={library.albums}
    nameOf={(a) => a.albumName}
    noun={{ one: 'album', many: 'albums' }}
    class="ps-4 pe-1 touch:px-4"
  >
    {#snippet children(visible, query)}
      {#if visible.length > 0}
        <div class="flex flex-col gap-0.5 p-1">
          {#each visible as a (a.id)}
            <AlbumItem album={a} active={a.id === activeId} {query} />
          {/each}
        </div>
      {/if}
    {/snippet}
  </ListFilter>
{/if}
