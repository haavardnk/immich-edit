import { lensBlurActive } from '#lib/types/edits.ts';
import { field, param } from './define';
import { AMOUNT } from './range';
import type { ParamDef, SectionDef } from './types';

export const lensBlurAmount = param({
  id: 'lens_blur_amount',
  label: 'Amount',
  history: 'Lens Blur Amount',
  synonyms: ['lens blur', 'bokeh', 'depth of field', 'background blur'],
  ...AMOUNT,
  ...field('lens_blur', 'amount')
});

export const lensBlurBoost = param({
  id: 'lens_blur_boost',
  label: 'Boost',
  history: 'Lens Blur Boost',
  synonyms: ['lens blur', 'bokeh', 'highlights'],
  ...AMOUNT,
  inactive: (edits) => !lensBlurActive(edits.lens_blur),
  ...field('lens_blur', 'boost')
});

export const LENS_BLUR_PARAMS: readonly ParamDef[] = [lensBlurAmount, lensBlurBoost];

export const LENS_BLUR: SectionDef = {
  id: 'lens_blur',
  title: 'Blur',
  rows: [[lensBlurAmount, lensBlurBoost]]
};
