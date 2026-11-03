import { field, param } from './define';
import { amountRange } from './range';
import type { ParamDef, SectionDef } from './types';

export const lensDistortion = param({
  id: 'lens_distortion',
  label: 'Distortion',
  history: 'Lens Distortion',
  synonyms: ['lens profile', 'barrel', 'pincushion'],
  ...amountRange(200),
  defaultValue: 100,
  ...field('lens', 'distortion_amount')
});

export const lensVignetting = param({
  id: 'lens_vignetting',
  label: 'Vignetting',
  history: 'Lens Vignetting',
  synonyms: ['lens profile', 'falloff'],
  ...amountRange(200),
  defaultValue: 100,
  ...field('lens', 'vignette_amount')
});

export const LENS_PARAMS: readonly ParamDef[] = [lensDistortion, lensVignetting];

export const LENS: SectionDef = {
  id: 'lens',
  title: 'Lens Profile',
  rows: [[lensDistortion, lensVignetting]]
};
