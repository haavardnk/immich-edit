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

export function viewportTransform(fitRatio: number, panX: number, panY: number): string {
  if (fitRatio === 1 && panX === 0 && panY === 0) return '';
  return `transform: scale(${fitRatio}) translate(${panX / fitRatio}px, ${panY / fitRatio}px); transform-origin: center;`;
}
