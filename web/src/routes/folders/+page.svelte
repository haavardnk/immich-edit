<script lang="ts">
  import { page } from '$app/state';
  import { onMount, untrack } from 'svelte';
  import { folderAssets } from '$lib/api/folders';
  import { editor } from '$lib/stores/editor.svelte';
  import { browsing } from '$lib/stores/browsing.svelte';
  import { browseControls } from '$lib/stores/browseControls.svelte';
  import AssetGrid from '$lib/components/browse/AssetGrid.svelte';
  import BrowseHeader from '$lib/components/browse/BrowseHeader.svelte';
  import Spinner from '$lib/components/Spinner.svelte';
  import EmptyState from '$lib/components/EmptyState.svelte';
  import type { AssetSummary } from '$lib/types/album';

  let assets = $state<AssetSummary[]>([]);
  let loading = $state(false);
  let folderPath = $state('');

  const queryPath = $derived(page.url.searchParams.get('path') ?? '');

  async function loadFolder(path: string): Promise<void> {
    if (!path) return;
    folderPath = path;
    loading = true;
    const raw = await folderAssets(path);
    assets = raw.map((a) => ({
      id: a.id,
      originalFileName: a.originalFileName,
      type: a.type,
      fileCreatedAt: a.fileCreatedAt,
      updatedAt: a.updatedAt,
      checksum: a.checksum,
      isFavorite: a.isFavorite ?? false,
      exifInfo: a.exifInfo ?? null,
    }));
    browsing.set(assets);
    loading = false;
  }

  $effect(() => {
    const p = queryPath;
    untrack(() => loadFolder(p));
  });

  onMount(() => {
    editor.unload();
    browseControls.reset();
  });
</script>

{#if loading}
  <div class="flex-1 flex items-center justify-center"><Spinner label="Loading folder…" /></div>
{:else if !folderPath}
  <EmptyState title="Select a folder" message="Pick a folder from the sidebar to browse its photos." />
{:else if assets.length === 0}
  <EmptyState title="No photos in this folder" />
{:else}
  <BrowseHeader title={folderPath} loaded={assets.length} />
  <div class="flex-1 min-h-0 overflow-y-auto scrollbar-hidden">
    <AssetGrid {assets} />
  </div>
{/if}
