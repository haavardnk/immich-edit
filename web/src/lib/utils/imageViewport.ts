import { zoomAnchor, type Rect } from './view-geometry';

export type Pan = { panX: number; panY: number };

export function zoomAtAnchor(
  containerW: number,
  containerH: number,
  frame: Rect,
  anchorX: number,
  anchorY: number,
  previousZoom: number,
  nextZoom: number
): Pan {
  return zoomAnchor(containerW, containerH, frame, anchorX, anchorY, nextZoom / previousZoom);
}

export function splitPosition(clientX: number, rectLeft: number, rectWidth: number): number {
  if (rectWidth <= 0) return 0;
  return Math.min(1, Math.max(0, (clientX - rectLeft) / rectWidth));
}

export function viewportTransform(zoom: number, panX: number, panY: number): string {
  if (zoom === 100 && panX === 0 && panY === 0) return '';
  const scale = zoom / 100;
  return `transform: scale(${scale}) translate(${panX / scale}px, ${panY / scale}px); transform-origin: center;`;
}
