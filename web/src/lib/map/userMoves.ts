import type { Map as MapLibreMap } from 'maplibre-gl';

export function onUserMove(map: MapLibreMap, moved: () => void): () => void {
  const zoom = (event: { originalEvent?: Event }): void => {
    if (event.originalEvent) moved();
  };
  const touch = (event: { originalEvent: TouchEvent }): void => {
    if (event.originalEvent.touches.length > 1) moved();
  };
  map.on('wheel', moved);
  map.on('dragstart', moved);
  map.on('zoomstart', zoom);
  map.on('touchstart', touch);
  return () => {
    map.off('wheel', moved);
    map.off('dragstart', moved);
    map.off('zoomstart', zoom);
    map.off('touchstart', touch);
  };
}
