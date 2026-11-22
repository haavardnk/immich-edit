import { replaceState } from '$app/navigation';
import { page } from '$app/state';
import type { MarkerFilter } from '#lib/api/map.ts';
import { formatView, writeMapFilter, type MapView } from '#lib/map/mapArea.ts';

export function writeMapUrl(view: MapView | null, filter: MarkerFilter): void {
  const url = new URL(page.url.href);
  if (view) url.searchParams.set('at', formatView(view));
  writeMapFilter(url.searchParams, filter);
  void replaceState(url, page.state);
}
