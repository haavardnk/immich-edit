import { miredScale } from '#lib/components/editor/controls/sliderScale.ts';
import {
  asShotWhiteBalance,
  KELVIN_MAX,
  KELVIN_MIN,
  RELATIVE_LIMIT,
  TINT_LIMIT
} from '#lib/types/edits.ts';
import { signedRange } from './range';
import type { ParamRange, SectionDef } from './types';

const TEMPERATURE_GRADIENT = 'var(--gradient-temperature)';
const TINT_GRADIENT = 'var(--gradient-tint)';

export const KELVIN: ParamRange = {
  min: KELVIN_MIN,
  max: KELVIN_MAX,
  step: 1,
  fineStep: 1,
  scale: miredScale(KELVIN_MIN, KELVIN_MAX),
  format: (value) => `${value.toFixed(0)} K`,
  gradient: TEMPERATURE_GRADIENT
};

export const TINT: ParamRange = { ...signedRange(TINT_LIMIT), gradient: TINT_GRADIENT };

export const SHIFT_TEMPERATURE: ParamRange = {
  ...signedRange(RELATIVE_LIMIT),
  gradient: TEMPERATURE_GRADIENT
};

export const SHIFT_TINT: ParamRange = { ...signedRange(RELATIVE_LIMIT), gradient: TINT_GRADIENT };

export const WHITE_BALANCE: SectionDef = {
  id: 'white_balance',
  title: 'White Balance',
  rows: [],
  clear: (edits) => {
    edits.basic.white_balance = asShotWhiteBalance();
  }
};
