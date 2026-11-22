import MapPin from '#lib/components/map/MapPin.svelte';
import type { Map as MapLibreMap, Marker } from 'maplibre-gl';
import { mount } from 'svelte';

type MapLibre = typeof import('maplibre-gl');

export function pinElement(size: 'small' | 'large' = 'large'): HTMLDivElement {
  const element = document.createElement('div');
  element.dataset['mapPin'] = '';
  mount(MapPin, { target: element, props: { class: size === 'small' ? 'size-7' : 'size-10' } });
  return element;
}

export function placePin(
  lib: MapLibre,
  map: MapLibreMap,
  at: { lat: number; lon: number },
  options: { draggable?: boolean; size?: 'small' | 'large' } = {}
): Marker {
  const element = pinElement(options.size);
  return new lib.Marker({ element, anchor: 'bottom', draggable: options.draggable ?? false })
    .setLngLat([at.lon, at.lat])
    .addTo(map);
}
