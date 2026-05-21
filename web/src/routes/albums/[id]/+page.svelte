<script lang="ts">
  import { page } from '$app/state';
  import { onMount, untrack } from 'svelte';
  import { album } from '$lib/stores/album.svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import AssetGrid from '$lib/components/browse/AssetGrid.svelte';

  const id = $derived(page.params.id as string);

  $effect(() => {
    const current = id;
    untrack(() => album.load(current));
  });

  onMount(() => {
    editor.unload();
  });
</script>

{#if album.loading && !album.current}
  <div class="flex-1 flex items-center justify-center text-sm opacity-50">loading album…</div>
{:else if album.error}
  <div class="flex-1 flex items-center justify-center text-sm text-error">{album.error}</div>
{:else if album.current}
  <div class="px-3 py-2 text-xs opacity-50 border-b border-base-content/10 flex items-center gap-2">
    <span class="font-semibold opacity-90 text-sm">{album.current.albumName}</span>
    <span>·</span>
    <span>{album.current.assetCount} assets</span>
  </div>
  <div class="flex-1 min-h-0 overflow-y-auto">
    <AssetGrid assets={album.current.assets} />
  </div>
{/if}
