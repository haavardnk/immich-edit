import type { ParamRange } from '#lib/editor/params/types.ts';
import { SCALE_POSITIONS } from './sliderScale';

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

function decimals(step: number): number {
  const text = String(step);
  const dot = text.indexOf('.');
  return dot < 0 ? 0 : text.length - dot - 1;
}

export function snapToStep(value: number, min: number, step: number): number {
  if (step <= 0) return value;
  const snapped = min + Math.round((value - min) / step) * step;
  const rounded = Number(snapped.toFixed(decimals(step)));
  return Object.is(rounded, -0) ? 0 : rounded;
}

export function dragBounds(range: ParamRange): { min: number; max: number } {
  return range.scale ? { min: 0, max: SCALE_POSITIONS } : { min: range.min, max: range.max };
}

export function positionOf(value: number, range: ParamRange): number {
  return range.scale ? range.scale.toPosition(value) : value;
}

export function valueAt(position: number, range: ParamRange): number {
  const raw = range.scale
    ? range.scale.fromPosition(Math.round(position))
    : snapToStep(position, range.min, range.step);
  const clamped = Math.min(range.max, Math.max(range.min, raw));
  return Object.is(clamped, -0) ? 0 : clamped;
}
