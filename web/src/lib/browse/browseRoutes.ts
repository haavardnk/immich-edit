import { loadEditedAssets, loadFolderAssets, runSearch } from './sources';
import { resolveSearchMode } from './searchMode';
import { fieldKey, timeOf, type SortKey } from './sortAssets';
import { LOADED_FILTERS } from './loadedFilter';
import {
  browseControls,
  type BrowseFilters,
  type SortFamily
} from '$lib/stores/browseControls.svelte';
import type { BrowseFeedOptions } from '$lib/stores/browseFeed.svelte';
import { immichCapabilities } from '$lib/stores/immichCapabilities.svelte';
import type { AssetSummary } from '$lib/types/album';
import type { SearchScope } from '$lib/types/search';

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

const SCOPE_EXCLUSIONS: Partial<Record<keyof BrowseFilters, string>> = {
  notInAlbum: 'album',
  untagged: 'tag',
  noPeople: 'person'
};

const STRUCTURED_ONLY: (keyof BrowseFilters)[] = ['noPeople', 'minSize', 'maxSize'];

export function structuredRoute(url: URL): boolean {
  if (!immichCapabilities.structuredSearch) return false;
  const route = matchBrowseRoute(url);
  return !route || !('feed' in route) || !route.feed.flatOnly?.();
}

export function routeScope(url: URL): SearchScope | undefined {
  const route = matchBrowseRoute(url);
  return route && 'feed' in route ? route.feed.baseFilter().scope : undefined;
}

export function supportsFilter(url: URL, key: keyof BrowseFilters): boolean {
  const route = matchBrowseRoute(url);
  if (route && !('feed' in route)) return LOADED_FILTERS.includes(key);
  const excluded = SCOPE_EXCLUSIONS[key];
  const structuredOnly = STRUCTURED_ONLY.includes(key);
  if (!excluded && !structuredOnly) return true;
  if (structuredOnly && !structuredRoute(url)) return false;
  if (!route) return url.pathname === '/search';
  if (!excluded) return true;
  const scope = routeScope(url);
  return !scope || !(excluded in scope);
}
