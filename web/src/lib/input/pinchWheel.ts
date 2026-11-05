const PINCH_DELTA_MAX = 50;

export type WheelSample = Pick<WheelEvent, 'ctrlKey' | 'metaKey' | 'deltaMode' | 'deltaY'>;

export function isPinchWheel(e: WheelSample): boolean {
  return e.ctrlKey && !e.metaKey && e.deltaMode === 0 && Math.abs(e.deltaY) < PINCH_DELTA_MAX;
}
