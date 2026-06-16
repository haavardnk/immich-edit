<script lang="ts">
  import { page } from '$app/state';
  import { onMount, untrack } from 'svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import { browseControls } from '$lib/stores/browseControls.svelte';
  import { BrowseFeed } from '$lib/stores/browseFeed.svelte';
  import { selection } from '$lib/stores/selection.svelte';
  import { searchSmart, searchMetadata, type SearchResult } from '$lib/api/search';
  import { toasts } from '$lib/stores/toasts.svelte';
  import AssetGrid from '$lib/components/browse/AssetGrid.svelte';
  import BrowseHeader from '$lib/components/browse/BrowseHeader.svelte';
  import Spinner from '$lib/components/Spinner.svelte';
  import EmptyState from '$lib/components/EmptyState.svelte';

  const query = $derived((page.url.searchParams.get('q') ?? '').trim());

  async function smartWithFallback(body: Record<string, unknown>): Promise<SearchResult> {
    try {
      return await searchSmart(body);
    } catch {
      const q = typeof body.query === 'string' ? body.query : '';
      const fallback: Record<string, unknown> = { ...body, originalFileName: q };
      delete fallback.query;
      toasts.push('warn', 'Smart search unavailable, showing filename matches');
      return await searchMetadata(fallback);
    }
  }

  const feed = new BrowseFeed({
    baseBody: () => ({ query }),
    fetcher: smartWithFallback,
    buildBody: (base) => browseControls.smartSearchBody(base),
    includeStats: false
  });

  $effect(() => {
    const _ = query;
    untrack(() => {
      selection.clear();
      browseControls.enter('search:' + query);
      feed.reset();
      if (query) feed.fetchPage(true);
    });
  });

  $effect(() => feed.watchFilterChange());

  onMount(() => {
    editor.unload();
  });
</script>

{#if !query}
  <BrowseHeader title="Search" loaded={0} hideSort hideFilenameFilter />
  <EmptyState title="Search your library" message="Type a description in the search bar to find photos." />
{:else if feed.loading && !feed.loadedOnce}
  <div class="flex-1 flex items-center justify-center"><Spinner label="Searching…" /></div>
{:else}
  <BrowseHeader
    title={`Search: ${query}`}
    loaded={feed.assets.length}
    hideSort
    hideFilenameFilter
  />
  {#if feed.assets.length === 0}
    <EmptyState title="No results" message={`Nothing matched “${query}”.`} />
  {:else}
    <div class="flex-1 min-h-0 overflow-y-auto scrollbar-hidden">
      <AssetGrid
        assets={feed.assets}
        loadingMore={feed.loadingMore}
        onLoadMore={feed.nextPage ? () => feed.loadMore() : undefined}
      />
    </div>
  {/if}
{/if}
