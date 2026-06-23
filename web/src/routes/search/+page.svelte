<script lang="ts">
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import { onMount, untrack } from 'svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import { browseControls } from '$lib/stores/browseControls.svelte';
  import { BrowseFeed } from '$lib/stores/browseFeed.svelte';
  import { selection } from '$lib/stores/selection.svelte';
  import { searchSmart, searchMetadata, type SearchResult } from '$lib/api/search';
  import { resolveSearchMode, type SearchMode } from '$lib/searchMode';
  import { toasts } from '$lib/stores/toasts.svelte';
  import AssetGrid from '$lib/components/browse/AssetGrid.svelte';
  import BrowseHeader from '$lib/components/browse/BrowseHeader.svelte';
  import Spinner from '$lib/components/Spinner.svelte';
  import EmptyState from '$lib/components/EmptyState.svelte';

  const query = $derived((page.url.searchParams.get('q') ?? '').trim());
  const mode = $derived(resolveSearchMode(query, page.url.searchParams.get('mode')));

  async function runSearch(body: Record<string, unknown>): Promise<SearchResult> {
    if (mode === 'filename') {
      const filenameBody: Record<string, unknown> = { ...body, originalFileName: query };
      delete filenameBody.query;
      return searchMetadata(filenameBody);
    }
    try {
      return await searchSmart(body);
    } catch {
      const fallback: Record<string, unknown> = { ...body, originalFileName: query };
      delete fallback.query;
      toasts.push('warn', 'Smart search unavailable, showing filename matches');
      return await searchMetadata(fallback);
    }
  }

  const feed = new BrowseFeed({
    baseBody: () => (mode === 'filename' ? {} : { query }),
    fetcher: runSearch,
    buildBody: (base) => browseControls.smartSearchBody(base),
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
  <div class="flex flex-none items-center gap-2 border-b border-white/10 px-3 py-1.5">
    <span class="text-[11px] text-immich-dark-fg/40">Search by</span>
    <div class="inline-flex rounded-full bg-white/5 p-0.5 text-[11px]">
      <button
        type="button"
        class="rounded-full px-2.5 py-0.5 transition-colors {mode === 'description'
          ? 'bg-white/15 text-immich-dark-fg'
          : 'text-immich-dark-fg/50 hover:text-immich-dark-fg'}"
        onclick={() => setMode('description')}
      >
        Description
      </button>
      <button
        type="button"
        class="rounded-full px-2.5 py-0.5 transition-colors {mode === 'filename'
          ? 'bg-white/15 text-immich-dark-fg'
          : 'text-immich-dark-fg/50 hover:text-immich-dark-fg'}"
        onclick={() => setMode('filename')}
      >
        Filename
      </button>
    </div>
  </div>
  {#if feed.assets.length === 0}
    <div class="flex flex-1 flex-col items-center justify-center">
      <EmptyState title="No results" message={`Nothing matched “${query}”.`} />
      {#if mode === 'description'}
        <button
          type="button"
          class="mt-3 text-xs text-immich-dark-primary hover:underline"
          onclick={() => setMode('filename')}
        >
          Search filenames instead
        </button>
      {/if}
    </div>
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
