import { DoubleTap } from '#lib/map/doubleTap.ts';
import { attachMarkerDrag } from '#lib/map/markerDrag.ts';
import type { MapMarker } from '#lib/api/map.ts';
import { clusterElement, markedOf, photoElement, showMarked } from '#lib/map/markerElements.ts';
import type { Feature, FeatureCollection } from 'geojson';
import type { GeoJSONSource, Map as MapLibreMap, Marker } from 'maplibre-gl';

type MapLibre = typeof import('maplibre-gl');

const SOURCE = 'immich-edit-photos';
const HIT_LAYER = 'immich-edit-photos-hit';
export const PHOTO_ZOOM = 12;
const CLUSTER_RADIUS = 56;

const DOUBLE_CLICK_MS = 450;

export interface MarkerActions {
  pick(ids: readonly string[]): void;
  focus(ids: readonly string[]): void;
  mark(ids: readonly string[]): void;
  move(ids: readonly string[], lat: number, lon: number): Promise<boolean>;
}

interface Drawn {
  marker: Marker;
  element: HTMLElement;
  clusterId: number | null;
  ids: readonly string[] | null;
  release: () => void;
}

export function pickOrder(features: readonly Feature[]): number[] {
  return features
    .map((feature) => feature.properties?.['order'])
    .filter((order): order is number => typeof order === 'number')
    .sort((a, b) => a - b);
}

export function markerIdentity(
  clusterId: number | null,
  order: number,
  count: number,
  photos: boolean
): string {
  return `${clusterId ?? `p${order}`}:${count}:${photos ? 'photo' : 'dot'}`;
}

export function shifted(a: readonly number[], b: readonly number[]): boolean {
  return Math.abs((a[0] ?? 0) - (b[0] ?? 0)) > 1e-7 || Math.abs((a[1] ?? 0) - (b[1] ?? 0)) > 1e-7;
}

export class PhotoMarkers {
  #map: MapLibreMap;
  #lib: MapLibre;
  #markers: readonly MapMarker[] = [];
  #drawn = new globalThis.Map<string, Drawn>();
  #actions: MarkerActions;
  #selected: (id: string) => boolean = () => false;
  #zoomBack: ReturnType<typeof setTimeout> | undefined;

  constructor(map: MapLibreMap, lib: MapLibre, actions: MarkerActions) {
    this.#map = map;
    this.#lib = lib;
    this.#actions = actions;
    map.on('style.load', this.#install);
    map.on('moveend', this.#sync);
    map.on('sourcedata', this.#onSource);
    if (map.isStyleLoaded()) this.#install();
  }

  set(markers: readonly MapMarker[]): void {
    this.#markers = markers;
    const source = this.#map.getSource<GeoJSONSource>(SOURCE);
    void source?.setData(this.#data());
    this.#clear();
  }

  mark(selected: (id: string) => boolean): void {
    this.#selected = selected;
    for (const drawn of this.#drawn.values()) this.#restyle(drawn);
  }

  destroy(): void {
    clearTimeout(this.#zoomBack);
    this.#map.doubleClickZoom.enable();
    this.#map.off('style.load', this.#install);
    this.#map.off('moveend', this.#sync);
    this.#map.off('sourcedata', this.#onSource);
    this.#clear();
  }

  #data(): FeatureCollection {
    return {
      type: 'FeatureCollection',
      features: this.#markers.map((marker, order) => ({
        type: 'Feature',
        geometry: { type: 'Point', coordinates: [marker.lon, marker.lat] },
        properties: { order }
      }))
    };
  }

  #install = (): void => {
    if (!this.#map.getSource(SOURCE)) {
      this.#map.addSource(SOURCE, {
        type: 'geojson',
        data: this.#data(),
        cluster: true,
        clusterRadius: CLUSTER_RADIUS,
        clusterMaxZoom: 17,
        clusterProperties: { first: ['min', ['get', 'order']] }
      });
    }
    if (!this.#map.getLayer(HIT_LAYER)) {
      this.#map.addLayer({
        id: HIT_LAYER,
        type: 'circle',
        source: SOURCE,
        paint: { 'circle-radius': 1, 'circle-opacity': 0 }
      });
    }
    this.#clear();
  };

  #onSource = (event: { sourceId?: string; isSourceLoaded?: boolean }): void => {
    if (event.sourceId === SOURCE && event.isSourceLoaded) this.#sync();
  };

  #clear(): void {
    for (const drawn of this.#drawn.values()) this.#remove(drawn);
    this.#drawn.clear();
  }

  #remove(drawn: Drawn): void {
    drawn.release();
    drawn.marker.remove();
  }

  #sync = (): void => {
    if (!this.#map.getSource(SOURCE)) return;
    const photos = this.#map.getZoom() >= PHOTO_ZOOM;
    const seen = new Set<string>();
    for (const feature of this.#map.querySourceFeatures(SOURCE)) {
      if (feature.geometry.type !== 'Point') continue;
      const props = feature.properties;
      const clusterId = typeof props['cluster_id'] === 'number' ? props['cluster_id'] : null;
      const order = clusterId === null ? props['order'] : props['first'];
      if (typeof order !== 'number') continue;
      const count = clusterId === null ? 1 : Number(props['point_count'] ?? 1);
      const key = markerIdentity(clusterId, order, count, photos);
      if (seen.has(key)) continue;
      seen.add(key);
      const single = clusterId === null ? this.#markers[order] : undefined;
      const at: [number, number] = single
        ? [single.lon, single.lat]
        : [feature.geometry.coordinates[0] ?? 0, feature.geometry.coordinates[1] ?? 0];
      const drawn = this.#drawn.get(key);
      if (!drawn) {
        this.#drawn.set(key, this.#draw(clusterId, order, count, photos, at));
        continue;
      }
      const now = drawn.marker.getLngLat();
      if (!drawn.element.dataset['dragging'] && shifted([now.lng, now.lat], at)) {
        drawn.marker.setLngLat(at);
      }
    }
    for (const [key, drawn] of this.#drawn) {
      if (seen.has(key)) continue;
      this.#remove(drawn);
      this.#drawn.delete(key);
    }
  };

  #draw(
    clusterId: number | null,
    order: number,
    count: number,
    photos: boolean,
    at: [number, number]
  ): Drawn {
    const single = clusterId === null ? this.#markers[order] : undefined;
    const pinned = photos || count === 1;
    const element = pinned ? photoElement(this.#markers[order], count) : clusterElement(count);
    const anchor = pinned ? 'bottom' : 'center';
    const marker = new this.#lib.Marker({ element, anchor }).setLngLat(at).addTo(this.#map);
    const drawn: Drawn = {
      marker,
      element,
      clusterId,
      ids: single ? [single.id] : null,
      release: () => undefined
    };
    const undrag = attachMarkerDrag(this.#map, marker, element, anchor, ({ to, from }) => {
      void this.#ids(drawn)
        .then((ids) => this.#actions.move(ids, to.lat, to.lng))
        .then((moved) => {
          if (!moved) marker.setLngLat(from);
        });
    });
    const unclick = this.#clicks(drawn);
    drawn.release = () => {
      undrag();
      unclick();
    };
    if (drawn.ids) this.#restyle(drawn);
    else void this.#ids(drawn).then(() => this.#restyle(drawn));
    return drawn;
  }

  #clicks(drawn: Drawn): () => void {
    let timer: ReturnType<typeof setTimeout> | undefined;
    const taps = new DoubleTap(DOUBLE_CLICK_MS);
    const down = (): void => this.#holdZoom();
    const click = (event: MouseEvent): void => {
      event.stopPropagation();
      this.#holdZoom();
      const tap = taps.tap(performance.now(), event.detail);
      if (tap === 'ignore') return;
      clearTimeout(timer);
      if (tap === 'double') {
        void this.#ids(drawn).then((ids) => this.#actions.mark(ids));
        return;
      }
      const ids = this.#ids(drawn);
      void ids.then((list) => this.#actions.pick(list));
      timer = setTimeout(() => {
        void ids.then((list) => this.#actions.focus(list));
      }, DOUBLE_CLICK_MS);
    };
    const double = (event: MouseEvent): void => {
      event.stopPropagation();
      event.preventDefault();
    };
    drawn.element.addEventListener('pointerdown', down);
    drawn.element.addEventListener('click', click);
    drawn.element.addEventListener('dblclick', double);
    return () => {
      clearTimeout(timer);
      drawn.element.removeEventListener('pointerdown', down);
      drawn.element.removeEventListener('click', click);
      drawn.element.removeEventListener('dblclick', double);
    };
  }

  #holdZoom(): void {
    clearTimeout(this.#zoomBack);
    this.#map.doubleClickZoom.disable();
    this.#zoomBack = setTimeout(() => this.#map.doubleClickZoom.enable(), DOUBLE_CLICK_MS * 2);
  }

  #restyle(drawn: Drawn): void {
    if (drawn.ids) showMarked(drawn.element, markedOf(drawn.ids, this.#selected));
  }

  async #ids(drawn: Drawn): Promise<readonly string[]> {
    if (drawn.ids) return drawn.ids;
    const source = this.#map.getSource<GeoJSONSource>(SOURCE);
    if (!source || drawn.clusterId === null) return [];
    const leaves = await source.getClusterLeaves(drawn.clusterId, Number.MAX_SAFE_INTEGER, 0);
    drawn.ids = pickOrder(leaves)
      .map((index) => this.#markers[index]?.id)
      .filter((id): id is string => id !== undefined);
    return drawn.ids;
  }
}
