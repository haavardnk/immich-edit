<script lang="ts">
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import { untrack } from 'svelte';
  import { browseControls } from '$lib/stores/browseControls.svelte';
  import { readFilterParams } from '$lib/browse/filterParams';
  import { BrowseFeed } from '$lib/stores/browseFeed.svelte';
  import { selection } from '$lib/stores/selection.svelte';
  import { runSearch } from '$lib/browse/sources';
  import { resolveSearchMode, type SearchMode } from '$lib/browse/searchMode';
  import AssetGrid from '$lib/components/browse/AssetGrid.svelte';
  import BrowseHeader from '$lib/components/browse/BrowseHeader.svelte';
  import FilterPanel from '$lib/components/browse/filters/FilterPanel.svelte';
  import { Button, LoadingSpinner } from '@immich/ui';

  const query = $derived((page.url.searchParams.get('q') ?? '').trim());
  const mode = $derived(resolveSearchMode(query, page.url.searchParams.get('mode')));

  const feed = new BrowseFeed({
    baseFilter: () => ({}),
    flatOnly: () => mode === 'description',
    fetcher: (filter, cursor) => runSearch(query, mode, filter, cursor),
    includeStats: false
  });

  function setMode(next: SearchMode): void {
    if (next === mode) return;
    const url = new URL(page.url);
    url.searchParams.set('mode', next);
    void goto(`${url.pathname}${url.search}`, { replaceState: true, keepFocus: true });
  }

  $effect(() => {
    const _ = query;
    const _mode = mode;
    untrack(() => {
      selection.clear();
      browseControls.enter(null, readFilterParams(page.url.searchParams));
      feed.reset();
      if (query) void feed.fetchPage(true);
    });
  });

  $effect(() => feed.watchFilterChange());
</script>

<div class="flex min-h-0 flex-1">
  <div class="flex min-w-0 flex-1 flex-col">
    {#if !query}
      <BrowseHeader title="Search" hideSort />
      <div class="flex flex-col items-center justify-center px-6 py-16 text-center text-dark/65">
        <h3 class="text-sm font-medium text-dark/70">Search your library</h3>
        <p class="mt-1 max-w-xs text-xs">Type a description in the search bar to find photos.</p>
      </div>
    {:else if feed.loading && !feed.loadedOnce}
      <div class="flex flex-1 items-center justify-center">
        <div class="inline-flex items-center gap-2 text-dark/65" aria-live="polite">
          <LoadingSpinner size="small" />
          <span class="text-xs">Searching…</span>
        </div>
      </div>
    {:else}
      <BrowseHeader title={`Search: ${query}`} hideSort />
      <div class="flex flex-none items-center gap-2 border-b border-dark/10 px-3 py-1.5">
        <span class="text-[11px] text-dark/65">Search by</span>
        <div class="inline-flex rounded-full bg-light-100 p-0.5 text-[11px]">
          <Button
            type="button"
            size="tiny"
            variant={mode === 'description' ? 'filled' : 'ghost'}
            color={mode === 'description' ? 'primary' : 'secondary'}
            aria-pressed={mode === 'description'}
            onclick={() => setMode('description')}
          >
            Description
          </Button>
          <Button
            type="button"
            size="tiny"
            variant={mode === 'filename' ? 'filled' : 'ghost'}
            color={mode === 'filename' ? 'primary' : 'secondary'}
            aria-pressed={mode === 'filename'}
            onclick={() => setMode('filename')}
          >
            Filename
          </Button>
        </div>
      </div>
      {#if feed.assets.length === 0}
        <div class="flex flex-1 flex-col items-center justify-center">
          <div
            class="flex flex-col items-center justify-center px-6 py-16 text-center text-dark/65"
          >
            <h3 class="text-sm font-medium text-dark/70">No results</h3>
            <p class="mt-1 max-w-xs text-xs">Nothing matched “{query}”.</p>
          </div>
          {#if mode === 'description'}
            <Button
              type="button"
              size="tiny"
              variant="ghost"
              color="secondary"
              class="mt-3"
              onclick={() => setMode('filename')}
            >
              Search filenames instead
            </Button>
          {/if}
        </div>
      {:else}
        <div class="flex-1 min-h-0 overflow-y-auto scrollbar-hidden">
          <AssetGrid
            assets={feed.assets}
            loadingMore={feed.loadingMore}
            onLoadMore={feed.hasMore ? () => feed.loadMore() : undefined}
            onLoadAll={() => feed.loadAll()}
          />
        </div>
      {/if}
    {/if}
  </div>
  <FilterPanel hideFilenameFilter />
</div>
