import { searchWindow } from '$lib/api/search';
import { loadEditedAssets, loadFolderAssets, runSearch } from '$lib/browseSources';
import { validReturnPath } from '$lib/editorNavigation';
import { resolveSearchMode } from '$lib/searchMode';
import { sortAssets } from '$lib/sortAssets';
import { browseControls, type SortFamily } from '$lib/stores/browseControls.svelte';
import { recallBrowseFilters } from '$lib/stores/browseContext';
import { BrowseFeed, type BrowseFeedOptions } from '$lib/stores/browseFeed.svelte';
import { browsing } from '$lib/stores/browsing.svelte';
import { loadCullTags, stampCullTags } from '$lib/stores/cullTags';
import type { AssetSummary } from '$lib/types/album';

const WINDOW_RADIUS = 250;

interface RouteBase {
  context: string;
  family: SortFamily | null;
}

export type BrowseRoute =
  | (RouteBase & { feed: BrowseFeedOptions })
  | (RouteBase & {
      load: () => Promise<AssetSummary[]>;
      sortAt: (asset: AssetSummary) => string | null;
    });

function segment(pathname: string, prefix: string): string | null {
  if (!pathname.startsWith(prefix)) return null;
  const rest = pathname.slice(prefix.length);
  if (!rest || rest.includes('/')) return null;
  return decodeURIComponent(rest);
}

export function matchBrowseRoute(url: URL): BrowseRoute | null {
  const pathname = url.pathname;

  if (pathname === '/photos') {
    return { context: 'photos', family: 'timeline', feed: { baseBody: () => ({}) } };
  }
  if (pathname === '/favorites') {
    return {
      context: 'favorites',
      family: 'timeline',
      feed: { baseBody: () => ({ isFavorite: true }) }
    };
  }

  const albumId = segment(pathname, '/albums/');
  if (albumId) {
    return {
      context: 'album:' + albumId,
      family: 'collection',
      feed: { baseBody: () => ({ albumIds: [albumId] }) }
    };
  }

  const personId = segment(pathname, '/people/');
  if (personId) {
    return {
      context: 'person:' + personId,
      family: 'collection',
      feed: { baseBody: () => ({ personIds: [personId] }) }
    };
  }

  const tagId = segment(pathname, '/tags/');
  if (tagId) {
    return {
      context: 'tag:' + tagId,
      family: 'collection',
      feed: { baseBody: () => ({ tagIds: [tagId] }) }
    };
  }

  if (pathname === '/search') {
    const query = (url.searchParams.get('q') ?? '').trim();
    if (!query) return null;
    const mode = resolveSearchMode(query, url.searchParams.get('mode'));
    return {
      context: 'search:' + query,
      family: null,
      feed: {
        baseBody: () => (mode === 'filename' ? {} : { query }),
        fetcher: (body) => runSearch(query, mode, body),
        buildBody: (base) => browseControls.smartSearchBody(base),
        includeStats: false
      }
    };
  }

  if (pathname === '/folders') {
    const folder = url.searchParams.get('path') ?? '';
    if (!folder) return null;
    return {
      context: 'folders',
      family: 'collection',
      load: () => loadFolderAssets(folder),
      sortAt: (a) => a.fileCreatedAt
    };
  }

  if (pathname === '/edited') {
    return {
      context: 'edited',
      family: 'edited',
      load: loadEditedAssets,
      sortAt: (a) => a.updatedAt
    };
  }

  return null;
}

export async function restoreBrowse(from: string | null, assetId: string): Promise<void> {
  const returnPath = validReturnPath(from) ? from : null;
  const route = matchBrowseRoute(new URL(returnPath ?? '/photos', 'http://restore.invalid'));
  if (!route) return;

  browseControls.enter(route.context, route.family);
  const filters = returnPath ? recallBrowseFilters(returnPath) : null;
  if (filters) browseControls.applyFilters(filters);

  if ('feed' in route) {
    if (!route.feed.fetcher && (await restoreWindow(route.feed, assetId))) return;
    if (returnPath) await new BrowseFeed(route.feed).fetchPage(true);
    return;
  }
  await loadCullTags().catch(() => undefined);
  const assets = await route.load();
  browsing.set(sortAssets(assets, route.sortAt, browseControls.sortDir));
}

async function restoreWindow(feed: BrowseFeedOptions, assetId: string): Promise<boolean> {
  const query = browseControls.searchBody(feed.baseBody());
  const [result] = await Promise.all([
    searchWindow(assetId, query, WINDOW_RADIUS).catch(() => null),
    loadCullTags().catch(() => undefined)
  ]);
  if (!result?.items.length) return false;
  browsing.set(stampCullTags(result.items));
  return true;
}
