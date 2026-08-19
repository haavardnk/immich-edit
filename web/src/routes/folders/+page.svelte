<script lang="ts">
  import { page } from '$app/state';
  import { onMount, untrack } from 'svelte';
  import { folderAssets } from '$lib/api/folders';
  import { editor } from '$lib/stores/editor.svelte';
  import { browsing } from '$lib/stores/browsing.svelte';
  import { browseControls } from '$lib/stores/browseControls.svelte';
  import { sortAssets } from '$lib/sortAssets';
  import BrowseShell from '$lib/components/browse/BrowseShell.svelte';
  import EmptyState from '$lib/components/EmptyState.svelte';
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
    editor.unload();
    browseControls.enter('folders', 'collection');
  });
</script>

{#if !loading && !folderPath}
  <EmptyState
    title="Select a folder"
    message="Pick a folder from the sidebar to browse its photos."
  />
{:else}
  <BrowseShell
    title={folderPath}
    {assets}
    {loading}
    loadingLabel="Loading folder…"
    emptyTitle="No photos in this folder"
  />
{/if}
