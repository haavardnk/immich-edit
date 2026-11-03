import {
  BASIC_PARAMS,
  PRESENCE,
  TONE,
  clarity,
  dehaze,
  saturation,
  texture,
  vibrance
} from './basic';
import { BW, COLOR_PARAMS, LUT } from './color';
import { COLOR_GRADING, GRADING_PARAMS } from './colorGrading';
import { DETAIL_PARAMS, NOISE_REDUCTION, SHARPENING } from './detail';
import { EFFECTS_PARAMS, GRAIN, VIGNETTE } from './effects';
import { HSL, HSL_PARAMS } from './hsl';
import { LENS, LENS_PARAMS } from './lens';
import { LENS_BLUR, LENS_BLUR_PARAMS } from './lensBlur';
import type { ParamDef, SectionDef, TouchGroup } from './types';

export const PARAMS: readonly ParamDef[] = [
  ...BASIC_PARAMS,
  ...DETAIL_PARAMS,
  ...EFFECTS_PARAMS,
  ...LENS_PARAMS,
  ...LENS_BLUR_PARAMS,
  ...HSL_PARAMS,
  ...GRADING_PARAMS,
  ...COLOR_PARAMS
];

export const DESKTOP_SECTIONS: readonly SectionDef[] = [
  TONE,
  PRESENCE,
  SHARPENING,
  NOISE_REDUCTION,
  VIGNETTE,
  GRAIN,
  LENS,
  LENS_BLUR,
  HSL,
  COLOR_GRADING,
  BW,
  LUT
];

const flat = (section: SectionDef): readonly ParamDef[] => section.rows.flat();

export const TOUCH_GROUPS: readonly TouchGroup[] = [
  {
    id: 'light',
    label: 'Light',
    views: [{ id: 'tone', label: 'Tone', params: flat(TONE) }]
  },
  {
    id: 'color',
    label: 'Color',
    views: [
      { id: 'basic', label: 'Basic', params: [vibrance, saturation] },
      { id: 'mix', label: 'Mix', params: flat(HSL) },
      { id: 'grading', label: 'Grading', params: flat(COLOR_GRADING) },
      { id: 'bw', label: 'B&W', params: flat(BW) }
    ]
  },
  {
    id: 'effects',
    label: 'Effects',
    views: [
      {
        id: 'basic',
        label: 'Basic',
        params: [texture, clarity, dehaze, ...flat(VIGNETTE), ...flat(GRAIN)]
      },
      { id: 'lens_blur', label: 'Lens Blur', params: flat(LENS_BLUR) }
    ]
  },
  {
    id: 'detail',
    label: 'Detail',
    views: [
      { id: 'sharpening', label: 'Sharpening', params: flat(SHARPENING) },
      { id: 'noise', label: 'Noise', params: flat(NOISE_REDUCTION) }
    ]
  },
  {
    id: 'optics',
    label: 'Optics',
    views: [{ id: 'corrections', label: 'Corrections', params: flat(LENS) }]
  },
  {
    id: 'profile',
    label: 'Profile',
    views: [{ id: 'lut', label: 'LUT', params: flat(LUT) }]
  }
];
