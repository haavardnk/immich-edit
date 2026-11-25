import type { PlaceMenuAt } from '#lib/map/placeEditor.svelte.ts';
import type { Map as MapLibreMap, MapMouseEvent, MapTouchEvent } from 'maplibre-gl';

const HOLD_MS = 550;
const SLOP_PX = 10;
const REPEAT_MS = 800;

function onCanvas(event: Event): boolean {
  return event.target instanceof HTMLCanvasElement;
}

export function watchPlaceMenu(map: MapLibreMap, onMenu: (menu: PlaceMenuAt) => void): () => void {
  let timer: ReturnType<typeof setTimeout> | null = null;
  let start: { x: number; y: number } | null = null;
  let firedAt = Number.NEGATIVE_INFINITY;

  const fire = (x: number, y: number): void => {
    const now = performance.now();
    if (now - firedAt < REPEAT_MS) return;
    firedAt = now;
    const at = map.unproject([x, y]);
    onMenu({ x, y, at: { lat: at.lat, lon: at.lng } });
  };

  const stop = (): void => {
    if (timer) clearTimeout(timer);
    timer = null;
    start = null;
  };

  const context = (event: MapMouseEvent): void => {
    if (!onCanvas(event.originalEvent)) return;
    event.preventDefault();
    fire(event.point.x, event.point.y);
  };

  const touchStart = (event: MapTouchEvent): void => {
    stop();
    if (event.points.length !== 1 || !onCanvas(event.originalEvent)) return;
    const at = { x: event.point.x, y: event.point.y };
    start = at;
    timer = setTimeout(() => {
      timer = null;
      if (start === at) fire(at.x, at.y);
    }, HOLD_MS);
  };

  const touchMove = (event: MapTouchEvent): void => {
    if (!start) return;
    const moved = Math.hypot(event.point.x - start.x, event.point.y - start.y);
    if (event.points.length !== 1 || moved > SLOP_PX) stop();
  };

  map.on('contextmenu', context);
  map.on('touchstart', touchStart);
  map.on('touchmove', touchMove);
  map.on('touchend', stop);
  map.on('touchcancel', stop);
  map.on('movestart', stop);
  return () => {
    stop();
    map.off('contextmenu', context);
    map.off('touchstart', touchStart);
    map.off('touchmove', touchMove);
    map.off('touchend', stop);
    map.off('touchcancel', stop);
    map.off('movestart', stop);
  };
}
