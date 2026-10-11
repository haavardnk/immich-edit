import { searchWindow } from '$lib/api/search';
import { loadEditedAssets, loadFolderAssets, runSearch } from './sources';
import { validReturnPath } from '$lib/editor/navigation';
import { resolveSearchMode } from './searchMode';
import { fieldKey, sortAssets, timeOf, type SortKey } from './sortAssets';
import { browseControls, type SortFamily } from '$lib/stores/browseControls.svelte';
import { readFilterParams } from './filterParams';
import { BrowseFeed, type BrowseFeedOptions } from '$lib/stores/browseFeed.svelte';
import { browsing } from '$lib/stores/browsing.svelte';
import { loadCullTags, stampCullTags } from '$lib/stores/cullTags';
import type { AssetSummary } from '$lib/types/album';

const WINDOW_RADIUS = 250;

interface RouteBase {
  family: SortFamily | null;
}

export type BrowseRoute =
  | (RouteBase & { feed: BrowseFeedOptions })
  | (RouteBase & {
      load: () => Promise<AssetSummary[]>;
      sortKey: () => SortKey;
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
    return { family: 'timeline', feed: { baseFilter: () => ({}) } };
  }
  if (pathname === '/favorites') {
    return {
      family: 'timeline',
      feed: { baseFilter: () => ({ favorite: true }) }
    };
  }

  const albumId = segment(pathname, '/albums/');
  if (albumId) {
    return {
      family: 'collection',
      feed: { baseFilter: () => ({ scope: { album: albumId } }) }
    };
  }

  const personId = segment(pathname, '/people/');
  if (personId) {
    return {
      family: 'collection',
      feed: { baseFilter: () => ({ scope: { person: personId } }) }
    };
  }

  const tagId = segment(pathname, '/tags/');
  if (tagId) {
    return {
      family: 'collection',
      feed: { baseFilter: () => ({ scope: { tag: tagId } }) }
    };
  }

  if (pathname === '/search') {
    const query = (url.searchParams.get('q') ?? '').trim();
    if (!query) return null;
    const mode = resolveSearchMode(query, url.searchParams.get('mode'));
    return {
      family: null,
      feed: {
        baseFilter: () => ({}),
        flatOnly: () => mode === 'description',
        fetcher: (filter, cursor) => runSearch(query, mode, filter, cursor),
        includeStats: false
      }
    };
  }

  if (pathname === '/folders') {
    const folder = url.searchParams.get('path') ?? '';
    if (!folder) return null;
    return {
      family: 'collection',
      load: () => loadFolderAssets(folder),
      sortKey: () => fieldKey(browseControls.sortField)
    };
  }

  if (pathname === '/edited') {
    return {
      family: 'edited',
      load: loadEditedAssets,
      sortKey: () => (a) => timeOf(a.updatedAt)
    };
  }

  return null;
}

export async function restoreBrowse(from: string | null, assetId: string): Promise<void> {
  const returnPath = validReturnPath(from) ? from : null;
  const url = new URL(returnPath ?? '/photos', 'http://restore.invalid');
  const route = matchBrowseRoute(url);
  if (!route) return;

  browseControls.enter(route.family, readFilterParams(url.searchParams));

  if ('feed' in route) {
    if (!route.feed.fetcher && (await restoreWindow(route.feed, assetId))) return;
    if (returnPath) await new BrowseFeed(route.feed).fetchPage(true);
    return;
  }
  await loadCullTags().catch(() => undefined);
  const assets = await route.load();
  browsing.set(sortAssets(assets, route.sortKey(), browseControls.sortDir));
}

async function restoreWindow(feed: BrowseFeedOptions, assetId: string): Promise<boolean> {
  const filter = browseControls.filter(feed.baseFilter());
  const [result] = await Promise.all([
    searchWindow(assetId, filter, browseControls.sort, WINDOW_RADIUS).catch(() => null),
    loadCullTags().catch(() => undefined)
  ]);
  if (!result?.items.length) return false;
  browsing.set(stampCullTags(result.items));
  return true;
}
