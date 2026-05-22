<script lang="ts">
  import { onMount } from 'svelte';
  import { searchMetadata, searchStatistics } from '$lib/api/search';
  import { editor } from '$lib/stores/editor.svelte';
  import { browsing } from '$lib/stores/browsing.svelte';
  import { browseControls } from '$lib/stores/browseControls.svelte';
  import AssetGrid from '$lib/components/browse/AssetGrid.svelte';
  import BrowseHeader from '$lib/components/browse/BrowseHeader.svelte';
  import type { AssetSummary } from '$lib/types/album';

  let assets = $state<AssetSummary[]>([]);
  let loading = $state(true);
  let loadedOnce = $state(false);
  let loadingMore = $state(false);
  let nextPage = $state<string | null>(null);
  let totalCount = $state<number | undefined>(undefined);
  let prevKey = $state('');

  onMount(() => {
    editor.unload();
    browseControls.reset();
    fetchPage(true);
  });

  function fetchPage(initial: boolean): void {
    if (initial) {
      if (!loadedOnce) loading = true;
      nextPage = null;
      totalCount = undefined;
      searchStatistics(browseControls.statsBody({}))
        .then((s) => (totalCount = s.total))
        .catch(() => {});
    }
    const body = browseControls.searchBody({});
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
    const key = browseControls.serverFilterKey;
    if (prevKey && key !== prevKey) {
      fetchPage(true);
    }
    prevKey = key;
  });

  function loadMore(): void {
    if (loadingMore || !nextPage) return;
    loadingMore = true;
    fetchPage(false);
  }
</script>

{#if loading && !loadedOnce}
  <div class="flex-1 flex items-center justify-center text-sm text-immich-dark-fg/40">loading…</div>
{:else}
  <BrowseHeader title="Photos" loaded={assets.length} {totalCount} />
  {#if assets.length === 0}
    <div class="flex-1 flex items-center justify-center text-sm text-immich-dark-fg/40">no photos</div>
  {:else}
    <div class="flex-1 min-h-0 overflow-y-auto scrollbar-hidden">
      <AssetGrid {assets} {loadingMore} onLoadMore={nextPage ? loadMore : undefined} />
    </div>
  {/if}
{/if}
