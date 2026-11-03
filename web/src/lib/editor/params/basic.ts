import { neutralCurves } from '#lib/types/edits.ts';
import { field, param } from './define';
import { PERCENT, hundredths } from './range';
import type { ParamDef, SectionDef } from './types';

export const exposure = param({
  id: 'exposure',
  label: 'Exposure',
  synonyms: ['EV', 'stops'],
  min: -5,
  max: 5,
  step: 0.05,
  fineStep: 0.01,
  format: hundredths,
  ...field('basic', 'exposure_ev')
});
export const brightness = param({
  id: 'brightness',
  label: 'Brightness',
  synonyms: ['midtones', 'lightness'],
  ...PERCENT,
  ...field('basic', 'brightness')
});
export const contrast = param({
  id: 'contrast',
  label: 'Contrast',
  synonyms: ['punch'],
  ...PERCENT,
  ...field('basic', 'contrast')
});
export const highlights = param({
  id: 'highlights',
  label: 'Highlights',
  synonyms: ['recovery', 'bright areas'],
  ...PERCENT,
  ...field('tone', 'highlights')
});
export const shadows = param({
  id: 'shadows',
  label: 'Shadows',
  synonyms: ['fill light', 'dark areas'],
  ...PERCENT,
  ...field('tone', 'shadows')
});
export const whites = param({
  id: 'whites',
  label: 'Whites',
  synonyms: ['white point', 'clipping'],
  ...PERCENT,
  ...field('tone', 'whites')
});
export const blacks = param({
  id: 'blacks',
  label: 'Blacks',
  synonyms: ['black point', 'crush'],
  ...PERCENT,
  ...field('tone', 'blacks')
});
export const texture = param({
  id: 'texture',
  label: 'Texture',
  synonyms: ['detail', 'skin'],
  ...PERCENT,
  ...field('basic', 'texture')
});
export const clarity = param({
  id: 'clarity',
  label: 'Clarity',
  synonyms: ['local contrast', 'midtone contrast'],
  ...PERCENT,
  ...field('basic', 'clarity')
});
export const dehaze = param({
  id: 'dehaze',
  label: 'Dehaze',
  synonyms: ['haze', 'fog', 'atmosphere'],
  ...PERCENT,
  ...field('basic', 'dehaze')
});
export const vibrance = param({
  id: 'vibrance',
  label: 'Vibrance',
  synonyms: ['color', 'smart saturation'],
  ...PERCENT,
  ...field('basic', 'vibrance')
});
export const saturation = param({
  id: 'saturation',
  label: 'Saturation',
  synonyms: ['color', 'intensity'],
  ...PERCENT,
  ...field('basic', 'saturation')
});

export const BASIC_PARAMS: readonly ParamDef[] = [
  exposure,
  brightness,
  contrast,
  highlights,
  shadows,
  whites,
  blacks,
  texture,
  clarity,
  dehaze,
  vibrance,
  saturation
];

export const TONE: SectionDef = {
  id: 'tone',
  title: 'Tone',
  rows: [
    [exposure, brightness, contrast],
    [highlights, shadows, whites, blacks]
  ]
};

export const PRESENCE: SectionDef = {
  id: 'presence',
  title: 'Presence',
  rows: [
    [texture, clarity, dehaze],
    [vibrance, saturation]
  ]
};

export const CURVES: SectionDef = {
  id: 'curves',
  title: 'Curves',
  rows: [],
  clear: (edits) => {
    edits.basic.curves = neutralCurves();
  }
};
