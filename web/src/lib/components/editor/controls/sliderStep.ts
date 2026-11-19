import type { ParamRange } from '#lib/editor/params/types.ts';
import { SCALE_POSITIONS } from './sliderScale';

export type StepSize = 'fine' | 'normal' | 'coarse';

export const COARSE_STEPS = 10;

export function decimalsOf(step: number): number {
  const text = String(step);
  const dot = text.indexOf('.');
  return dot < 0 ? 0 : text.length - dot - 1;
}

export function clampTo(value: number, min: number, max: number): number {
  const clamped = Math.min(max, Math.max(min, value));
  return Object.is(clamped, -0) ? 0 : clamped;
}

export function snapToStep(value: number, min: number, step: number): number {
  if (step <= 0) return value;
  const snapped = min + Math.round((value - min) / step) * step;
  const rounded = Number(snapped.toFixed(decimalsOf(step)));
  return Object.is(rounded, -0) ? 0 : rounded;
}

export function settleValue(value: number, range: ParamRange): number {
  return clampTo(snapToStep(value, range.min, range.fineStep), range.min, range.max);
}

export function nudgeValue(
  value: number,
  range: ParamRange,
  steps: number,
  size: StepSize
): number {
  if (range.scale) {
    const stride = size === 'coarse' ? COARSE_STEPS : 1;
    const position = clampTo(
      Math.round(range.scale.toPosition(value)) + steps * stride,
      0,
      SCALE_POSITIONS
    );
    return clampTo(range.scale.fromPosition(position), range.min, range.max);
  }
  const unit =
    size === 'fine' ? range.fineStep : size === 'coarse' ? range.step * COARSE_STEPS : range.step;
  return settleValue(value + steps * unit, range);
}

export function entryText(value: number, range: ParamRange): string {
  const text = String(Number(value.toFixed(decimalsOf(range.fineStep))));
  return text === '-0' ? '0' : text;
}
