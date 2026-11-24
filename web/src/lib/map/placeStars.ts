import type { SavedPlace } from '#lib/api/places.ts';
import type { Point } from '#lib/gestures/point.ts';
import { mdiStar } from '@mdi/js';
import type { Map as MapLibreMap, Marker } from 'maplibre-gl';

type MapLibre = typeof import('maplibre-gl');

export const SNAP_PX = 32;
export const NAME_ZOOM = 10;

const SVG = 'http://www.w3.org/2000/svg';

interface Drawn {
  place: SavedPlace;
  marker: Marker;
  element: HTMLButtonElement;
}

function starIcon(): SVGSVGElement {
  const svg = document.createElementNS(SVG, 'svg');
  svg.setAttribute('viewBox', '0 0 24 24');
  svg.setAttribute('aria-hidden', 'true');
  svg.setAttribute('class', 'size-3.5 fill-primary');
  const path = document.createElementNS(SVG, 'path');
  path.setAttribute('d', mdiStar);
  svg.append(path);
  return svg;
}

export function starElement(place: SavedPlace): HTMLButtonElement {
  const element = document.createElement('button');
  element.type = 'button';
  element.dataset['mapPlace'] = place.id;
  element.className =
    'group flex items-center gap-1.5 outline-none data-[hit=true]:z-10 [&[data-names=false]_[data-place-name]]:hidden';
  const dot = document.createElement('span');
  dot.className =
    'grid size-7 place-items-center rounded-full bg-light-100 shadow-lg ring-2 ring-primary group-focus-visible:ring-light-950 group-data-[hit=true]:scale-125 group-data-[hit=true]:ring-3 group-data-[hit=true]:ring-light-950 motion-safe:transition-transform motion-safe:duration-fast';
  dot.append(starIcon());
  const name = document.createElement('span');
  name.dataset['placeName'] = '';
  name.className =
    'rounded-md bg-light-100/85 px-1.5 py-0.5 text-caption font-semibold whitespace-nowrap text-light-950 shadow';
  element.append(dot, name);
  return element;
}

function label(element: HTMLButtonElement, place: SavedPlace): void {
  element.setAttribute('aria-label', `Saved place ${place.name}`);
  const name = element.querySelector('[data-place-name]');
  if (name) name.textContent = place.name;
}

export class PlaceStars {
  #map: MapLibreMap;
  #lib: MapLibre;
  #onPick: (place: SavedPlace, element: HTMLElement) => void;
  #drawn = new Map<string, Drawn>();

  constructor(
    map: MapLibreMap,
    lib: MapLibre,
    onPick: (place: SavedPlace, element: HTMLElement) => void
  ) {
    this.#map = map;
    this.#lib = lib;
    this.#onPick = onPick;
    map.on('zoom', this.#names);
  }

  set(places: readonly SavedPlace[]): void {
    const keep = new Set(places.map((place) => place.id));
    for (const [id, drawn] of this.#drawn) {
      if (keep.has(id)) continue;
      drawn.marker.remove();
      this.#drawn.delete(id);
    }
    for (const place of places) {
      const drawn = this.#drawn.get(place.id) ?? this.#draw(place);
      drawn.place = place;
      drawn.marker.setLngLat([place.lon, place.lat]);
      label(drawn.element, place);
    }
    this.#names();
  }

  nearest(point: Point, within: number = SNAP_PX): SavedPlace | null {
    let best: { place: SavedPlace; distance: number } | null = null;
    for (const { place } of this.#drawn.values()) {
      const at = this.#map.project([place.lon, place.lat]);
      const distance = Math.hypot(at.x - point.x, at.y - point.y);
      if (distance <= within && (!best || distance < best.distance)) best = { place, distance };
    }
    return best?.place ?? null;
  }

  hit(id: string | null): void {
    for (const [key, { element }] of this.#drawn) {
      element.dataset['hit'] = String(key === id);
    }
  }

  destroy(): void {
    this.#map.off('zoom', this.#names);
    for (const { marker } of this.#drawn.values()) marker.remove();
    this.#drawn.clear();
  }

  #draw(place: SavedPlace): Drawn {
    const element = starElement(place);
    const marker = new this.#lib.Marker({ element, anchor: 'left', offset: [-14, 0] })
      .setLngLat([place.lon, place.lat])
      .addTo(this.#map);
    const drawn: Drawn = { place, marker, element };
    element.addEventListener('click', (event) => {
      event.stopPropagation();
      this.#onPick(drawn.place, element);
    });
    this.#drawn.set(place.id, drawn);
    return drawn;
  }

  #names = (): void => {
    const shown = String(this.#map.getZoom() >= NAME_ZOOM);
    for (const { element } of this.#drawn.values()) element.dataset['names'] = shown;
  };
}
