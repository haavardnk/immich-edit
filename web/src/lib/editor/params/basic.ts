import type { BasicEdits, ToneEdits } from '#lib/types/edits.ts';
import { hundredths, signedRange } from './range';
import type { ParamDef, ParamRange, SectionDef } from './types';

type BasicKey = Exclude<keyof BasicEdits, 'white_balance' | 'curves'>;

interface FieldSpec {
  id: string;
  label: string;
  synonyms: readonly string[];
  range?: ParamRange;
}

const PERCENT = signedRange(100);

function fieldParam(
  { range = PERCENT, ...spec }: FieldSpec,
  get: ParamDef['get'],
  set: ParamDef['set']
): ParamDef {
  return { ...spec, ...range, defaultValue: 0, get, set };
}

function basicParam(key: BasicKey, spec: FieldSpec): ParamDef {
  return fieldParam(
    spec,
    (edits) => edits.basic[key],
    (edits, value) => {
      edits.basic[key] = value;
    }
  );
}

function toneParam(key: keyof ToneEdits, spec: FieldSpec): ParamDef {
  return fieldParam(
    spec,
    (edits) => edits.tone[key],
    (edits, value) => {
      edits.tone[key] = value;
    }
  );
}

export const exposure = basicParam('exposure_ev', {
  id: 'exposure',
  label: 'Exposure',
  synonyms: ['EV', 'stops'],
  range: { min: -5, max: 5, step: 0.05, fineStep: 0.01, format: hundredths }
});
export const brightness = basicParam('brightness', {
  id: 'brightness',
  label: 'Brightness',
  synonyms: ['midtones', 'lightness']
});
export const contrast = basicParam('contrast', {
  id: 'contrast',
  label: 'Contrast',
  synonyms: ['punch']
});
export const highlights = toneParam('highlights', {
  id: 'highlights',
  label: 'Highlights',
  synonyms: ['recovery', 'bright areas']
});
export const shadows = toneParam('shadows', {
  id: 'shadows',
  label: 'Shadows',
  synonyms: ['fill light', 'dark areas']
});
export const whites = toneParam('whites', {
  id: 'whites',
  label: 'Whites',
  synonyms: ['white point', 'clipping']
});
export const blacks = toneParam('blacks', {
  id: 'blacks',
  label: 'Blacks',
  synonyms: ['black point', 'crush']
});
export const texture = basicParam('texture', {
  id: 'texture',
  label: 'Texture',
  synonyms: ['detail', 'skin']
});
export const clarity = basicParam('clarity', {
  id: 'clarity',
  label: 'Clarity',
  synonyms: ['local contrast', 'midtone contrast']
});
export const dehaze = basicParam('dehaze', {
  id: 'dehaze',
  label: 'Dehaze',
  synonyms: ['haze', 'fog', 'atmosphere']
});
export const vibrance = basicParam('vibrance', {
  id: 'vibrance',
  label: 'Vibrance',
  synonyms: ['color', 'smart saturation']
});
export const saturation = basicParam('saturation', {
  id: 'saturation',
  label: 'Saturation',
  synonyms: ['color', 'intensity']
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
