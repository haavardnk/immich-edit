import type { Map as MapLibreMap } from 'maplibre-gl';

export interface ScreenPoint {
  x: number;
  y: number;
  width: number;
}

export function followPoint(
  map: MapLibreMap,
  at: { lat: number; lon: number },
  onPoint: (point: ScreenPoint) => void
): () => void {
  const place = (): void => {
    const projected = map.project([at.lon, at.lat]);
    onPoint({ x: projected.x, y: projected.y, width: map.getContainer().clientWidth });
  };
  place();
  map.on('move', place);
  return () => map.off('move', place);
}
