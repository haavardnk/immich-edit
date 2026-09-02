<script lang="ts">
  import { page } from '$app/state';
  import { library } from '$lib/stores/library.svelte';
  import AlbumItem from './AlbumItem.svelte';

  const activeId = $derived(
    page.url.pathname.startsWith('/albums/') ? (page.params.id ?? null) : null
  );
</script>

{#if library.albums.length === 0}
  <div class="px-5 py-3 text-xs text-muted" role="status">No albums</div>
{:else}
  <div class="flex flex-col gap-0.5 p-1">
    {#each library.albums as a (a.id)}
      <AlbumItem album={a} active={a.id === activeId} />
    {/each}
  </div>
{/if}
