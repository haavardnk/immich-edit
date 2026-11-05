import type { ParamRange, ToolRange } from './types';

export const whole = (value: number): string => value.toFixed(0);

export const tenths = (value: number): string => value.toFixed(1);

export const hundredths = (value: number): string => value.toFixed(2);

export const thousandths = (value: number): string => value.toFixed(3);

export function signedRange(limit: number): ParamRange {
  return { min: -limit, max: limit, step: 1, fineStep: 1, format: whole };
}

export function amountRange(max: number): ParamRange {
  return { min: 0, max, step: 1, fineStep: 1, format: whole };
}

export const PERCENT = signedRange(100);

export const AMOUNT = amountRange(100);

export const HUE = amountRange(360);

export function toolRange(
  min: number,
  max: number,
  step: number,
  defaultValue: number,
  options: Partial<Pick<ToolRange, 'format' | 'keyStep'>> = {}
): ToolRange {
  return {
    min,
    max,
    step,
    fineStep: step,
    format: hundredths,
    keyStep: step,
    defaultValue,
    ...options
  };
}
