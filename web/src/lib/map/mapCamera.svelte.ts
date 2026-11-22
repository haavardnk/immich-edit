import type { MapMarker } from '#lib/api/map.ts';
import { boundsOf, sameView, type LngLatBox, type MapView } from '#lib/map/mapArea.ts';
import { FIT_MS, FLY_MAX_MS } from '#lib/map/motion.ts';
import { PHOTO_ZOOM } from '#lib/map/photoMarkers.ts';
import type { Map as MapLibreMap, PaddingOptions } from 'maplibre-gl';

const ALL_MAX_ZOOM = 14;
const PICK_MAX_ZOOM = 16;

export type CameraPadding = number | PaddingOptions;

export function boxOf(target: MapLibreMap): LngLatBox {
  const bounds = target.getBounds();
  return {
    west: bounds.getWest(),
    south: bounds.getSouth(),
    east: bounds.getEast(),
    north: bounds.getNorth()
  };
}

export class MapCamera {
  map = $state.raw<MapLibreMap | null>(null);
  shown: MapView | null = null;
  #padding: () => CameraPadding;

  constructor(padding: () => CameraPadding) {
    this.#padding = padding;
  }

  fitAll(extent: LngLatBox | null, animate = true): void {
    if (extent) this.#fit(extent, animate ? FIT_MS : 0, ALL_MAX_ZOOM);
  }

  focus(markers: readonly MapMarker[], ids: readonly string[]): void {
    const chosen = new Set(ids);
    const extent = boundsOf(markers.filter((marker) => chosen.has(marker.id)));
    if (extent) this.#fit(extent, FIT_MS, PICK_MAX_ZOOM);
  }

  follow(view: MapView): boolean {
    const map = this.map;
    if (!map || (this.shown && sameView(view, this.shown))) return false;
    map.flyTo({ center: [view.lon, view.lat], zoom: view.zoom, maxDuration: FLY_MAX_MS });
    return true;
  }

  #fit(extent: LngLatBox, duration: number, maxZoom: number): void {
    const map = this.map;
    if (!map) return;
    const padding = this.#padding();
    if (extent.west === extent.east && extent.south === extent.north) {
      map.easeTo({
        center: [extent.west, extent.south],
        zoom: Math.max(map.getZoom(), PHOTO_ZOOM),
        padding,
        duration
      });
      return;
    }
    map.fitBounds(
      [
        [extent.west, extent.south],
        [extent.east, extent.north]
      ],
      { padding, maxZoom, duration }
    );
  }
}
