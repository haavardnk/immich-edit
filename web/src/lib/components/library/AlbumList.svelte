<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/state';
  import { library } from '$lib/stores/library.svelte';
  import AlbumItem from './AlbumItem.svelte';

  onMount(() => {
    void library.load();
  });

  const activeId = $derived(page.params.id ?? null);
</script>

{#if library.loading && library.albums.length === 0}
  <div class="p-3 text-xs opacity-50">loading…</div>
{:else if library.error}
  <div class="p-3 text-xs text-error">{library.error}</div>
{:else if library.albums.length === 0}
  <div class="p-3 text-xs opacity-50">no albums</div>
{:else}
  <ul class="menu menu-sm w-full p-1 gap-0.5">
    {#each library.albums as a (a.id)}
      <AlbumItem album={a} active={a.id === activeId} />
    {/each}
  </ul>
{/if}
