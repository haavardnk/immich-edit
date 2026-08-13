<script lang="ts">
  import AssetGrid from '$lib/components/browse/AssetGrid.svelte';
  import BrowseHeader from '$lib/components/browse/BrowseHeader.svelte';
  import EmptyState from '$lib/components/EmptyState.svelte';
  import Spinner from '$lib/components/Spinner.svelte';
  import type { AssetSummary } from '$lib/types/album';

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
    error = null
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
    error?: string | null;
  } = $props();
</script>

{#if loading}
  <div class="flex-1 flex items-center justify-center"><Spinner label={loadingLabel} /></div>
{:else if error}
  <div class="flex-1 flex items-center justify-center text-sm text-red-400">{error}</div>
{:else}
  <BrowseHeader {title} loaded={assets.length} {totalCount} {favoriteLocked} />
  {#if assets.length === 0}
    <EmptyState title={emptyTitle} message={emptyMessage} />
  {:else}
    <div class="flex-1 min-h-0 overflow-y-auto scrollbar-hidden">
      <AssetGrid {assets} {loadingMore} {onLoadMore} />
    </div>
  {/if}
{/if}
