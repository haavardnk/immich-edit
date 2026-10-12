<script lang="ts">
  import AssetGrid from '$lib/components/browse/AssetGrid.svelte';
  import BrowseHeader from '$lib/components/browse/BrowseHeader.svelte';
  import FilterPanel from '$lib/components/browse/filters/FilterPanel.svelte';
  import Notice from '$lib/components/Notice.svelte';
  import { browseControls } from '$lib/stores/browseControls.svelte';
  import { hides } from '$lib/browse/loadedFilter';
  import type { AssetSummary } from '$lib/types/album';
  import { Button, LoadingSpinner } from '@immich/ui';

  let {
    title,
    assets,
    loading,
    loadingLabel,
    emptyTitle,
    emptyMessage,
    totalCount,
    countLoaded = false,
    favoriteLocked = false,
    loadingMore = false,
    onLoadMore,
    onLoadAll,
    sortBasis = 'capture',
    error = null,
    onRetry
  }: {
    title: string;
    assets: AssetSummary[];
    loading: boolean;
    loadingLabel: string;
    emptyTitle: string;
    emptyMessage?: string;
    totalCount?: number;
    countLoaded?: boolean;
    favoriteLocked?: boolean;
    loadingMore?: boolean;
    onLoadMore?: () => void;
    onLoadAll?: () => Promise<boolean>;
    sortBasis?: 'capture' | 'edit';
    error?: string | null;
    onRetry?: () => void;
  } = $props();

  const hiddenCount = $derived(
    assets.filter((asset) => hides(browseControls.filters, asset)).length
  );
  const headerTotal = $derived(countLoaded ? assets.length - hiddenCount : totalCount);
</script>

<div class="flex min-h-0 flex-1">
  <div class="flex min-w-0 flex-1 flex-col">
    {#if loading}
      <div class="flex flex-1 items-center justify-center">
        <div class="inline-flex items-center gap-2 text-muted" role="status">
          <LoadingSpinner size="small" />
          <span class="text-xs">{loadingLabel}</span>
        </div>
      </div>
    {:else if error}
      <div class="flex-1 flex items-center justify-center">
        <Notice message={error} class="text-sm">
          {#if onRetry}
            <Button size="tiny" variant="outline" color="danger" class="ms-1" onclick={onRetry}>
              Retry
            </Button>
          {/if}
        </Notice>
      </div>
    {:else}
      <BrowseHeader
        {title}
        totalCount={headerTotal}
        hiddenCount={countLoaded ? 0 : hiddenCount}
        {sortBasis}
      />
      {#if assets.length === 0}
        <div
          class="flex flex-1 flex-col items-center justify-center px-6 py-16 text-center text-muted"
        >
          <h2 class="text-base font-medium text-white">{emptyTitle}</h2>
          {#if emptyMessage}
            <p class="mt-1 max-w-sm text-sm">{emptyMessage}</p>
          {/if}
        </div>
      {:else}
        <div class="flex-1 min-h-0 overflow-y-auto scrollbar-hidden">
          <AssetGrid {assets} {loadingMore} {onLoadMore} {onLoadAll} />
        </div>
      {/if}
    {/if}
  </div>
  <FilterPanel {favoriteLocked} />
</div>
