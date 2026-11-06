import { validReturnPath } from '#lib/editor/navigation.ts';
import { arrangeLoaded } from './loadedFilter';
import { browseControls } from '#lib/stores/browseControls.svelte.ts';
import { readFilterParams } from './filterParams';
import { matchBrowseRoute } from './browseRoutes';
import { BrowseFeed } from '#lib/stores/browseFeed.svelte.ts';
import { browsing } from '#lib/stores/browsing.svelte.ts';
import { loadCullTags } from '#lib/stores/cullTags.ts';

const RESTORE_REACH = { before: 250, after: 250 };

export async function restoreBrowse(from: string | null, assetId: string): Promise<void> {
  const returnPath = validReturnPath(from) ? from : null;
  const url = new URL(returnPath ?? '/photos', 'http://restore.invalid');
  const route = matchBrowseRoute(url);
  if (!route) return;

  browseControls.enter(route.family, readFilterParams(url));

  if ('feed' in route) {
    const feed = new BrowseFeed(route.feed);
    if (await feed.jumpTo(assetId, { reach: RESTORE_REACH, failure: null })) return;
    if (returnPath) await feed.fetchPage(true);
    return;
  }
  await loadCullTags().catch(() => undefined);
  const assets = await route.load();
  browsing.set(arrangeLoaded(assets, route.sortKey()));
}
