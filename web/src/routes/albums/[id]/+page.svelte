<script lang="ts">
  import { page } from '$app/state';
  import { onMount, untrack } from 'svelte';
  import { album } from '$lib/stores/album.svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import { browsing } from '$lib/stores/browsing.svelte';
  import AssetGrid from '$lib/components/browse/AssetGrid.svelte';

  const id = $derived(page.params.id as string);

  $effect(() => {
    const current = id;
    untrack(() => album.load(current));
  });

  $effect(() => {
    if (album.current) browsing.set(album.current.assets);
  });

  onMount(() => {
    editor.unload();
  });
</script>

{#if album.loading && !album.current}
  <div class="flex-1 flex items-center justify-center text-sm text-immich-dark-fg/40">loading album…</div>
{:else if album.error}
  <div class="flex-1 flex items-center justify-center text-sm text-red-400">{album.error}</div>
{:else if album.current}
  <div class="px-4 py-2.5 text-xs text-immich-dark-fg/40 border-b border-white/5 flex items-center gap-2">
    <span class="font-semibold text-immich-dark-fg/70 text-sm">{album.current.albumName}</span>
    <span class="text-immich-dark-fg/20">·</span>
    <span>{album.current.assetCount} assets</span>
  </div>
  <div class="flex-1 min-h-0 overflow-y-auto scrollbar-hidden">
    <AssetGrid assets={album.current.assets} />
  </div>
{/if}
