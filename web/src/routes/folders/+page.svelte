<script lang="ts">
  import { page } from '$app/state';
  import { onMount, untrack } from 'svelte';
  import { folderAssets } from '$lib/api/folders';
  import { browsing } from '$lib/stores/browsing.svelte';
  import { browseControls } from '$lib/stores/browseControls.svelte';
  import { sortAssets } from '$lib/sortAssets';
  import BrowseShell from '$lib/components/browse/BrowseShell.svelte';
  import type { AssetSummary } from '$lib/types/album';

  let loaded = $state<AssetSummary[]>([]);
  let loading = $state(false);
  let folderPath = $state('');

  const queryPath = $derived(page.url.searchParams.get('path') ?? '');
  const assets = $derived(sortAssets(loaded, (a) => a.fileCreatedAt, browseControls.sortDir));

  async function loadFolder(path: string): Promise<void> {
    if (!path) return;
    folderPath = path;
    loading = true;
    const raw = await folderAssets(path);
    loaded = raw.map((a) => ({
      id: a.id,
      originalFileName: a.originalFileName,
      type: a.type,
      fileCreatedAt: a.fileCreatedAt,
      updatedAt: a.updatedAt,
      checksum: a.checksum,
      isFavorite: a.isFavorite ?? false,
      exifInfo: a.exifInfo ?? null,
      tags: []
    }));
    loading = false;
  }

  $effect(() => {
    const p = queryPath;
    untrack(() => loadFolder(p));
  });

  $effect(() => {
    browsing.set(assets);
  });

  onMount(() => {
    browseControls.enter('folders', 'collection');
  });
</script>

{#if !loading && !folderPath}
  <div class="flex flex-col items-center justify-center px-6 py-16 text-center text-dark/65">
    <h3 class="text-sm font-medium text-dark/70">Select a folder</h3>
    <p class="mt-1 max-w-xs text-xs">Pick a folder from the sidebar to browse its photos.</p>
  </div>
{:else}
  <BrowseShell
    title={folderPath}
    {assets}
    {loading}
    loadingLabel="Loading folder…"
    emptyTitle="No photos in this folder"
  />
{/if}
