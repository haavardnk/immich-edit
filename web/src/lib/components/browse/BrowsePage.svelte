<script lang="ts">
  import AssetGrid from '#lib/components/browse/AssetGrid.svelte';
  import BrowseHeader from '#lib/components/browse/BrowseHeader.svelte';
  import BulkBar from '#lib/components/browse/bulk/BulkBar.svelte';
  import FilterPanel from '#lib/components/browse/filters/FilterPanel.svelte';
  import Notice from '#lib/components/Notice.svelte';
  import DateScrubber from '#lib/components/browse/scrubber/DateScrubber.svelte';
  import type { BrowseFeed } from '#lib/stores/browseFeed.svelte.ts';
  import { browseControls } from '#lib/stores/browseControls.svelte.ts';
  import { monthCounts } from '#lib/stores/monthCounts.svelte.ts';
  import { hides } from '#lib/browse/loadedFilter.ts';
  import { trash } from '#lib/stores/trash.svelte.ts';
  import { toasts } from '#lib/stores/toasts.svelte.ts';
  import type { AssetSummary } from '#lib/types/album.ts';
  import { Button, LoadingSpinner } from '@immich/ui';
  import { onMount, type Snippet } from 'svelte';

  let {
    title,
    assets: listed = [],
    loading: pageLoading = false,
    feed,
    loadingLabel,
    emptyTitle,
    emptyMessage,
    totalCount,
    countLoaded = false,
    favoriteLocked = false,
    sortBasis = 'capture',
    error = null,
    onRetry,
    headerWhileLoading = false,
    trashRejects = true,
    toolbar,
    emptyAction,
    bar,
    lead
  }: {
    title: string;
    assets?: AssetSummary[];
    loading?: boolean;
    feed?: BrowseFeed;
    loadingLabel: string;
    emptyTitle: string;
    emptyMessage?: string;
    totalCount?: number;
    countLoaded?: boolean;
    favoriteLocked?: boolean;
    sortBasis?: 'capture' | 'edit' | null;
    error?: string | null;
    onRetry?: () => void;
    headerWhileLoading?: boolean;
    trashRejects?: boolean;
    toolbar?: Snippet;
    emptyAction?: Snippet;
    bar?: Snippet;
    lead?: Snippet;
  } = $props();

  const assets = $derived(feed ? feed.assets : listed);
  const loading = $derived(pageLoading || (feed ? feed.loading && !feed.loadedOnce : false));
  const total = $derived(totalCount ?? feed?.totalCount);
  const hiddenCount = $derived(
    assets.filter((asset) => hides(browseControls.filters, asset)).length
  );
  const headerTotal = $derived(countLoaded ? assets.length - hiddenCount : total);
  const dayHeaders = $derived(sortBasis === 'capture' && browseControls.sortField === 'taken');
  const timeline = $derived(dayHeaders && feed?.metadataBacked ? feed : null);

  let scroller: HTMLDivElement | undefined = $state();

  onMount(() => monthCounts.invalidate());

  async function trashRejected(): Promise<void> {
    if (feed && !(await feed.loadAll())) return;
    const rejects = assets.filter((asset) => !hides(browseControls.filters, asset));
    if (rejects.length === 0) {
      toasts.push('info', 'No rejected photos here');
      return;
    }
    void trash.request(rejects);
  }
</script>

{#snippet loadingLine()}
  <div class="flex flex-1 items-center justify-center">
    <div class="inline-flex items-center gap-2 text-muted" role="status">
      <LoadingSpinner size="small" />
      <span class="text-xs">{loadingLabel}</span>
    </div>
  </div>
{/snippet}

{#snippet emptyState()}
  <div class="flex flex-1 flex-col items-center justify-center px-6 py-16 text-center text-muted">
    <h2 class="text-base font-medium text-light-950">{emptyTitle}</h2>
    {#if emptyMessage}
      <p class="mt-1 max-w-sm text-sm">{emptyMessage}</p>
    {/if}
    {@render emptyAction?.()}
  </div>
{/snippet}

<div class="flex min-h-0 flex-1">
  <div class="flex min-w-0 flex-1 flex-col">
    {#if error}
      <div class="flex-1 flex items-center justify-center">
        <Notice message={error} class="text-sm">
          {#if onRetry}
            <Button size="tiny" variant="outline" color="danger" class="ms-1" onclick={onRetry}>
              Retry
            </Button>
          {/if}
        </Notice>
      </div>
    {:else if loading && !headerWhileLoading}
      {@render loadingLine()}
    {:else}
      <BrowseHeader
        {title}
        totalCount={headerTotal}
        hiddenCount={countLoaded ? 0 : hiddenCount}
        {sortBasis}
        {bar}
        onTrashRejected={trashRejects && assets.length > hiddenCount ? trashRejected : undefined}
      />
      {@render toolbar?.()}
      {#if loading}
        {@render loadingLine()}
      {:else if assets.length === 0 && lead}
        <div
          class="flex min-h-0 flex-1 flex-col overflow-y-auto scrollbar-hidden"
          data-testid="browse-scroller"
        >
          {@render lead()}
          {@render emptyState()}
        </div>
      {:else if assets.length === 0}
        {@render emptyState()}
      {:else}
        <div class="relative flex min-h-0 flex-1">
          <div class="relative flex min-w-0 flex-1" data-testid="grid-column">
            <div
              bind:this={scroller}
              class="min-h-0 min-w-0 flex-1 overflow-y-auto scrollbar-hidden"
              data-testid="browse-scroller"
            >
              {@render lead?.()}
              <AssetGrid
                {assets}
                {dayHeaders}
                total={headerTotal}
                loadingMore={feed?.loadingMore}
                loadingPrevious={feed?.loadingPrevious}
                onLoadMore={feed?.hasMore ? () => void feed.loadMore() : undefined}
                onLoadPrevious={feed?.hasPrevious ? () => void feed.loadPrevious() : undefined}
                onLoadAll={feed ? (signal) => feed.loadAll(signal) : undefined}
              />
            </div>
            <BulkBar placement="float" />
          </div>
          {#if timeline}
            <DateScrubber feed={timeline} {scroller} />
          {/if}
        </div>
        <BulkBar placement="dock" />
      {/if}
    {/if}
  </div>
  <FilterPanel {favoriteLocked} count={headerTotal} {sortBasis} />
</div>
