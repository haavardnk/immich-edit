<script lang="ts">
  import AssetGrid from '$lib/components/browse/AssetGrid.svelte';
  import BrowseHeader from '$lib/components/browse/BrowseHeader.svelte';
  import Notice from '$lib/components/Notice.svelte';
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
    favoriteLocked = false,
    loadingMore = false,
    onLoadMore,
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
    favoriteLocked?: boolean;
    loadingMore?: boolean;
    onLoadMore?: () => void;
    sortBasis?: 'capture' | 'edit';
    error?: string | null;
    onRetry?: () => void;
  } = $props();
</script>

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
  <BrowseHeader {title} loaded={assets.length} {totalCount} {favoriteLocked} {sortBasis} />
  {#if assets.length === 0}
    <div class="flex flex-1 flex-col items-center justify-center px-6 py-16 text-center text-muted">
      <h2 class="text-base font-medium text-white">{emptyTitle}</h2>
      {#if emptyMessage}
        <p class="mt-1 max-w-sm text-sm">{emptyMessage}</p>
      {/if}
    </div>
  {:else}
    <div class="flex-1 min-h-0 overflow-y-auto scrollbar-hidden">
      <AssetGrid {assets} {loadingMore} {onLoadMore} />
    </div>
  {/if}
{/if}
