import { readStored, writeStored } from '$lib/utils/storage';

export const MAX_ZOOM = 800;
export const ZOOM_STOPS = [25, 33, 50, 66, 100, 200, 300, 400, MAX_ZOOM];
export const SLIDER_STEPS = 1000;

const STOP_EPSILON = 0.5;
const ZOOM_LEVEL_KEY = 'immich-edit:zoomLevel';
const DEFAULT_ZOOM_LEVEL = 100;

type PersistedZoomLevel = { zoom?: number };

function floorZoom(fitZoom: number): number {
  if (!Number.isFinite(fitZoom) || fitZoom <= 0) return 1;
  return Math.min(fitZoom, 100);
}

function ceilingZoom(fitZoom: number): number {
  if (!Number.isFinite(fitZoom) || fitZoom <= 0) return MAX_ZOOM;
  return Math.max(MAX_ZOOM, fitZoom);
}

export function clampZoom(zoom: number, fitZoom: number): number {
  const floor = floorZoom(fitZoom);
  if (!Number.isFinite(zoom)) return floor;
  return Math.min(ceilingZoom(fitZoom), Math.max(floor, zoom));
}

export function zoomStops(fitZoom: number): number[] {
  const floor = floorZoom(fitZoom);
  const ceiling = ceilingZoom(fitZoom);
  const inner = ZOOM_STOPS.filter((stop) => stop > floor + STOP_EPSILON && stop < ceiling);
  return [floor, ...inner, ceiling];
}

export function nextStop(zoom: number, direction: 1 | -1, fitZoom: number): number {
  const stops = zoomStops(fitZoom);
  const current = clampZoom(zoom, fitZoom);
  if (direction === 1) return stops.find((stop) => stop > current + STOP_EPSILON) ?? current;
  return [...stops].reverse().find((stop) => stop < current - STOP_EPSILON) ?? current;
}

export function zoomToSlider(zoom: number, fitZoom: number): number {
  const floor = floorZoom(fitZoom);
  const span = Math.log(ceilingZoom(fitZoom) / floor);
  if (span <= 0) return SLIDER_STEPS;
  return Math.round((Math.log(clampZoom(zoom, fitZoom) / floor) / span) * SLIDER_STEPS);
}

export function sliderToZoom(value: number, fitZoom: number): number {
  const floor = floorZoom(fitZoom);
  const ceiling = ceilingZoom(fitZoom);
  if (ceiling <= floor) return floor;
  const t = Math.min(1, Math.max(0, value / SLIDER_STEPS));
  return floor * Math.pow(ceiling / floor, t);
}

export function readZoomLevel(): number {
  const stored = readStored<PersistedZoomLevel>(ZOOM_LEVEL_KEY);
  if (typeof stored?.zoom !== 'number' || !Number.isFinite(stored.zoom)) return DEFAULT_ZOOM_LEVEL;
  return Math.min(MAX_ZOOM, Math.max(1, stored.zoom));
}

export function writeZoomLevel(zoom: number): void {
  writeStored(ZOOM_LEVEL_KEY, { zoom } satisfies PersistedZoomLevel);
}
