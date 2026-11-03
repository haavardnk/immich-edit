import type { ParamRange } from './types';

export const whole = (value: number): string => value.toFixed(0);

export const tenths = (value: number): string => value.toFixed(1);

export const hundredths = (value: number): string => value.toFixed(2);

export function signedRange(limit: number): ParamRange {
  return { min: -limit, max: limit, step: 1, fineStep: 1, format: whole };
}

export function amountRange(max: number): ParamRange {
  return { min: 0, max, step: 1, fineStep: 1, format: whole };
}

export const PERCENT = signedRange(100);

export const AMOUNT = amountRange(100);
