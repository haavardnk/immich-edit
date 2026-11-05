<script lang="ts">
  import AssetGrid from '#lib/components/browse/AssetGrid.svelte';
  import BrowseHeader from '#lib/components/browse/BrowseHeader.svelte';
  import FilterPanel from '#lib/components/browse/filters/FilterPanel.svelte';
  import Notice from '#lib/components/Notice.svelte';
  import { browseControls } from '#lib/stores/browseControls.svelte.ts';
  import { hides } from '#lib/browse/loadedFilter.ts';
  import { trash } from '#lib/stores/trash.svelte.ts';
  import { toasts } from '#lib/stores/toasts.svelte.ts';
  import type { AssetSummary } from '#lib/types/album.ts';
  import { Button, LoadingSpinner } from '@immich/ui';
  import type { Snippet } from 'svelte';

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
    onRetry,
    headerWhileLoading = false,
    trashRejects = true,
    toolbar,
    emptyAction,
    bar,
    lead
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

  const hiddenCount = $derived(
    assets.filter((asset) => hides(browseControls.filters, asset)).length
  );
  const headerTotal = $derived(countLoaded ? assets.length - hiddenCount : totalCount);

  async function trashRejected(): Promise<void> {
    if (onLoadAll && !(await onLoadAll())) return;
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
        <div class="flex-1 min-h-0 overflow-y-auto scrollbar-hidden" data-testid="browse-scroller">
          {@render lead?.()}
          <AssetGrid {assets} {loadingMore} {onLoadMore} {onLoadAll} />
        </div>
      {/if}
    {/if}
  </div>
  <FilterPanel {favoriteLocked} />
</div>
