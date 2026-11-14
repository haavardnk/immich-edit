import { PERCENT } from './range';
import type { ParamRange } from './types';

export const STRAIGHTEN: ParamRange = {
  min: -45,
  max: 45,
  step: 0.1,
  fineStep: 0.1,
  format: (value) => `${value.toFixed(1)}°`
};

export const PERSPECTIVE: ParamRange = PERCENT;
