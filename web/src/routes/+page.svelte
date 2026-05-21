<script lang="ts">
  import Navbar from '$lib/components/layout/Navbar.svelte';
  import AlbumCard from '$lib/components/browse/AlbumCard.svelte';
  import { listAlbums } from '$lib/api/albums';
  import type { AlbumSummary } from '$lib/types/album';
  import { onMount } from 'svelte';

  let albums = $state<AlbumSummary[]>([]);
  let error = $state<string | null>(null);
  let loading = $state(true);

  onMount(async () => {
    try {
      albums = await listAlbums();
    } catch (e) {
      error = (e as Error).message;
    } finally {
      loading = false;
    }
  });
</script>

<div class="min-h-screen flex flex-col">
  <Navbar />
  <main class="flex-1 p-4">
    {#if loading}
      <div class="opacity-60">loading albums…</div>
    {:else if error}
      <div class="alert alert-error">{error}</div>
    {:else if albums.length === 0}
      <div class="opacity-60">no albums</div>
    {:else}
      <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-6 gap-3">
        {#each albums as album (album.id)}
          <AlbumCard {album} />
        {/each}
      </div>
    {/if}
  </main>
</div>
