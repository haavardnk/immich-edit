<script lang="ts">
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import { untrack } from 'svelte';
  import { browseControls } from '#lib/stores/browseControls.svelte.ts';
  import { readFilterParams } from '#lib/browse/filterParams.ts';
  import { BrowseFeed } from '#lib/stores/browseFeed.svelte.ts';
  import { selection } from '#lib/stores/selection.svelte.ts';
  import { searchFeed } from '#lib/browse/sources.ts';
  import {
    resolveSearchMode,
    SEARCH_MODES,
    searchModeAvailable,
    type SearchMode
  } from '#lib/browse/searchMode.ts';
  import { runSearch } from '#lib/browse/searchInput.svelte.ts';
  import BrowsePage from '#lib/components/browse/BrowsePage.svelte';
  import RecentSearches from '#lib/components/browse/RecentSearches.svelte';
  import TouchSearchBar from '#lib/components/shell/touch/TouchSearchBar.svelte';
  import { Button } from '@immich/ui';

  const query = $derived((page.url.searchParams.get('q') ?? '').trim());
  const mode = $derived(resolveSearchMode(query, page.url.searchParams.get('mode')));
  const modes = $derived(SEARCH_MODES.filter((option) => searchModeAvailable(option.mode)));

  const feed = new BrowseFeed(
    searchFeed(
      () => query,
      () => mode
    )
  );

  function setMode(next: SearchMode): void {
    if (next === mode) return;
    const url = new URL(page.url.href);
    url.searchParams.set('mode', next);
    void goto(`${url.pathname}${url.search}`, { replace: true, reset: false });
  }

  $effect(() => {
    const _ = query;
    const _mode = mode;
    untrack(() => {
      selection.clear();
      browseControls.enter(null, readFilterParams(page.url));
      feed.reset();
      if (query) void feed.fetchPage(true);
    });
  });

  $effect(() => feed.watchFilterChange());

  const view = $derived(
    query
      ? {
          title: `Search: ${query}`,
          loading: feed.loading && !feed.loadedOnce,
          toolbar: modeBar,
          emptyTitle: 'No results',
          emptyMessage: `Nothing matched “${query}”.`,
          emptyAction: mode === 'context' ? filenameFallback : undefined
        }
      : {
          title: 'Search',
          loading: false,
          toolbar: undefined,
          emptyTitle: 'Search your library',
          emptyMessage: 'Type in the search bar to find photos.',
          emptyAction: recents
        }
  );
</script>

{#snippet modeBar()}
  <div class="flex flex-none items-center gap-2 border-b border-dark/10 px-3 py-1.5">
    <span class="shrink-0 text-[11px] text-dark/65">Search by</span>
    <div class="flex min-w-0 flex-wrap gap-0.5 rounded-2xl bg-light-100 p-0.5 text-[11px]">
      {#each modes as option (option.mode)}
        <Button
          type="button"
          size="tiny"
          variant={mode === option.mode ? 'filled' : 'ghost'}
          color={mode === option.mode ? 'primary' : 'secondary'}
          aria-pressed={mode === option.mode}
          onclick={() => setMode(option.mode)}
        >
          {option.label}
        </Button>
      {/each}
    </div>
  </div>
{/snippet}

{#snippet recents()}
  <div class="mt-6 flex w-full justify-center">
    <RecentSearches onRun={runSearch} />
  </div>
{/snippet}

{#snippet searchBar()}
  <TouchSearchBar />
{/snippet}

{#snippet filenameFallback()}
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
{/snippet}

<BrowsePage
  {...view}
  bar={searchBar}
  assets={feed.assets}
  loadingLabel="Searching…"
  headerWhileLoading
  sortBasis={null}
  trashRejects={false}
  loadingMore={feed.loadingMore}
  onLoadMore={feed.hasMore ? () => feed.loadMore() : undefined}
  onLoadAll={() => feed.loadAll()}
/>
