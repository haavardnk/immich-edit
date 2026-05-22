<script lang="ts">
  import { page } from '$app/state';
  import { onMount, untrack } from 'svelte';
  import { searchMetadata, searchStatistics } from '$lib/api/search';
  import { editor } from '$lib/stores/editor.svelte';
  import { browsing } from '$lib/stores/browsing.svelte';
  import { browseControls } from '$lib/stores/browseControls.svelte';
  import AssetGrid from '$lib/components/browse/AssetGrid.svelte';
  import BrowseHeader from '$lib/components/browse/BrowseHeader.svelte';
  import type { AssetSummary } from '$lib/types/album';

  const id = $derived(page.params.id as string);
  let assets = $state<AssetSummary[]>([]);
  let loading = $state(false);
  let loadedOnce = $state(false);
  let loadingMore = $state(false);
  let nextPage = $state<string | null>(null);
  let totalCount = $state<number | undefined>(undefined);
  let prevKey = $state('');

  function fetchPage(tagId: string, initial: boolean): void {
    if (initial) {
      if (!loadedOnce) loading = true;
      nextPage = null;
      totalCount = undefined;
      searchStatistics(browseControls.statsBody({ tagIds: [tagId] }))
        .then((s) => (totalCount = s.total))
        .catch(() => {});
    }
    const body = browseControls.searchBody({ tagIds: [tagId] });
    if (!initial && nextPage) body.page = nextPage;
    searchMetadata(body)
      .then((result) => {
        assets = initial ? result.items : [...assets, ...result.items];
        browsing.set(assets);
        nextPage = result.nextPage;
      })
      .catch(() => {})
      .finally(() => {
        loading = false;
        loadedOnce = true;
        loadingMore = false;
      });
  }

  $effect(() => {
    const current = id;
    untrack(() => {
      browseControls.reset();
      prevKey = '';
      assets = [];
      loadedOnce = false;
      fetchPage(current, true);
    });
  });

  $effect(() => {
    const key = browseControls.serverFilterKey;
    if (prevKey && key !== prevKey) {
      fetchPage(id, true);
    }
    prevKey = key;
  });

  function loadMore(): void {
    if (loadingMore || !nextPage) return;
    loadingMore = true;
    fetchPage(id, false);
  }

  onMount(() => {
    editor.unload();
  });
</script>

{#if loading && !loadedOnce}
  <div class="flex-1 flex items-center justify-center text-sm text-immich-dark-fg/40">loading…</div>
{:else}
  <BrowseHeader title="Tag" loaded={assets.length} {totalCount} />
  <div class="flex-1 min-h-0 overflow-y-auto scrollbar-hidden">
    <AssetGrid {assets} {loadingMore} onLoadMore={nextPage ? loadMore : undefined} />
  </div>
{/if}
