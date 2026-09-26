import { zoomAnchor, type Rect } from './view-geometry';

export type Pan = { panX: number; panY: number };
export type PanStep = [number, number];

const PAN_STEP = 0.1;
const PAN_STEPS: Record<string, PanStep> = {
  ArrowLeft: [-PAN_STEP, 0],
  ArrowRight: [PAN_STEP, 0],
  ArrowUp: [0, -PAN_STEP],
  ArrowDown: [0, PAN_STEP]
};

export function panStep(key: string): PanStep | null {
  return PAN_STEPS[key] ?? null;
}

export function nudgePan(
  pan: Pan,
  step: PanStep,
  frame: { width: number; height: number },
  viewW: number,
  viewH: number
): Pan {
  const limitX = Math.max(0, (frame.width - viewW) / 2);
  const limitY = Math.max(0, (frame.height - viewH) / 2);
  return {
    panX: Math.min(limitX, Math.max(-limitX, pan.panX - step[0] * viewW)),
    panY: Math.min(limitY, Math.max(-limitY, pan.panY - step[1] * viewH))
  };
}

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
