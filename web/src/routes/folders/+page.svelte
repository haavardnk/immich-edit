<script lang="ts">
  import { page } from '$app/state';
  import { onMount, untrack } from 'svelte';
  import { loadFolderAssets } from '#lib/browse/sources.ts';
  import { readFilterParams } from '#lib/browse/filterParams.ts';
  import { folderName, subfolders } from '#lib/browse/folders.ts';
  import { browsing } from '#lib/stores/browsing.svelte.ts';
  import { browseControls } from '#lib/stores/browseControls.svelte.ts';
  import { library } from '#lib/stores/library.svelte.ts';
  import { fieldKey } from '#lib/browse/sortAssets.ts';
  import { arrangeLoaded } from '#lib/browse/loadedFilter.ts';
  import { countOf } from '#lib/utils/plural.ts';
  import BrowsePage from '#lib/components/browse/BrowsePage.svelte';
  import FolderCrumbs from '#lib/components/library/FolderCrumbs.svelte';
  import FolderRows from '#lib/components/library/FolderRows.svelte';
  import type { AssetSummary } from '#lib/types/album.ts';

  let loaded = $state<AssetSummary[]>([]);
  let loading = $state(false);
  let folderPath = $state('');

  const queryPath = $derived(page.url.searchParams.get('path') ?? '');
  const assets = $derived(arrangeLoaded(loaded, fieldKey(browseControls.sortField)));
  const children = $derived(subfolders(library.folderTree, folderPath));

  async function loadFolder(path: string): Promise<void> {
    folderPath = path;
    if (!path) {
      loaded = [];
      loading = false;
      return;
    }
    loading = true;
    const next = await loadFolderAssets(path);
    if (folderPath !== path) return;
    loaded = next;
    loading = false;
  }

  onMount(() => {
    library.request('folders');
  });

  $effect(() => {
    const p = queryPath;
    untrack(() => {
      browseControls.enter('collection', readFilterParams(page.url));
      void loadFolder(p);
    });
  });

  $effect(() => {
    browsing.set(assets);
  });
</script>

{#snippet crumbs()}
  <FolderCrumbs path={folderPath} />
{/snippet}

{#snippet lead()}
  <FolderRows folders={children} />
  {#if children.length > 0 && assets.length > 0}
    <div
      class="flex items-baseline justify-between border-t border-hairline px-3 pt-3 pb-1 sm:px-5 touch:px-4"
    >
      <h2 class="text-xs font-semibold text-light-950 touch:text-[13px]">In this folder</h2>
      <span class="text-xs text-dark/65">{countOf(assets.length, 'photo')}</span>
    </div>
  {/if}
{/snippet}

<BrowsePage
  title={folderPath ? folderName(folderPath) : 'Folders'}
  {assets}
  {loading}
  loadingLabel="Loading folder…"
  emptyTitle={folderPath ? 'No photos in this folder' : 'Select a folder'}
  emptyMessage={folderPath ? undefined : 'Pick a folder to browse its photos.'}
  countLoaded={folderPath !== ''}
  headerWhileLoading
  toolbar={crumbs}
  {lead}
/>
