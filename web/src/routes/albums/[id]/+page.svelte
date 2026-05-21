<script lang="ts">
  import { page } from '$app/state';
  import Navbar from '$lib/components/layout/Navbar.svelte';
  import AssetTile from '$lib/components/browse/AssetTile.svelte';
  import { getAlbum } from '$lib/api/albums';
  import type { AlbumDetail } from '$lib/types/album';
  import { onMount } from 'svelte';

  let album = $state<AlbumDetail | null>(null);
  let error = $state<string | null>(null);
  let loading = $state(true);

  onMount(async () => {
    try {
      album = await getAlbum(page.params.id as string);
    } catch (e) {
      error = (e as Error).message;
    } finally {
      loading = false;
    }
  });
</script>

<div class="min-h-screen flex flex-col">
  <Navbar title={album?.albumName ?? '…'} back="/" />
  <main class="flex-1 p-4">
    {#if loading}
      <div class="opacity-60">loading album…</div>
    {:else if error}
      <div class="alert alert-error">{error}</div>
    {:else if album}
      <h1 class="text-xl font-bold mb-3">{album.albumName}</h1>
      <p class="text-sm opacity-60 mb-3">{album.assetCount} assets</p>
      <div class="grid grid-cols-3 sm:grid-cols-4 md:grid-cols-6 lg:grid-cols-8 gap-2">
        {#each album.assets as asset (asset.id)}
          <AssetTile {asset} />
        {/each}
      </div>
    {/if}
  </main>
</div>
