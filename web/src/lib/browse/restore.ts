import { searchWindow } from '$lib/api/search';
import { validReturnPath } from '$lib/editor/navigation';
import { arrangeLoaded } from './loadedFilter';
import { browseControls } from '$lib/stores/browseControls.svelte';
import { readFilterParams } from './filterParams';
import { matchBrowseRoute } from './browseRoutes';
import { BrowseFeed, type BrowseFeedOptions } from '$lib/stores/browseFeed.svelte';
import { browsing } from '$lib/stores/browsing.svelte';
import { loadCullTags, stampCullTags } from '$lib/stores/cullTags';

const WINDOW_RADIUS = 250;

export async function restoreBrowse(from: string | null, assetId: string): Promise<void> {
  const returnPath = validReturnPath(from) ? from : null;
  const url = new URL(returnPath ?? '/photos', 'http://restore.invalid');
  const route = matchBrowseRoute(url);
  if (!route) return;

  browseControls.enter(route.family, readFilterParams(url));

  if ('feed' in route) {
    if (!route.feed.fetcher && (await restoreWindow(route.feed, assetId))) return;
    if (returnPath) await new BrowseFeed(route.feed).fetchPage(true);
    return;
  }
  await loadCullTags().catch(() => undefined);
  const assets = await route.load();
  browsing.set(arrangeLoaded(assets, route.sortKey()));
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
