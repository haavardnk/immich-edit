import { replaceState } from '$app/navigation';
import { navigating, page } from '$app/state';
import type { MarkerFilter } from '#lib/api/map.ts';
import { dropMapPhoto, formatView, writeMapFilter, type MapView } from '#lib/map/mapArea.ts';
import { liveUrl } from '#lib/utils/route.ts';

export function writeMapUrl(view: MapView | null, filter: MarkerFilter): void {
  if (navigating.type !== null) return;
  const url = new URL(liveUrl().href);
  if (view) url.searchParams.set('at', formatView(view));
  writeMapFilter(url.searchParams, filter);
  dropMapPhoto(url.searchParams);
  void replaceState(url, page.state);
}
