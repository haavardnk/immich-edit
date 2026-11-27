import type { MarkerFilter } from '#lib/api/map.ts';
import { parseView, readMapFilter, type MapView } from '#lib/map/mapArea.ts';
import { mapFullscreen } from '#lib/map/mapFullscreen.svelte.ts';
import { forgetMapPage, type MapTab } from '#lib/map/mapPage.svelte.ts';

export interface MapMemory {
  view: MapView | null;
  filter: MarkerFilter;
  tab: MapTab;
  panel: boolean | null;
  stars: boolean;
  photos: boolean;
}

const FILTER_KEYS = ['favorite', 'after', 'before'] as const;

const FRESH: MapMemory = {
  view: null,
  filter: {},
  tab: 'view',
  panel: null,
  stars: true,
  photos: true
};

export const mapMemory: MapMemory = { ...FRESH };

export const MAP_EXITS: ReadonlySet<string> = new Set(['/library']);

export function forgetMap(): void {
  Object.assign(mapMemory, FRESH);
  mapFullscreen.exit();
  forgetMapPage();
}

export function mapStart(
  params: Pick<URLSearchParams, 'get' | 'has'>,
  memory: MapMemory
): { view: MapView | null; filter: MarkerFilter } {
  const linked = parseView(params.get('at'));
  const filtered = FILTER_KEYS.some((key) => params.has(key));
  return {
    view: linked ?? memory.view,
    filter: filtered || linked ? readMapFilter(params) : memory.filter
  };
}
