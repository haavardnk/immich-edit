import type { ParamRange } from './types';

export const whole = (value: number): string => value.toFixed(0);

export const hundredths = (value: number): string => value.toFixed(2);

export function signedRange(limit: number): ParamRange {
  return { min: -limit, max: limit, step: 1, fineStep: 1, format: whole };
}
