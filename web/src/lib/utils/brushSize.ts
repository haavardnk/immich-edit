import type { PreviewMeta } from '$lib/types/preview';

export type BrushRange = { step: number; min: number; max: number };

export const RETOUCH_SIZE: BrushRange = { step: 0.005, min: 0.005, max: 0.3 };
export const BRUSH_SIZE: BrushRange = { step: 0.01, min: 0.005, max: 0.5 };
export const HARDNESS: BrushRange = { step: 0.1, min: 0, max: 1 };

const WHEEL_NOTCH_PX = 100;
const WHEEL_LINE_PX = 16;
const SIZE_PER_NOTCH = 1.1;

export function formatBrushSize(size: number, meta: PreviewMeta | null): string {
  const shortEdge = meta ? Math.min(meta.source_w, meta.source_h) : 0;
  return shortEdge > 1 ? `${Math.round(size * shortEdge)} px` : size.toFixed(3);
}

function clamp(value: number, { min, max }: BrushRange): number {
  return Math.min(max, Math.max(min, value));
}

export function stepBrush(current: number, key: string, range: BrushRange): number {
  const delta = key === '[' || key === '{' ? -range.step : range.step;
  return clamp(current + delta, range);
}

export function wheelNotches(e: Pick<WheelEvent, 'deltaX' | 'deltaY' | 'deltaMode'>): number {
  const delta = e.deltaY || e.deltaX;
  const px = e.deltaMode === 1 ? delta * WHEEL_LINE_PX : delta;
  return -px / WHEEL_NOTCH_PX;
}

export function wheelSize(current: number, notches: number, range: BrushRange): number {
  return clamp(current * SIZE_PER_NOTCH ** notches, range);
}

export function wheelHardness(current: number, notches: number, range: BrushRange): number {
  return clamp(current + notches * range.step, range);
}
