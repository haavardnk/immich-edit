import type { MaskedEditKey } from '#lib/types/edits.ts';
import {
  blacks,
  brightness,
  clarity,
  contrast,
  exposure,
  highlights,
  saturation,
  shadows,
  texture,
  vibrance,
  whites
} from './basic';
import { hundredths, signedRange } from './range';
import type { ParamRange } from './types';
import { SHIFT_TEMPERATURE, SHIFT_TINT } from './whiteBalance';

export interface MaskParam extends ParamRange {
  key: MaskedEditKey;
  label: string;
}

function maskParam(
  key: MaskedEditKey,
  { label, min, max, step, fineStep, format, gradient, scale }: ParamRange & { label: string }
): MaskParam {
  return { key, label, min, max, step, fineStep, format, gradient, scale };
}

export const MASK_AMOUNT: ParamRange & { defaultValue: number } = {
  min: 0,
  max: 1,
  step: 0.01,
  fineStep: 0.01,
  defaultValue: 1,
  format: hundredths
};

export const MASK_PARAM_ROWS: readonly (readonly MaskParam[])[] = [
  [
    maskParam('wb_temp', { label: 'Temperature', ...SHIFT_TEMPERATURE }),
    maskParam('wb_tint', { label: 'Tint', ...SHIFT_TINT })
  ],
  [
    maskParam('exposure_ev', exposure),
    maskParam('brightness', brightness),
    maskParam('contrast', contrast),
    maskParam('highlights', highlights),
    maskParam('shadows', shadows),
    maskParam('whites', whites),
    maskParam('blacks', blacks)
  ],
  [
    maskParam('texture', texture),
    maskParam('clarity', clarity),
    maskParam('sharpen', { label: 'Sharpening', ...signedRange(150) })
  ],
  [maskParam('saturation', saturation), maskParam('vibrance', vibrance)]
];
