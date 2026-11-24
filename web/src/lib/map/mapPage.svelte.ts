import { getMapMarkers, type MapMarker, type MarkerFilter } from '#lib/api/map.ts';
import {
  boundsOf,
  markerQuery,
  markersIn,
  placeOfMarkers,
  type LngLatBox
} from '#lib/map/mapArea.ts';
import type { LocationMove } from '#lib/map/locationMoves.ts';
import { MapFeed } from '#lib/map/mapFeed.svelte.ts';
import { UnplacedFeed } from '#lib/map/unplacedFeed.svelte.ts';
import { errorMessage } from '#lib/utils/errors.ts';
import { countOf } from '#lib/utils/plural.ts';

export type MapTab = 'view' | 'unplaced';

export class MapPageState {
  filter = $state<MarkerFilter>({});
  markers = $state.raw<MapMarker[]>([]);
  loaded = $state(false);
  error = $state<string | null>(null);
  box = $state.raw<LngLatBox | null>(null);
  picked = $state.raw<readonly string[] | null>(null);
  tab = $state<MapTab>('view');
  readonly feed = new MapFeed();
  readonly unplaced = new UnplacedFeed();
  #request = 0;

  readonly inView = $derived(this.box ? markersIn(this.markers, this.box) : this.markers);
  readonly shownIds = $derived(this.picked ?? this.inView.map((marker) => marker.id));
  readonly place = $derived.by(() => {
    const picked = this.picked;
    if (!picked) return placeOfMarkers(this.inView);
    const ids = new Set(picked);
    return placeOfMarkers(this.markers.filter((marker) => ids.has(marker.id)));
  });
  readonly extent = $derived(boundsOf(this.markers));
  readonly title = $derived.by(() => {
    if (this.tab === 'unplaced') {
      const shown = this.unplaced.assets.length;
      const more = this.unplaced.hasMore ? '+' : '';
      return `${shown}${more} ${shown === 1 && !more ? 'photo' : 'photos'} without location`;
    }
    const count = countOf(this.shownIds.length, 'photo');
    return this.picked ? count : `${count} in view`;
  });

  constructor(filter: MarkerFilter) {
    this.filter = filter;
  }

  async load(): Promise<void> {
    const request = ++this.#request;
    this.error = null;
    try {
      const markers = await getMapMarkers(markerQuery(this.filter));
      if (request !== this.#request) return;
      this.markers = markers;
      this.loaded = true;
    } catch (error) {
      if (request !== this.#request) return;
      this.error = errorMessage(error);
    }
  }

  move(moves: readonly LocationMove[]): void {
    const byId = new Map(moves.map((move) => [move.id, move]));
    const known = new Set(this.markers.map((marker) => marker.id));
    this.markers = this.markers.map((marker) => {
      const move = byId.get(marker.id);
      if (!move) return marker;
      return {
        ...marker,
        lat: move.lat,
        lon: move.lon,
        city: move.place?.city ?? null,
        state: move.place?.state ?? null,
        country: move.place?.country ?? null
      };
    });
    if (moves.some((move) => !known.has(move.id))) void this.load();
    this.unplaced.remove(moves.map((move) => move.id));
  }

  setFilter(filter: MarkerFilter): void {
    this.filter = filter;
    this.picked = null;
    void this.load();
  }

  pick(ids: readonly string[]): void {
    this.picked = ids;
  }

  showAll(): void {
    this.picked = null;
  }
}

let kept: { state: MapPageState; release: () => void } | null = null;

function sameFilter(a: MarkerFilter, b: MarkerFilter): boolean {
  return JSON.stringify(markerQuery(a)) === JSON.stringify(markerQuery(b));
}

export function mapPageState(filter: MarkerFilter): MapPageState {
  if (kept && sameFilter(kept.state.filter, filter)) return kept.state;
  forgetMapPage();
  let state: MapPageState | null = null;
  const release = $effect.root(() => {
    state = new MapPageState(filter);
  });
  kept = { state: state!, release };
  return kept.state;
}

export function forgetMapPage(): void {
  kept?.release();
  kept = null;
}
