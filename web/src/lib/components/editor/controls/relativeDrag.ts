import type { ParamRange } from '#lib/editor/params/types.ts';
import { SCALE_POSITIONS } from './sliderScale';
import { clampTo, settleValue } from './sliderStep';

export const FINE_FROM = 40;
export const FINE_SPAN = 80;
export const MIN_FINE = 0.1;

export type DragTrack = { min: number; max: number; width: number };

export function fineFactor(offsetY: number): number {
  const beyond = Math.max(0, Math.abs(offsetY) - FINE_FROM);
  return Math.max(MIN_FINE, 1 / (1 + beyond / FINE_SPAN));
}

export function dragPosition(
  position: number,
  stepX: number,
  offsetY: number,
  track: DragTrack
): number {
  if (track.width <= 0) return position;
  const next = position + (stepX / track.width) * (track.max - track.min) * fineFactor(offsetY);
  return Math.min(track.max, Math.max(track.min, next));
}

export function dragBounds(range: ParamRange): { min: number; max: number } {
  return range.scale ? { min: 0, max: SCALE_POSITIONS } : { min: range.min, max: range.max };
}

export function positionOf(value: number, range: ParamRange): number {
  return range.scale ? range.scale.toPosition(value) : value;
}

export function valueAt(position: number, range: ParamRange): number {
  if (!range.scale) return settleValue(position, range);
  return clampTo(range.scale.fromPosition(Math.round(position)), range.min, range.max);
}
