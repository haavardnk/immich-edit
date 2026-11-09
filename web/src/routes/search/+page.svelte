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
  import SearchModeBar from '#lib/components/browse/SearchModeBar.svelte';
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
      if (query) void feed.open(null);
    });
    return () => feed.close();
  });

  $effect(() => feed.watchFilterChange());

  const view = $derived(
    query
      ? {
          title: `Search: ${query}`,
          toolbar: modeBar,
          emptyTitle: 'No results',
          emptyMessage: `Nothing matched “${query}”.`,
          emptyAction: mode === 'context' ? filenameFallback : undefined
        }
      : {
          title: 'Search',
          toolbar: undefined,
          emptyTitle: 'Search your library',
          emptyMessage: 'Type in the search bar to find photos.',
          emptyAction: undefined
        }
  );
</script>

{#snippet modeBar()}
  <SearchModeBar {modes} {mode} onSelect={setMode} />
{/snippet}

{#snippet recents()}
  <div class="pt-1">
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
  {feed}
  {...view}
  bar={searchBar}
  lead={query ? undefined : recents}
  loadingLabel="Searching…"
  headerWhileLoading
  sortBasis={null}
  trashRejects={false}
/>
